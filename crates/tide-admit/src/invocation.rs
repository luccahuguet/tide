//! Fixed operator interface, source snapshot, successful build cache (partial RUN-2), and the rustc
//! callbacks that run the admission checks.
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use rustc_ast::ast::{AttrKind, Attribute, Item, ItemKind, MacCall};
use rustc_ast::token::{Delimiter, TokenKind};
use rustc_ast::tokenstream::{TokenStream, TokenTree};
use rustc_ast::visit::{self, Visitor};
use rustc_driver::{Callbacks, Compilation};
use rustc_hir::def_id::CrateNum;
use rustc_interface::interface::{Compiler, Config};
use rustc_middle::ty::TyCtxt;
use rustc_session::config::CrateType;
use rustc_session::parse::ParseSess;
use rustc_span::fatal_error::FatalError;
use rustc_span::source_map::{FileLoader, get_source_map};
use rustc_span::{
    FileName, RemapPathScopeComponents, SourceFileHash, SourceFileHashAlgorithm, sym,
};

use crate::diag::{Code, Diagnostics, RustcLabels};
use crate::synthesis::{self, Insertions};
use crate::walk::{Grants, Holes, check_crate, check_mono_items};

mod replay;
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
mod sandbox;
/// Elsewhere RUN-3's level is `none`, and the linked run modes are unavailable (x86_64 Linux).
#[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
mod sandbox {
    use std::path::{Path, PathBuf};
    use std::process::Command;
    pub(super) struct RunDir {
        pub(super) path: PathBuf,
    }
    impl RunDir {
        pub(super) fn create(_: &Path) -> Result<Self, String> {
            Err("test and simulation runs need an x86_64 Linux host (sandbox: none)".into())
        }
        pub(super) fn token(&self) -> String {
            String::new()
        }
        pub(super) fn publish(self, _: &Path) -> Result<(), String> {
            Ok(())
        }
        pub(super) fn remove(self) {}
    }
    pub(super) struct Sandbox;
    impl Sandbox {
        pub(super) fn prepare(_: &RunDir) -> Result<Self, String> {
            Ok(Self)
        }
        pub(super) fn level(&self) -> &str {
            "none"
        }
        pub(super) fn apply(self, _: &mut Command) {}
    }
}

const TEST_LIBRARY_NAME: &str = "tide_candidate";

/// The directory of the C toolchain that links executables: its `cc` is rustc's linker and GCC
/// searches it for `ld` (`-B`). A build may bake in its own (`TIDE_LINKER_BIN`, read only when
/// the driver is compiled; the Nix flake sets it, docs/nix.md); otherwise the host's `/usr/bin`.
const LINKER_BIN: &str = match option_env!("TIDE_LINKER_BIN") {
    Some(dir) => dir,
    None => "/usr/bin",
};

/// Library directories RUN-3 searches for a test's libraries before the system directories,
/// separated by `:`: the libraries the baked-in linker's executables load (the Nix flake's glibc
/// and libgcc). Read only when the driver is compiled; unset, none. A test binary's own search
/// path (`DT_RUNPATH`) is the untrusted child's and never chooses a grant.
const RUNTIME_LIB_DIRS: &str = match option_env!("TIDE_RUNTIME_LIB_DIRS") {
    Some(dirs) => dirs,
    None => "",
};

/// A packaging mistake fails the driver's build: a relative or empty directory would resolve
/// against the operator's working directory, which would then choose the linker or a grant.
const _: () = {
    assert!(
        matches!(LINKER_BIN.as_bytes(), [b'/', ..]),
        "TIDE_LINKER_BIN must be an absolute directory"
    );
    assert!(
        absolute_entries(RUNTIME_LIB_DIRS),
        "TIDE_RUNTIME_LIB_DIRS must list absolute directories separated by `:`, none empty"
    );
};

/// Whether each `:`-separated entry of a nonempty `dirs` starts with `/`; an empty list holds
/// none, while an empty entry, such as a trailing `:`, fails.
const fn absolute_entries(dirs: &str) -> bool {
    let bytes = dirs.as_bytes();
    let mut i = 0;
    let mut entry_start = true;
    while i < bytes.len() {
        if entry_start && bytes[i] != b'/' {
            return false;
        }
        entry_start = bytes[i] == b':';
        i += 1;
    }
    bytes.is_empty() || !entry_start
}

/// The linker's `cc` and `ld`, whose bytes identify a linked build (cache key and receipt).
fn linker_tools() -> [PathBuf; 2] {
    [Path::new(LINKER_BIN).join("cc"), Path::new(LINKER_BIN).join("ld")]
}

/// Tide's std and the directory of its dependencies, used when the operator passes no
/// `--tide-std`. Only a packaged build bakes them in (the Nix flake's `tidelang-admit`), so its
/// driver runs on its own; an unset pair keeps Rust's pinned std as the default.
const DEFAULT_STD: Option<(&str, &str)> =
    match (option_env!("TIDE_DEFAULT_STD"), option_env!("TIDE_DEFAULT_DEP_DIR")) {
        (Some(std), Some(deps)) => Some((std, deps)),
        (None, None) => None,
        _ => panic!("set TIDE_DEFAULT_STD and TIDE_DEFAULT_DEP_DIR together"),
    };

struct Admission {
    grants: Grants,
    /// The staging directory, which gets the trust artifact (`interface.rs`).
    out: PathBuf,
    /// The walk's trust holes, for the trust artifact.
    holes: Holes,
    labels: RustcLabels,
    /// Whether the operator supplied Tide's std (`--tide-std`), whose `#[tide::main]` every
    /// binary's entry point needs (ROOT-1); the driver's own tests on Rust's std have none.
    tide_std: bool,
    sources: Sources,
    library_artifact: Option<PathBuf>,
    test_library: Option<CrateNum>,
    write_interface: bool,
    backstop_only: bool,
    test_ice: bool,
    synthesize: bool,
    insertions: Insertions,
    test_configs: BTreeMap<String, BTreeMap<String, String>>,
    /// The paths of the libraries this build loaded, spelled as rustc's crate store has them
    /// (the search path joined with the entry rustc chose, symbolic links unresolved), except the
    /// pinned sysroot's and the paired library's (REPLAY-2's dependency identity).
    loaded: BTreeSet<PathBuf>,
}

