//! Identity and execution boundary: never ask candidate code to identify itself.
use super::*;
use crate::replay as wire;
use crate::replay::{Build, Context, Manifest, Receipt, encoded};

impl Invocation {
    pub(super) fn receipt_path(&self) -> PathBuf {
        self.executable().with_extension("build.json")
    }

    pub(super) fn build_identity(
        &self,
        sources: &Sources,
        library: Option<(&Invocation, &Sources)>,
    ) -> Result<Build, String> {
        let mut dependencies = Vec::new();
        if let Some(std) = &self.std {
            dependencies.push(BTreeMap::from([("std".into(), file_digest(std)?)]));
        }
        for dep in &self.deps {
            let mut files = BTreeMap::new();
            artifact_tree(dep, dep, &mut BTreeSet::new(), &mut files)?;
            dependencies.push(files);
        }
        if self.link {
            dependencies.push(BTreeMap::from([
                ("linker:cc".into(), file_digest(Path::new("/usr/bin/cc"))?),
                ("linker:ld".into(), file_digest(Path::new("/usr/bin/ld"))?),
            ]));
        }
        let mut options = self.semantic_options(library.is_some());
        if let Some((library, _)) = library {
            options.extend(
                library
                    .semantic_options(false)
                    .into_iter()
                    .map(|arg| format!("paired-library:{arg}")),
            );
        }
        Ok(Build {
            source: (*sources.logical).clone(),
            library: library
                .map(|(_, s)| (*s.logical).clone())
                .unwrap_or_default(),
            dependencies,
            compiler: env!("TIDE_COMPILER_ID").into(),
            target: env!("TIDE_COMPILER_ID")
                .split(';')
                .find_map(|line| line.strip_prefix("host: "))
                .ok_or("compiler target identity missing")?
                .into(),
            profile: self.profile.clone(),
            options,
            driver: file_digest(&std::env::current_exe().map_err(|e| e.to_string())?)?,
            versions: wire::VERSIONS.into(),
        })
    }

    fn semantic_options(&self, paired: bool) -> Vec<String> {
        // Normalize only locators the driver owns, never arbitrary strings in semantic flags.
        let normalized = Invocation {
            source: PathBuf::from(format!(
                "source/{}",
                self.source
                    .strip_prefix(&self.remap[0].0)
                    .unwrap_or(&self.source)
                    .display()
            )),
            out: "output".into(),
            std: self.std.as_ref().map(|_| "std".into()),
            deps: self
                .deps
                .iter()
                .enumerate()
                .map(|(i, _)| format!("dependency-{i}").into())
                .collect(),
            name: self.name.clone(),
            kind: self.kind.clone(),
            test: self.test,
            tested_library: self.tested_library,
            link: self.link,
            profile: self.profile.clone(),
            remap: self
                .remap
                .iter()
                .enumerate()
                .map(|(i, (_, role))| (format!("source-root-{i}").into(), role.clone()))
                .collect(),
            ..Invocation::default()
        };
        let mut options = normalized.rustc_args(
            Path::new("output"),
            paired.then(|| Path::new("paired-library")),
        );
        for value in &mut options {
            if value == env!("TIDE_SYSROOT") {
                *value = "pinned-sysroot".into();
            }
        }
        options.push(format!(
            "admission:unsafe={};backstop={};ice={}",
            self.grants.unsafe_, self.backstop_only, self.test_ice
        ));
        options
    }

    pub(super) fn validate_replay(&self, build: &Build) -> Result<Option<Manifest>, String> {
        let Some(path) = &self.replay else {
            return Ok(None);
        };
        let manifest: Manifest = wire::read(path)?;
        manifest.validate()?;
        manifest.build.compare(build)?;
        Ok(Some(manifest))
    }

    pub(super) fn publish_receipt(&self, staging: &Path, build: &Build) -> Result<(), String> {
        let receipt = Receipt {
            format: "tide-build/1".into(),
            build: build.clone(),
            executable: file_digest(&staging.join("executable"))?,
            tests: if self.test {
                wire::read(&staging.join("test-config.json"))?
            } else {
                BTreeMap::new()
            },
        };
        fs::write(staging.join("build.json"), encoded(&receipt)).map_err(|e| e.to_string())
    }

