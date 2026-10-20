// Task 7: backup -- src/lib.rs
//
// Durable, atomic-per-file copy. `backup` attenuates `fs` at once: the
// copying code holds only a read-only `Fs` below `src` and a read-write `Fs`
// below `dst`. The full `fs` is used for nothing but creating `dst` (and its
// missing ancestors) and syncing their entries.

use std::fs::{Fs, Mode};
use std::io;
use std::path::Path;

/// Copies every regular file directly inside `src` whose name ends in
/// `suffix` into `dst` (created if missing). When it returns `Ok`, every copy
/// and its directory entry are durable; a crash at any point leaves each
/// replaced file with either its old or its new contents. Returns the copied
/// names, sorted.
pub fn backup(fs: &Fs, src: &str, dst: &str, suffix: &str) -> io::Result<Vec<String>> {
    let from = fs.dir(src, Mode::Read)?; // fails (NotFound) before dst is touched
    create_dir_durably(fs, Path::new(dst))?;
    let to = fs.dir(dst, Mode::ReadWrite)?;
    copy_matching(&from, &to, suffix)
}

/// Creates `dir` and its missing ancestors, then syncs each new entry into
/// its parent, so `dir` itself survives a crash.
fn create_dir_durably(fs: &Fs, dir: &Path) -> io::Result<()> {
    let missing: Vec<&Path> = dir
        .ancestors()
        .take_while(|p| !p.as_os_str().is_empty() && !fs.exists(p))
        .collect();
    fs.create_dir_all(dir)?;
    for p in missing {
        let parent = p.parent().filter(|q| !q.as_os_str().is_empty()).unwrap_or(Path::new("."));
        fs.sync_dir(parent)?;
    }
    Ok(())
}

/// Reads only below `from`, writes only below `to`.
fn copy_matching(from: &Fs, to: &Fs, suffix: &str) -> io::Result<Vec<String>> {
    let mut names = Vec::new();
    for entry in from.read_dir(".")? {
        let entry = entry?;
        // A name that is not UTF-8 cannot be returned as a String; skip it.
        let Ok(name) = entry.file_name().into_string() else { continue };
        if name.ends_with(suffix) && from.metadata(&name)?.is_file() {
            names.push(name);
        }
    }
    names.sort();
    for name in &names {
        let data = from.read(name)?;
        // Write a synced temporary, then rename it over the target: the
        // target always names either the old file or the complete new one.
        let mut tmp = format!("{name}.backup-tmp");
        while names.contains(&tmp) {
            tmp.push('~');
        }
        to.write_synced(&tmp, &data)?;
        to.rename(&tmp, name)?;
    }
    to.sync_dir(".")?; // make every rename durable
    Ok(names)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sim::{Fault, NodeRoot, SimRoot};
    use std::time::Duration;

    #[sim_test]
    fn copies_two_matching_files_and_skips_the_other(sim: SimRoot) {
        sim.world.file("/data/b.log", "bee");
        sim.world.file("/data/a.log", "ay");
        sim.world.file("/data/c.txt", "see");
        let got = backup(&sim.fs, "/data", "/backup/daily", ".log").unwrap();
        assert_eq!(got, vec!["a.log".to_string(), "b.log".to_string()]);
        assert_eq!(sim.fs.read_to_string("/backup/daily/a.log").unwrap(), "ay");
        assert_eq!(sim.fs.read_to_string("/backup/daily/b.log").unwrap(), "bee");
        assert!(!sim.fs.exists("/backup/daily/c.txt"));
        assert_eq!(sim.fs.read_to_string("/data/c.txt").unwrap(), "see");
    }

    #[sim_test]
    fn an_existing_file_is_replaced(sim: SimRoot) {
        sim.world.file("/data/a.log", "new");
        sim.world.file("/backup/a.log", "old");
        sim.world.file("/backup/keep.txt", "untouched");
        let got = backup(&sim.fs, "/data", "/backup", ".log").unwrap();
        assert_eq!(got, vec!["a.log".to_string()]);
        assert_eq!(sim.fs.read_to_string("/backup/a.log").unwrap(), "new");
        assert_eq!(sim.fs.read_to_string("/backup/keep.txt").unwrap(), "untouched");
        assert!(!sim.fs.exists("/backup/a.log.backup-tmp"));
    }

    #[sim_test]
    fn a_missing_src_is_an_error(sim: SimRoot) {
        let err = backup(&sim.fs, "/nope", "/backup", ".log").unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
        assert!(!sim.fs.exists("/backup"));
    }

    /// A crash anywhere leaves the old or the new contents, and once backup
    /// has returned Ok the new contents survive.
    #[sim_test(seeds = 200)]
    fn a_crash_leaves_old_or_new_contents(sim: SimRoot) {
        let node = sim.node("box");
        node.start(|r: NodeRoot| {
            r.fs.create_dir_all("/src")?;
            r.fs.create_dir_all("/dst")?;
            r.fs.sync_dir("/")?;
            r.fs.write_synced("/src/a.log", "new contents")?;
            r.fs.write_synced("/dst/a.log", "old")?;
            info!("setup done");
            let copied = backup(&r.fs, "/src", "/dst", ".log")?;
            info!("backup done: {}", copied.len());
            Ok::<(), io::Error>(())
        });
        sim.faults.at(Duration::from_millis(5), Fault::crash(&node));
        sim.run_for(Duration::from_secs(1));

        let records = sim.records();
        let said = |prefix: &str| records.iter().any(|r| r.node == "box" && r.message.starts_with(prefix));
        let disk = node.fs();
        if said("backup done") {
            assert_eq!(disk.read_to_string("/dst/a.log").unwrap(), "new contents");
        } else if said("setup done") {
            let got = disk.read_to_string("/dst/a.log").unwrap();
            assert!(got == "old" || got == "new contents", "mixed contents: {got:?}");
        }
    }
}