impl Callbacks for Admission {
    fn config(&mut self, config: &mut Config) {
        config.file_loader = Some(Box::new(self.sources.clone()));
        let labels = self.labels.clone();
        config.psess_created = Some(Box::new(move |psess| labels.install(psess)));
        // The backstop's roots: a metadata check exports nothing, which leaves a library's mono
        // items without roots, so rustc gets the items a build exports.
        config.override_queries = Some(|_, providers| {
            providers.queries.reachable_non_generics =
                |tcx, _| rustc_codegen_ssa::back::symbol_export::reachable_non_generics_helper(tcx);
        });
    }
    fn after_expansion<'tcx>(&mut self, _: &Compiler, tcx: TyCtxt<'tcx>) -> Compilation {
        assert!(!self.test_ice, "`--tide-test-ice`: a seeded compiler crash");
        // Before `check_crate` lowers the crate, which takes the expanded AST.
        let (surface_errors, unused_imports) =
            check_expanded(tcx, self.tide_std, &self.sources.insertions);
        self.test_library = self.library_artifact.as_ref().and_then(|path| {
            tcx.crates(()).iter().copied().find(|&krate| {
                tcx.crate_extern_paths(krate)
                    .iter()
                    .any(|loaded| loaded == path)
            })
        });
        if self.synthesize && tcx.dcx().has_errors().is_none() && unused_imports.is_empty() {
            self.insertions = synthesis::plan(tcx, self.test_library);
            tcx.dcx().abort_if_errors();
            if !self.insertions.is_empty() {
                return Compilation::Stop;
            }
        }
        self.test_configs = replay::test_configs(tcx);
        self.holes = check_crate(
            tcx,
            self.grants,
            self.tide_std,
            !self.backstop_only,
            self.test_library,
            self.library_artifact.is_some(),
            &self.labels,
            &self.sources.insertions,
            surface_errors,
            &unused_imports,
        );
        Compilation::Continue
    }
    fn after_analysis<'tcx>(&mut self, _: &Compiler, tcx: TyCtxt<'tcx>) -> Compilation {
        let canonical = |path: &Path| fs::canonicalize(path).unwrap_or_else(|_| path.into());
        let sysroot = canonical(Path::new(env!("TIDE_SYSROOT")));
        let library = self.library_artifact.as_deref().map(canonical);
        self.loaded = tcx
            .crates(())
            .iter()
            .flat_map(|&krate| tcx.crate_extern_paths(krate).iter().cloned())
            .filter(|path| {
                // Resolved only to recognize the sysroot and the paired library.
                let resolved = canonical(path);
                !resolved.starts_with(&sysroot) && Some(&resolved) != library.as_ref()
            })
            .collect();
        // rustc's analysis stops before this on any error, the walk's included.
        check_mono_items(tcx, self.grants, &mut self.holes, self.test_library);
        // In link mode too, no rejected instance may reach codegen or the linker.
        tcx.dcx().abort_if_errors();
        if self.write_interface
            && self.tide_std
            && tcx.dcx().has_errors().is_none()
            && let Err(e) = crate::interface::write(tcx, &self.holes, &self.out)
        {
            tcx.dcx()
                .fatal(format!("cannot write the trust artifact: {e}"));
        }
        Compilation::Continue
    }
}

#[derive(Default)]
struct Invocation {
    source: PathBuf,
    out: PathBuf,
    std: Option<PathBuf>,
    deps: Vec<PathBuf>,
    name: String,
    kind: String,
    test: bool,
    test_library: Option<PathBuf>,
    /// The library a `--tide-test-library` suite tests: its contracts run as in a test build
    /// (`--cfg __tide_test_library`) while `cfg(test)` stays off.
    tested_library: bool,
    link: bool,
    profile: String,
    grants: Grants,
    /// `--tide-test-backstop`: the walk skips its obligation steps, so tests can show what the
    /// backstop alone rejects.
    backstop_only: bool,
    /// `--tide-test-ice`: the compiler crashes after expansion, so tests can show that a crash
    /// writes no report.
    test_ice: bool,
    compile: Option<String>,
    replay: Option<PathBuf>,
    seed: Option<u64>,
    run_test: Option<String>,
    /// `--tide-eval-timeout SECS`: a simulation test's per-evaluation wall-clock limit (300 s).
    timeout: Option<u64>,
    remap: Vec<(PathBuf, String)>,
}