    pub(super) fn run_test_process(
        &self,
        manifest: Option<Manifest>,
        current: &Build,
        artifacts: Option<&Path>,
    ) -> Result<ExitCode, String> {
        if self.run_test.is_none() && manifest.is_none() {
            return Ok(ExitCode::SUCCESS);
        }
        let receipt_path = artifacts.map_or_else(|| self.receipt_path(), |p| p.join("build.json"));
        let receipt: Receipt = wire::read(&receipt_path)?;
        if receipt.format != "tide-build/1" {
            return Err("replay receipt format mismatch".into());
        }
        receipt.build.compare(current)?;
        if let Some(manifest) = &manifest {
            manifest.build.compare(&receipt.build)?;
        }
        let name = self
            .run_test
            .as_deref()
            .unwrap_or_else(|| &manifest.as_ref().unwrap().test)
            .to_owned();
        if name.is_empty() || name.starts_with('-') {
            return Err("replay test identity is invalid".into());
        }
        let current_config = receipt
            .tests
            .get(&name)
            .ok_or("replay test identity/configuration is not a Tide harness test")?;
        if let Some(manifest) = &manifest {
            if &manifest.config != current_config {
                return Err(
                    "replay configuration mismatch; use fresh --seed for changed inputs".into(),
                );
            }
        }
        let directory = self.out.join("target/sim");
        fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
        // Printed commands name operator-selected build inputs, never paths from a manifest.
        let mut command = vec![
            std::env::current_exe()
                .map_err(|e| e.to_string())?
                .display()
                .to_string(),
            "--test".into(),
            format!("--crate-type={}", self.kind),
            "--emit=link".into(),
            format!("--tide-profile={}", self.profile),
            format!("--crate-name={}", self.name),
            "--out-dir".into(),
            self.out.display().to_string(),
        ];
        if let Some(std) = &self.std {
            command.extend(["--tide-std".into(), std.display().to_string()]);
        }
        for dep in &self.deps {
            command.extend(["--tide-dep-dir".into(), dep.display().to_string()]);
        }
        // `test_library` is retained by the paired path after it resolves the library.
        if let Some(lib) = &self.test_library {
            command.extend(["--tide-test-library".into(), lib.display().to_string()]);
        }
        command.push(self.source.display().to_string());
        let context_dir = create_test_staging(&directory)?;
        // A concurrent publication must not replace the bytes between validation and exec.
        let executable = context_dir.join("test");
        let source = artifacts.map_or_else(|| self.executable(), |p| p.join("executable"));
        let bound = fs::copy(source, &executable)
            .map_err(|e| e.to_string())
            .and_then(|_| {
                if file_digest(&executable)? == receipt.executable {
                    Ok(())
                } else {
                    Err("replay executable/receipt mismatch".into())
                }
            });
        if let Err(error) = bound {
            let _ = fs::remove_dir_all(&context_dir);
            return Err(error);
        }
        let replaying = manifest.is_some();
        let context = Context {
            receipt,
            completion: context_dir.join("reproduced").display().to_string(),
            directory: directory.display().to_string(),
            command,
            replay: manifest,
        };
        let context_path = context_dir.join("context.json");
        fs::write(&context_path, encoded(&context)).map_err(|e| e.to_string())?;
        let mut process = Command::new(executable);
        process
            .args([
                "--exact",
                name.as_str(),
                "--nocapture",
                "--include-ignored",
                "--test-threads=1",
            ])
            .env_clear()
            .env(wire::CONTEXT, &context_path)
            .stdin(Stdio::null());
        if let Some(seed) = self.seed {
            process
                .env("TIDE_SIM_SEED", seed.to_string())
                .env("TIDE_TEST_SEED", seed.to_string());
        }
        let result = process.status().map_err(|e| e.to_string());
        let reproduced = fs::read(&context.completion).is_ok_and(|bytes| bytes == b"reproduced");
        let _ = fs::remove_dir_all(context_dir);
        let status = result?;
        if replaying && !reproduced {
            return Err("replay evaluation did not reproduce the recording (trace, outcome, roots or choice tape mismatch)".into());
        }
        Ok(ExitCode::from(status.code().unwrap_or(2) as u8))
    }
}
fn file_digest(path: &Path) -> Result<String, String> {
    // Use rustc's streaming SHA-256 for large supplied artifacts.
    let hash = SourceFileHash::new(
        SourceFileHashAlgorithm::Sha256,
        fs::File::open(path).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    Ok(hash
        .hash_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}
fn artifact_tree(
    root: &Path,
    path: &Path,
    seen: &mut BTreeSet<PathBuf>,
    files: &mut BTreeMap<String, String>,
) -> Result<(), String> {
    let canonical = fs::canonicalize(path).map_err(|e| e.to_string())?;
    if canonical.is_dir() {
        if !seen.insert(canonical.clone()) {
            return Err("dependency artifact directory cycle/alias".into());
        }
        let mut entries = fs::read_dir(path)
            .map_err(|e| e.to_string())?
            .map(|e| e.map(|e| e.path()))
            .collect::<io::Result<Vec<_>>>()
            .map_err(|e| e.to_string())?;
        entries.sort();
        for entry in entries {
            artifact_tree(root, &entry, seen, files)?;
        }
        seen.remove(&canonical);
    } else {
        files.insert(
            path.strip_prefix(root)
                .unwrap()
                .to_str()
                .ok_or("non-UTF-8 dependency artifact name")?
                .into(),
            file_digest(path)?,
        );
    }
    Ok(())
}

/// Read only macro-produced metadata from the same expanded tree that admission checks.
pub(super) fn test_configs(tcx: TyCtxt<'_>) -> BTreeMap<String, BTreeMap<String, String>> {
    let crates = crate::crates::Crates::new(tcx);
    let mut tests = BTreeMap::new();
    for id in tcx.hir_free_items() {
        let def = id.owner_id.def_id;
        if tcx.def_kind(def) != rustc_hir::def::DefKind::Fn {
            continue;
        }
        let mut config = BTreeMap::new();
        for attr in tcx.hir_attrs(id.hir_id()) {
            let Some(text) = attr.doc_str() else {
                continue;
            };
            let Some(text) = text.as_str().strip_prefix("__tide_replay_config:") else {
                continue;
            };
            let expansion = attr.span().ctxt().outer_expn_data();
            if !expansion.macro_def_id.is_some_and(|mac| {
                crates.exported(mac) && matches!(tcx.item_name(mac).as_str(), "test" | "sim_test")
            }) {
                continue;
            }
            if let Some((key, value)) = text.split_once(':') {
                config.insert(key.into(), value.into());
            }
        }
        if !config.is_empty() {
            use rustc_middle::ty::print::{
                with_crate_prefix, with_no_trimmed_paths, with_no_visible_paths,
            };
            let path = with_no_visible_paths!(with_no_trimmed_paths!(with_crate_prefix!(
                tcx.def_path_str(def)
            )));
            let name = path.strip_prefix("crate::").unwrap_or(&path);
            tests.insert(name.into(), wire::configuration(config));
        }
    }
    tests
}