impl Invocation {
    fn parse(args: &[String]) -> Result<Self, String> {
        let mut this = Self {
            out: PathBuf::from("."),
            kind: "lib".into(),
            profile: "debug".into(),
            ..Self::default()
        };
        let mut args = args.iter();
        while let Some(arg) = args.next() {
            let (flag, inline) = arg
                .split_once('=')
                .map_or((arg.as_str(), None), |(a, b)| (a, Some(b)));
            if matches!(
                flag,
                "--test" | "--tide-grant" | "--tide-test-backstop" | "--tide-test-ice"
            ) {
                match (flag, inline) {
                    ("--test", None) => this.test = true,
                    ("--tide-test-backstop", None) => this.backstop_only = true,
                    ("--tide-test-ice", None) => this.test_ice = true,
                    ("--tide-grant", Some("unsafe")) => this.grants.unsafe_ = true,
                    _ => return Err(format!("unsupported argument `{arg}`")),
                }
                continue;
            }
            if matches!(
                flag,
                "--edition"
                    | "--crate-type"
                    | "--crate-name"
                    | "--emit"
                    | "--out-dir"
                    | "--error-format"
                    | "--tide-std"
                    | "--tide-dep-dir"
                    | "--tide-test-library"
                    | "--tide-compile"
                    | "--tide-profile"
                    | "--replay"
                    | "--seed"
                    | "--tide-run-test"
                    | "--tide-eval-timeout"
            ) {
                let value = inline
                    .or_else(|| args.next().map(String::as_str))
                    .ok_or_else(|| format!("missing value for `{flag}`"))?;
                if value.is_empty() || value.starts_with('-') {
                    return Err(format!("invalid value for `{flag}`"));
                }
                match flag {
                    "--replay" if this.replay.is_none() => this.replay = Some(value.into()),
                    "--seed" if this.seed.is_none() => this.seed = Some(value.parse().map_err(|_| "--seed requires a decimal u64")?),
                    "--tide-run-test" if this.run_test.is_none() => this.run_test = Some(value.into()),
                    "--tide-eval-timeout" if this.timeout.is_none() => {
                        this.timeout = Some(value.parse().ok().filter(|&s| s > 0).ok_or("--tide-eval-timeout requires a positive number of seconds")?)
                    }
                    "--edition" if value == "2024" => (),
                    "--crate-type" if matches!(value, "lib" | "bin") => this.kind = value.into(),
                    "--emit" if matches!(value, "metadata" | "link") => this.link = value == "link",
                    "--tide-profile" if matches!(value, "debug" | "release") => {
                        this.profile = value.into()
                    }
                    "--error-format" if value == "short" => (),
                    "--crate-name" => this.name = value.into(),
                    "--out-dir" => this.out = value.into(),
                    "--tide-std" if this.std.is_none() => this.std = Some(value.into()),
                    "--tide-dep-dir" => this.deps.push(value.into()),
                    "--tide-test-library" if this.test_library.is_none() => {
                        this.test_library = Some(value.into());
                    }
                    "--tide-compile" if this.compile.is_none() => this.compile = Some(value.into()),
                    _ => return Err(format!("unsupported argument `{arg}`")),
                }
            } else if !arg.starts_with(['-', '@']) && this.source.as_os_str().is_empty() {
                this.source = arg.into();
            } else {
                return Err(format!("unsupported argument `{arg}`"));
            }
        }
        if this.std.is_none()
            && let Some((std, deps)) = DEFAULT_STD
        {
            this.std = Some(std.into());
            this.deps.push(deps.into());
        }
        if this.source.extension().is_none_or(|ext| ext != "rs") {
            return Err("expected one .rs source file".into());
        }
        if this.link && this.kind != "bin" && !this.test {
            return Err("link output requires `--crate-type=bin` or `--test`".into());
        }
        if this.link && !env!("TIDE_COMPILER_ID").contains("host: x86_64-unknown-linux-gnu;") {
            return Err("link output currently requires an x86_64-unknown-linux-gnu host".into());
        }
        if this.name.is_empty() {
            this.name = this
                .source
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("tide")
                .replace('-', "_");
        }
        if !this
            .name
            .chars()
            .enumerate()
            .all(|(i, c)| c == '_' || c.is_ascii_alphabetic() || i > 0 && c.is_ascii_digit())
        {
            return Err("invalid crate name".into());
        }
        if let Some(library) = &this.test_library {
            if !this.test
                || this.grants.unsafe_
                || this.backstop_only
                || this.test_ice
                || this.compile.is_some()
            {
                return Err(
                    "`--tide-test-library` requires `--test` and full admission without grants"
                        .into(),
                );
            }
            if library.extension().is_none_or(|ext| ext != "rs") {
                return Err("`--tide-test-library` requires a .rs source".into());
            }
            if this.name == TEST_LIBRARY_NAME {
                return Err(format!(
                    "`--tide-test-library` reserves crate name `{TEST_LIBRARY_NAME}`"
                ));
            }
        }
        if this.replay.is_some() && (this.seed.is_some() || this.run_test.is_some()) {
            return Err("--replay accepts no --seed or --tide-run-test configuration override".into());
        }
        if this.timeout.is_some() && this.run_test.is_none() && this.replay.is_none() {
            return Err("--tide-eval-timeout requires --tide-run-test or --replay".into());
        }
        if this.seed.is_some() && this.run_test.is_none() {
            return Err("fresh --seed requires --tide-run-test NAME".into());
        }
        if (this.replay.is_some() || this.run_test.is_some()) && (!this.link || !this.test || this.std.is_none() || this.grants.unsafe_ || this.backstop_only || this.test_ice) {
            return Err("replay/test execution requires --test --emit=link --tide-std and full admission without grants".into());
        }
        Ok(this)
    }

    fn rustc_args(&self, out: &Path, library_artifact: Option<&Path>) -> Vec<String> {
        let mut args = vec![
            "tide-admit".into(),
            "--edition=2024".into(),
            format!("--crate-type={}", self.kind),
            format!("--crate-name={}", self.name),
            format!("--emit={}", if self.link { "link" } else { "metadata" }),
            "--error-format=short".into(),
            "-Zallow-features=".into(),
            "-Zpolonius=no".into(),
            // TD0913 owns unused imports independently of Rust lint levels.
            "-Aunused-imports".into(),
            // The backstop's mono items: the crate makes its own instances of upstream generics
            // instead of linking an upstream crate's.
            "-Zshare-generics=no".into(),
            // MIR inlining would hide instances from the backstop before LLVM codegen.
            "-Zinline-mir=no".into(),
            // Inferred cross-crate inlining would remove exported library roots in release.
            "-Zcross-crate-inline-threshold=never".into(),
            "-Coverflow-checks=on".into(),
            format!(
                "-Copt-level={}",
                if self.profile == "release" { 3 } else { 0 }
            ),
            format!(
                "-Cdebug-assertions={}",
                if self.profile == "release" {
                    "off"
                } else {
                    "on"
                }
            ),
            "--sysroot".into(),
            env!("TIDE_SYSROOT").into(),
            self.source.display().to_string(),
        ];
        // Full debug information, as Cargo's dev profile gives Rust, so debuggers see source lines
        // and locals. It enters the replay identity through these arguments.
        if self.profile != "release" {
            args.push("-Cdebuginfo=2".into());
        }
        if self.link {
            args.extend([
                "-Zunstable-options".into(),
                format!("-Ztemps-dir={}", out.display()),
            ]);
        }
        if self.link && self.kind != "rlib" {
            let [cc, _] = linker_tools();
            args.extend([
                format!("-Clinker={}", cc.display()),
                "-Clinker-flavor=gnu-cc".into(),
                "-Clink-self-contained=-linker".into(),
                format!("-Clink-arg=-B{LINKER_BIN}"),
                "-o".into(),
                out.join("executable").display().to_string(),
            ]);
        } else {
            args.extend(["--out-dir".into(), out.display().to_string()]);
        }
        if self.link && (self.test || self.tested_library) {
            args.push("--remap-path-scope=macro,object".into());
            for (root, role) in &self.remap {
                args.push(format!("--remap-path-prefix={}={role}", root.display()));
            }
        }
        if self.test {
            args.push("--test".into());
        }
        if self.tested_library {
            args.extend(["--cfg".into(), "__tide_test_library".into()]);
        }
        if let Some(std) = &self.std {
            // Tide pins its prelude's Truncate alongside core's unstable inherent truncate.
            // At a pin bump, delete Truncate if core's truncate has stabilized; rebuild the
            // conformance corpus without -Aunstable-name-collisions to expose other collisions.
            args.push("-Aunstable-name-collisions".into());
            args.extend(["--extern".into(), format!("std={}", std.display())]);
        }
        if let Some(library) = library_artifact {
            args.extend([
                "--extern".into(),
                format!("{TEST_LIBRARY_NAME}={}", library.display()),
            ]);
        }
        for dep in &self.deps {
            args.extend(["-L".into(), format!("dependency={}", dep.display())]);
        }
        args
    }

    fn executable(&self) -> PathBuf {
        self.out.join(format!(
            "{}{}",
            self.name,
            if self.test { "-test" } else { "" }
        ))
    }

    fn compile(
        &self,
        out: &Path,
        sources: Sources,
        library_artifact: Option<&Path>,
        write_interface: bool,
        loaded: &mut BTreeSet<PathBuf>,
    ) -> ExitCode {
        let rustc_args = self.rustc_args(out, library_artifact);
        let mut admission = Admission {
            grants: self.grants,
            out: out.into(),
            holes: Holes::new(),
            labels: RustcLabels::default(),
            sources,
            tide_std: self.std.is_some(),
            library_artifact: library_artifact.map(Path::to_path_buf),
            test_library: None,
            write_interface,
            backstop_only: self.backstop_only,
            test_ice: self.test_ice,
            synthesize: self.test && self.std.is_some(),
            insertions: Insertions::default(),
            test_configs: BTreeMap::new(),
            loaded: BTreeSet::new(),
        };
        let status = rustc_driver::catch_with_exit_code(|| {
            rustc_driver::run_compiler(&rustc_args, &mut admission);
            if !admission.insertions.is_empty() {
                // The edits depend only on the snapshot, compiler, std and paired library.
                // Reuse the snapshot and run all checks on the final compilation.
                admission.sources.insertions = Arc::new(std::mem::take(&mut admission.insertions));
                admission.sources.intent = Arc::default();
                admission.synthesize = false;
                rustc_driver::run_compiler(&rustc_args, &mut admission);
            }
        });
        loaded.append(&mut admission.loaded);
        if status == ExitCode::SUCCESS && self.link && self.test {
            if let Err(e) = fs::write(out.join("test-config.json"), crate::replay::encoded(&admission.test_configs)) {
                eprintln!("cannot write test configuration receipt: {e}");
                return ExitCode::FAILURE;
            }
        }
        status
    }

    fn package_root(&mut self, cwd: &Path) -> Result<PathBuf, String> {
        // Lexical, so a `..` cannot hide the package root from the checks below.
        let mut source = PathBuf::new();
        for component in cwd.join(&self.source).components() {
            match component {
                std::path::Component::ParentDir => {
                    source.pop();
                }
                std::path::Component::CurDir => (),
                _ => source.push(component.as_os_str()),
            }
        }
        self.source = source;
        let source_dir = self.source.parent().ok_or("missing source directory")?;
        let lexical_package = if source_dir.file_name().is_some_and(|s| s == "src") {
            source_dir.parent().ok_or("missing package directory")?
        } else {
            source_dir
        };
        let package =
            fs::canonicalize(lexical_package).map_err(|_| "cannot resolve package directory")?;
        if package != lexical_package {
            return Err("package path must not traverse a directory symlink".into());
        }
        Ok(package)
    }

    fn source_paths(&self, package: &Path) -> Result<(PathBuf, PathBuf), String> {
        let source_root = fs::canonicalize(self.source.parent().ok_or("missing source directory")?)
            .map_err(|_| "cannot resolve source directory")?;
        if !source_root.starts_with(package) {
            return Err("source directory leaves its package".into());
        }
        let source =
            fs::canonicalize(&self.source).map_err(|_| "cannot resolve candidate source")?;
        if !source.starts_with(&source_root) {
            return Err("candidate source leaves its source directory".into());
        }
        Ok((source_root, source))
    }

    fn trusted_inputs(&mut self, package: &Path) -> Result<(), String> {
        if let Some(std) = &mut self.std {
            *std = fs::canonicalize(&*std).map_err(|_| "cannot resolve trusted std")?;
            if !std.is_file() || std.starts_with(package) {
                return Err("trusted std must be a file outside the candidate package".into());
            }
        }
        for dep in &mut self.deps {
            *dep = fs::canonicalize(&*dep)
                .map_err(|_| "cannot resolve trusted dependency directory")?;
            if !dep.is_dir() || dep.starts_with(package) || package.starts_with(&*dep) {
                return Err(
                    "trusted dependencies must be separate from the candidate package".into(),
                );
            }
        }
        Ok(())
    }

    fn clear_outputs(&self) -> Result<(), String> {
        // A refused replay must leave previously published outputs intact.
        if self.replay.is_some() { return Ok(()); }
        // Failed builds leave neither an earlier interface nor their requested executable.
        if self.link {
            remove_if_present(&self.executable())?;
            remove_if_present(&self.receipt_path())?;
        }
        remove_if_present(&self.out.join(format!("{}.interface.json", self.name)))
    }

    fn compile_with_library(
        &mut self,
        cwd: &Path,
        package: &Path,
        library: PathBuf,
    ) -> Result<ExitCode, String> {
        let mut library = Invocation {
            source: library,
            std: self.std.clone(),
            deps: self.deps.clone(),
            name: TEST_LIBRARY_NAME.into(),
            kind: "rlib".into(),
            // The downstream mono-item backstop needs the rlib's export and MIR metadata.
            link: true,
            profile: self.profile.clone(),
            tested_library: true,
            ..Invocation::default()
        };
        let library_package = library.package_root(cwd)?;
        if library_package.starts_with(package) || package.starts_with(&library_package) {
            return Err("the test and its library must be in separate packages".into());
        }
        if self.out.starts_with(&library_package)
            || self.deps.iter().any(|dep| self.out.starts_with(dep))
        {
            return Err("test output must be outside the library package and trusted dependency directories".into());
        }
        self.clear_outputs()?;
        let (suite_root, suite_source) = self.source_paths(package)?;
        let (root, source) = library.source_paths(&library_package)?;
        if reject_build_script(package) || reject_build_script(&library_package) {
            return Ok(ExitCode::FAILURE);
        }
        library.trusted_inputs(&library_package)?;
        let sources = source_snapshot(&suite_root, &suite_source, &mut Vec::new())?;
        let library_sources = source_snapshot(&root, &source, &mut Vec::new())?;
        self.test_library = Some(library.source.clone());
        library.remap = vec![(root.clone(), "tide-library".into())];
        self.remap = vec![(suite_root.clone(), "tide-source".into()), (root, "tide-library".into())];
        let build = self.build_identity(&sources, Some((&library, &library_sources)))?;
        let manifest = self.validate_replay(&build)?;
        let replaying = manifest.is_some();
        let staging = create_test_staging(&self.out)?;
        let result: Result<ExitCode, String> = (|| {
            let mut loaded = BTreeSet::new();
            let status = library.compile(&staging, library_sources, None, false, &mut loaded);
            if status != ExitCode::SUCCESS {
                return Ok(status);
            }
            let artifact = staging.join(format!("lib{TEST_LIBRARY_NAME}.rlib"));
            let status = self.compile(&staging, sources, Some(&artifact), false, &mut loaded);
            if status == ExitCode::SUCCESS && self.link {
                self.publish_receipt(&staging, &build, &loaded)?;
                if replaying { return self.run_test_process(manifest, &build, Some(&staging)); }
                fs::rename(staging.join("executable"), self.executable()).map_err(|e| e.to_string())?;
                fs::rename(staging.join("build.json"), self.receipt_path()).map_err(|e| e.to_string())?;
            }
            Ok(status)
        })();
        let _ = fs::remove_dir_all(staging);
        let status = result?;
        if status == ExitCode::SUCCESS && !replaying { self.run_test_process(None, &build, None) } else { Ok(status) }
    }
}

fn create_test_staging(out: &Path) -> Result<PathBuf, String> {
    const ATTEMPTS: usize = 16;
    let error = |e| {
        format!(
            "cannot create paired test staging under {}: {e}",
            out.display()
        )
    };
    fs::create_dir_all(out).map_err(error)?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let pid = std::process::id();
    let mut dir = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        dir.mode(0o700);
    }
    // A killed build can leave a directory behind even after its sandbox PID is reused.
    for attempt in 0..ATTEMPTS {
        let staging = out.join(format!(".tide-test-{pid}-{nonce}-{attempt}"));
        match dir.create(&staging) {
            Ok(()) => return Ok(staging),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(error(e)),
        }
    }
    Err(format!(
        "cannot create paired test staging under {}: {ATTEMPTS} names already exist",
        out.display()
    ))
}

fn remove_if_present(path: &Path) -> Result<(), String> {
    match fs::remove_file(path) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e.to_string()),
        _ => Ok(()),
    }
}

fn reject_build_script(package: &Path) -> bool {
    if package.join("build.rs").symlink_metadata().is_err() {
        return false;
    }
    eprintln!(
        "error: {}: package build scripts are not admitted",
        Code::Td0915.as_str()
    );
    true
}

fn source_snapshot(root: &Path, source: &Path, key: &mut Vec<u8>) -> Result<Sources, String> {
    let sources = Sources::snapshot(root, key).map_err(|_| "cannot snapshot candidate sources")?;
    if !sources.files.contains_key(source) {
        return Err("candidate source is not admitted".into());
    }
    Ok(sources)
}

fn record(key: &mut Vec<u8>, label: &[u8], bytes: &[u8]) {
    key.extend_from_slice(&(label.len() as u64).to_le_bytes());
    key.extend_from_slice(label);
    key.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    key.extend_from_slice(bytes);
}

fn fingerprint(path: &Path, key: &mut Vec<u8>) -> io::Result<()> {
    if path.is_dir() {
        let mut entries: Vec<_> = fs::read_dir(path)?
            .map(|e| e.map(|e| e.path()))
            .collect::<io::Result<_>>()?;
        entries.sort();
        for entry in entries {
            fingerprint(&entry, key)?;
        }
    } else {
        let digest = SourceFileHash::new(SourceFileHashAlgorithm::Sha256, fs::File::open(path)?)?;
        record(
            key,
            path.as_os_str().as_encoded_bytes(),
            digest.hash_bytes(),
        );
    }
    Ok(())
}

fn destination(path: &Path) -> io::Result<PathBuf> {
    match fs::canonicalize(path) {
        Ok(path) => Ok(path),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            let parent = path
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
                .unwrap_or(Path::new("."));
            let name = path
                .file_name()
                .ok_or_else(|| io::Error::other("invalid output path"))?;
            Ok(destination(parent)?.join(name))
        }
        Err(error) => Err(error),
    }
}

#[derive(Clone)]
struct Sources {
    files: Arc<BTreeMap<PathBuf, String>>,
    logical: Arc<BTreeMap<String, String>>,
    intent: Arc<Mutex<SourceIntent>>,
    insertions: Arc<Insertions>,
}

#[derive(Default)]
struct SourceIntent {
    module_probes: BTreeSet<PathBuf>,
    guarded: BTreeSet<PathBuf>,
}

impl Sources {
    fn snapshot(root: &Path, key: &mut Vec<u8>) -> io::Result<Self> {
        fn walk(
            root: &Path,
            dir: &Path,
            seen: &mut BTreeSet<PathBuf>,
            files: &mut BTreeMap<PathBuf, String>,
            logical: &mut BTreeMap<String, String>,
            key: &mut Vec<u8>,
        ) -> io::Result<()> {
            let mut entries: Vec<_> = fs::read_dir(dir)?
                .map(|e| e.map(|e| e.path()))
                .collect::<io::Result<_>>()?;
            entries.sort();
            for entry in entries {
                // A dangling or escaping symlink, like a non-UTF-8 file, reads as missing.
                let Ok(canonical) = fs::canonicalize(&entry) else {
                    continue;
                };
                if !canonical.starts_with(root) {
                    continue;
                }
                let name = entry.strip_prefix(root).unwrap().to_str().ok_or_else(|| io::Error::other("non-UTF-8 source name"))?.to_string();
                let target = canonical.strip_prefix(root).unwrap().to_str().ok_or_else(|| io::Error::other("non-UTF-8 source alias"))?;
                if canonical.is_dir() {
                    logical.insert(format!("directory:{name}"), target.into());
                    record(
                        key,
                        entry.as_os_str().as_encoded_bytes(),
                        canonical.as_os_str().as_encoded_bytes(),
                    );
                    if seen.insert(canonical.clone()) {
                        walk(root, &entry, seen, files, logical, key)?;
                    }
                } else if entry.extension().is_some_and(|ext| ext == "rs")
                    && canonical.extension().is_some_and(|ext| ext == "rs")
                {
                    let Ok(source) = fs::read_to_string(&canonical) else {
                        continue;
                    };
                    record(key, entry.as_os_str().as_encoded_bytes(), source.as_bytes());
                    logical.insert(format!("file:{name}"), format!("{target}:{}", crate::replay::digest(source.as_bytes())));
                    files.insert(canonical, source);
                }
            }
            Ok(())
        }
        let mut files = BTreeMap::new();
        let mut logical = BTreeMap::new();
        walk(
            root,
            root,
            &mut BTreeSet::from([root.into()]),
            &mut files,
            &mut logical,
            key,
        )?;
        Ok(Self {
            files: Arc::new(files),
            logical: Arc::new(logical),
            intent: Arc::default(),
            insertions: Arc::default(),
        })
    }

    fn guard(path: &Path, source: &str) {
        // Pre-parse each loaded snapshot file; unloaded modules never cause parser file reads.
        // Silently: rustc's own parse of these bytes reports their syntax errors in its format,
        // and what an unparsable file leaves for expansion still meets the loader.
        let psess = ParseSess::new();
        psess.dcx().make_silent();
        let krate = match rustc_parse::new_parser_from_source_str(
            &psess,
            FileName::Custom(path.display().to_string()),
            source.into(),
            rustc_parse::lexer::StripTokens::ShebangAndFrontmatter,
        ) {
            Ok(mut parser) => parser.parse_crate_mod().map_err(|error| {
                error.emit();
            }),
            Err(errors) => Err(errors.into_iter().for_each(|error| {
                error.emit();
            })),
        };
        if let Ok(krate) = krate {
            Guard {
                psess: &psess,
                path,
            }
            .visit_crate(&krate);
        }
    }
}

impl FileLoader for Sources {
    fn file_exists(&self, path: &Path) -> bool {
        fs::canonicalize(path).ok().is_some_and(|path| {
            let exists = self.files.contains_key(&path);
            if exists {
                // Classify native module intent; snapshot containment still owns authorization.
                self.intent.lock().unwrap().module_probes.insert(path);
            }
            exists
        })
    }

    fn read_file(&self, path: &Path) -> io::Result<String> {
        let canonical = fs::canonicalize(path).ok();
        // Whether a trusted crate's imported source file has this name.
        let foreign = get_source_map().is_some_and(|sm| {
            // Trusted dependencies may remap their own paths. rustc reverses that mapping
            // when requesting imported diagnostic source. Match
            // its mapped identity too; this still supplies only empty text, never a host read.
            let requested = sm
                .path_mapping()
                .to_real_filename(&rustc_span::RealFileName::empty(), path);
            sm.files().iter().any(|file| {
                file.is_imported() && matches!(&file.name, FileName::Real(name)
                    if name.local_path() == Some(path)
                        || name.path(RemapPathScopeComponents::DIAGNOSTICS)
                            == requested.path(RemapPathScopeComponents::DIAGNOSTICS))
            })
        });
        if let Some(canonical) = &canonical
            && let Some(source) = self.files.get(canonical)
        {
            let (guarded, module) = {
                let mut intent = self.intent.lock().unwrap();
                (
                    intent.guarded.contains(canonical),
                    intent.module_probes.remove(canonical),
                )
            };
            if guarded || module || !foreign {
                if !guarded {
                    Self::guard(canonical, source);
                    self.intent
                        .lock()
                        .unwrap()
                        .guarded
                        .insert(canonical.clone());
                }
                return Ok(self
                    .insertions
                    .apply(path, source)
                    .unwrap_or_else(|| source.clone()));
            }
        }
        if foreign {
            // Optional diagnostic source: a hash mismatch marks it absent without a host retry.
            // Never return Err: include! may request this same name and trigger rustc's fallback.
            return Ok(String::new());
        }
        // Not Err: rustc_parse directly reopens a path after *every* loader Err on this pin.
        eprintln!("error: TD0520: compile-time file read denied");
        FatalError.raise()
    }

    fn read_binary_file(&self, path: &Path) -> io::Result<Arc<[u8]>> {
        // rustc reads trusted crates' metadata itself, never through the loader.
        fs::canonicalize(path)
            .ok()
            .and_then(|path| self.files.get(&path))
            .map(|source| source.as_bytes().into())
            .ok_or_else(|| io::ErrorKind::NotFound.into())
    }

    fn current_directory(&self) -> io::Result<PathBuf> {
        std::env::current_dir()
    }
}

struct Guard<'a> {
    psess: &'a ParseSess,
    path: &'a Path,
}

impl Guard<'_> {
    fn reject(&self, span: rustc_span::Span, code: Code, message: &str) -> ! {
        let location = self.psess.source_map().lookup_char_pos(span.lo());
        eprintln!(
            "{}:{}:{}: error: {}: {message}",
            self.path.display(),
            location.line,
            location.col.0 + 1,
            code.as_str()
        );
        FatalError.raise()
    }
}

/// Whether the attribute written as `trees` (inside `#[...]`) loads a file: `path` or
/// `debugger_visualizer`, directly or among the attributes a `cfg_attr` applies.
fn loads_file(trees: &[TokenTree]) -> bool {
    match trees {
        [TokenTree::Token(token, _), rest @ ..] => match token.kind {
            TokenKind::Ident(name, _) => {
                name == sym::path
                    || name == sym::debugger_visualizer
                    || name == sym::cfg_attr
                        && matches!(rest, [TokenTree::Delimited(_, _, Delimiter::Parenthesis, args), ..] if cfg_attr_loads_file(args))
            }
            _ => false,
        },
        _ => false,
    }
}

/// Whether an attribute that `cfg_attr(predicate, attr, ...)` applies loads a file.
fn cfg_attr_loads_file(args: &TokenStream) -> bool {
    let trees: Vec<_> = args.iter().cloned().collect();
    trees
        .split(|tree| matches!(tree, TokenTree::Token(token, _) if token.kind == TokenKind::Comma))
        .skip(1)
        .any(loads_file)
}

fn forbidden_macro_attributes(stream: &TokenStream) -> bool {
    let mut pound = false;
    for tree in stream.iter() {
        match tree {
            TokenTree::Token(token, _) => {
                if token.kind == TokenKind::Bang && pound {
                    continue;
                }
                pound = token.kind == TokenKind::Pound;
            }
            TokenTree::Delimited(_, _, delim, inner) => {
                if pound
                    && *delim == Delimiter::Bracket
                    && loads_file(&inner.iter().cloned().collect::<Vec<_>>())
                {
                    return true;
                }
                if forbidden_macro_attributes(inner) {
                    return true;
                }
                pound = false;
            }
        }
    }
    false
}

impl<'ast> Visitor<'ast> for Guard<'_> {
    fn visit_attribute(&mut self, attr: &'ast Attribute) {
        let forbidden = attr.has_name(sym::path)
            || attr.has_name(sym::debugger_visualizer)
            || attr.has_name(sym::cfg_attr)
                && matches!(&attr.kind, AttrKind::Normal(normal) if matches!(&normal.item.args, rustc_ast::ast::AttrArgs::Delimited(args) if cfg_attr_loads_file(&args.tokens)));
        if forbidden {
            self.reject(
                attr.span,
                Code::Td0911,
                "file-loading attributes are not admitted",
            );
        }
        visit::walk_attribute(self, attr);
    }

    fn visit_mac_call(&mut self, mac: &'ast MacCall) {
        if forbidden_macro_attributes(&mac.args.tokens) {
            self.reject(
                mac.path.span,
                Code::Td0911,
                "file-loading attributes are not admitted in macro arguments",
            );
        }
    }
}

/// Rejects user macro definitions (TD0101), `#![no_std]` (TD0520), `#![no_main]` on a binary
/// (TD0512), and the file-loading attributes that only expansion shows (TD0911), such as ones
/// built from a macro's tokens. `Guard` rejects the written attributes before rustc reads their
/// files; the file loader confines what the rest read. Returns emitted driver errors and
/// deferred unused imports, so later checks can exclude earlier errors and rejected imports.
fn check_expanded(
    tcx: TyCtxt<'_>,
    tide_std: bool,
    insertions: &Insertions,
) -> (usize, Vec<rustc_span::Span>) {
    struct Expanded<'tcx> {
        tcx: TyCtxt<'tcx>,
        diags: Diagnostics,
    }
    impl<'ast> Visitor<'ast> for Expanded<'_> {
        fn visit_attribute(&mut self, attr: &'ast Attribute) {
            if attr.has_name(sym::path) || attr.has_name(sym::debugger_visualizer) {
                self.diags.report(
                    self.tcx,
                    attr.span,
                    Code::Td0911,
                    "file-loading attributes are not admitted".into(),
                    None,
                );
            }
            visit::walk_attribute(self, attr);
        }

        fn visit_item(&mut self, item: &'ast Item) {
            // A trusted crate's macro may define one; the candidate's own source may not.
            if matches!(item.kind, ItemKind::MacroDef(..))
                && !item.span.in_external_macro(self.tcx.sess.source_map())
            {
                self.diags.report(
                    self.tcx,
                    item.span,
                    Code::Td0101,
                    "user macro definitions are not admitted".into(),
                    None,
                );
            }
            visit::walk_item(self, item);
        }
    }
    let mut check = Expanded {
        tcx,
        diags: Diagnostics::default(),
    };
    let krate = tcx.resolver_for_lowering().1.borrow();
    check.visit_crate(&krate);
    crate::walk::check_trait_surface(tcx, &krate, &mut check.diags);
    crate::walk::check_habit_surface(tcx, &krate, &mut check.diags);
    let unused_imports = crate::walk::check_module_surface(tcx, &krate, &mut check.diags, insertions);
    // ADMIT-1: the std rustc injects is Tide's, whose identity the rules key on
    // (`Crates::tide_std`). Under `#![no_std]` rustc injects `core`, so no crate is Tide's std and
    // the `Pure` rules are off, and an `extern crate std` loads it only as an ordinary crate.
    // `#![no_core]` is an unstable feature (ADMIT-2).
    if tide_std {
        for attr in krate.attrs.iter().filter(|a| a.has_name(sym::no_std)) {
            let message = "`#![no_std]` is not admitted: Tide's std is every crate's std";
            check
                .diags
                .report(tcx, attr.span, Code::Td0520, message.into(), None);
        }
    }
    // ROOT-1: every binary starts at the `main` that `#[tide::main]` generates. rustc reads
    // `no_main` only at the crate root; elsewhere it is inert.
    if tide_std && tcx.crate_types().contains(&CrateType::Executable) {
        for attr in krate.attrs.iter().filter(|a| a.has_name(sym::no_main)) {
            let message = "`#![no_main]` leaves the binary without `#[tide::main]`'s entry point";
            check
                .diags
                .report(tcx, attr.span, Code::Td0512, message.into(), None);
        }
    }
    let before = tcx.dcx().err_count();
    check.diags.emit(tcx);
    (tcx.dcx().err_count() - before, unused_imports)
}

/// The clean process's whole environment: rustc's crash hook otherwise writes a report to the
/// current directory, which may be in the candidate's tree (RUN-2); the message still goes to
/// stderr.
const ICE_VAR: &str = "RUSTC_ICE";
const ICE_OFF: &str = "0";

pub fn run() -> Result<ExitCode, String> {
    let args: Vec<String> = std::env::args_os()
        .skip(1)
        .map(|arg| {
            arg.into_string()
                .map_err(|_| "an argument is not valid UTF-8".to_string())
        })
        .collect::<Result<_, _>>()?;
    if args == ["--tide-print-codes"] {
        for code in Code::ALL {
            println!("{}", code.as_str());
        }
        return Ok(ExitCode::SUCCESS);
    }
    if let [flag, base, head] = args.as_slice()
        && flag == "--tide-diff"
    {
        print!(
            "{}",
            crate::interface::diff(Path::new(base), Path::new(head))?
        );
        return Ok(ExitCode::SUCCESS);
    }
    if args == ["--help"] {
        println!("tide-admit [fixed build options] SOURCE.rs
  --test --emit=link --tide-std PATH --tide-dep-dir DIR --out-dir DIR
  --tide-run-test NAME [--seed U64]  run one exact recorded test (excludes const tests and test main)
  --replay MANIFEST                 identity-checked replay of one recorded evaluation
  --tide-eval-timeout SECS          a simulation evaluation's wall-clock limit (default 300)
Replay is a partial precursor to tide sim, for reviewed code with modeled effects.
Replay accepts no configuration overrides and has no force option. Artifacts: OUT/target/sim.");
        return Ok(ExitCode::SUCCESS);
    }
    let mut invocation = Invocation::parse(&args)?;
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    // Even direct invocation of the internal compiler mode gets a clean process.
    if std::env::vars_os().ne([(OsString::from(ICE_VAR), OsString::from(ICE_OFF))]) {
        let status = Command::new(exe)
            .args(&args)
            .env_clear()
            .env(ICE_VAR, ICE_OFF)
            .stdin(Stdio::null())
            .status()
            .map_err(|e| e.to_string())?;
        return Ok(ExitCode::from(status.code().unwrap_or(2) as u8));
    }
    // Only now, so no earlier panic gets a hook that writes reports.
    rustc_driver::install_ice_hook("the tide-admit driver (crates/tide-admit)", |_| ());
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let package = invocation.package_root(&cwd)?;
    invocation.out = destination(&invocation.out).map_err(|e| e.to_string())?;
    if invocation.out.starts_with(&package) {
        return Err("output and cache must be outside the candidate package".into());
    }
    if let Some(library) = invocation.test_library.take() {
        invocation.trusted_inputs(&package)?;
        return invocation.compile_with_library(&cwd, &package, library);
    }
    invocation.clear_outputs()?;
    let (source_root, source) = invocation.source_paths(&package)?;
    invocation.remap = vec![(source_root.clone(), "tide-source".into())];
    if reject_build_script(&package) {
        return Ok(ExitCode::FAILURE);
    }
    fs::create_dir_all(&invocation.out).map_err(|e| e.to_string())?;
    invocation.trusted_inputs(&package)?;
    let mut key = Vec::new();
    fingerprint(&exe, &mut key).map_err(|e| e.to_string())?;
    // The installed pinned toolchain is trusted and immutable; operator artifacts are hashed below.
    record(
        &mut key,
        b"pinned compiler",
        env!("TIDE_COMPILER_ID").as_bytes(),
    );
    if let Some(std) = &invocation.std {
        fingerprint(std, &mut key).map_err(|e| e.to_string())?;
    }
    if invocation.link {
        // Like the pinned sysroot, the linker's C toolchain and libraries are trusted and immutable.
        for tool in linker_tools() {
            fingerprint(&tool, &mut key).map_err(|e| e.to_string())?;
        }
    }
    for dep in &invocation.deps {
        fingerprint(dep, &mut key).map_err(|e| e.to_string())?;
    }
    for arg in invocation.rustc_args(&invocation.out, None) {
        record(&mut key, b"argument", arg.as_bytes());
    }
    record(
        &mut key,
        b"unsafe grant",
        &[u8::from(invocation.grants.unsafe_)],
    );
    record(
        &mut key,
        b"backstop only",
        &[u8::from(invocation.backstop_only)],
    );
    record(&mut key, b"test ice", &[u8::from(invocation.test_ice)]);
    let sources = source_snapshot(&source_root, &source, &mut key)?;
    let build = invocation.build_identity(&sources, None)?;
    let manifest = invocation.validate_replay(&build)?;
    let key: String = SourceFileHash::new_in_memory(SourceFileHashAlgorithm::Sha256, &key)
        .hash_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    let cache = invocation.out.join(".tide-cache");
    if cache
        .symlink_metadata()
        .is_ok_and(|metadata| metadata.file_type().is_symlink())
    {
        return Err("cache directory must not be a symbolic link".into());
    }
    fs::create_dir_all(&cache).map_err(|e| e.to_string())?;
    if let Some(stage) = &invocation.compile {
        if !stage
            .strip_prefix(&format!("{key}."))
            .is_some_and(|pid| !pid.is_empty() && pid.bytes().all(|c| c.is_ascii_digit()))
        {
            return Err("compiler snapshot changed or internal invocation is invalid".into());
        }
        let out = cache.join(stage);
        // A killed earlier run with this pid may have left its staging directory.
        let _ = fs::remove_dir_all(&out);
        fs::create_dir(&out).map_err(|e| e.to_string())?;
        let mut loaded = BTreeSet::new();
        let status = invocation.compile(&out, sources, None, true, &mut loaded);
        if status == ExitCode::SUCCESS && invocation.link { invocation.publish_receipt(&out, &build, &loaded)?; }
        return Ok(status);
    }
    // Granted assembly and native links can read inputs the source snapshot cannot key.
    let cacheable = !invocation.link || !invocation.grants.unsafe_;
    let mut entry = cache.join(&key);
    if !cacheable || !entry.is_dir() {
        let stage = format!("{key}.{}", std::process::id());
        let output = Command::new(exe)
            .args(&args)
            .arg(format!("--tide-compile={stage}"))
            .env_clear()
            .env(ICE_VAR, ICE_OFF)
            .stdin(Stdio::null())
            .output()
            .map_err(|e| e.to_string())?;
        let staging = cache.join(stage);
        if !output.status.success() {
            let _ = fs::remove_dir_all(staging);
            io::stdout()
                .write_all(&output.stdout)
                .map_err(|e| e.to_string())?;
            io::stderr()
                .write_all(&output.stderr)
                .map_err(|e| e.to_string())?;
            return Ok(ExitCode::from(output.status.code().unwrap_or(2) as u8));
        }
        fs::write(staging.join("stdout"), &output.stdout).map_err(|e| e.to_string())?;
        fs::write(staging.join("stderr"), &output.stderr).map_err(|e| e.to_string())?;
        if cacheable {
            match fs::rename(&staging, &entry) {
                Ok(()) => (),
                Err(_) if entry.is_dir() => {
                    let _ = fs::remove_dir_all(&staging);
                }
                Err(e) => return Err(e.to_string()),
            }
        } else {
            entry = staging;
        }
    }
    // Replay checks and runs private artifacts without replacing any published outputs.
    if manifest.is_some() { return invocation.run_test_process(manifest, &build, Some(&entry)); }
    for file in fs::read_dir(&entry).map_err(|e| e.to_string())? {
        let file = file.map_err(|e| e.to_string())?.path();
        let executable = invocation.link && file.file_name().is_some_and(|n| n == "executable");
        if file.file_name().is_some_and(|n| n == "test-config.json") { continue; }
        if executable
            || file
                .extension()
                .is_some_and(|ext| ext == "rmeta" || ext == "json")
        {
            let dest = if executable {
                invocation.executable()
            } else {
                if file.file_name().is_some_and(|n| n == "build.json") {
                    invocation.receipt_path()
                } else {
                    invocation.out.join(file.file_name().unwrap())
                }
            };
            let tmp = invocation
                .out
                .join(format!(".artifact-{}", std::process::id()));
            fs::copy(&file, &tmp).map_err(|e| e.to_string())?;
            fs::rename(tmp, dest).map_err(|e| e.to_string())?;
        }
    }
    io::stdout()
        .write_all(&fs::read(entry.join("stdout")).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    io::stderr()
        .write_all(&fs::read(entry.join("stderr")).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    if !cacheable {
        fs::remove_dir_all(entry).map_err(|e| e.to_string())?;
    }
    invocation.run_test_process(None, &build, None)
}
