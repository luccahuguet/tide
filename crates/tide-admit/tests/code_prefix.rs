//! Tide's diagnostic codes are spelled `TD####`. They were `FX####` (from the superseded name
//! Ferox) until bead fx-qlek.12; this guard fails if the old spelling returns to a live file.
//! It walks the source tree itself, so it also runs from a source archive without `.git`.

use std::path::Path;

/// Historical records, kept unchanged as evidence of what was reviewed, decided, or run. Each is
/// a prefix of a `/`-separated path relative to the repository root, so `trials/e1` covers every
/// E1 copy (`trials/e1-checked/` too) and later trials stay guarded.
const KEPT: &[&str] = &["archive/", "research/", "reviews/", "spikes/", "trials/e1"];

/// Whether `text` contains `FX` followed by four ASCII digits.
fn has_legacy_code(text: &[u8]) -> bool {
    text.windows(6)
        .any(|w| w.starts_with(b"FX") && w[2..].iter().all(u8::is_ascii_digit))
}

/// Checks every regular file under `dir`, skipping kept paths, build output (`target/`), and
/// dot-directories (`.git`, the `.beads` ledger, `.worktrees`, tool state). Symbolic links are
/// not followed. Returns the number of files read.
fn scan(root: &Path, dir: &Path, found: &mut Vec<String>) -> usize {
    let mut checked = 0;
    let entries = std::fs::read_dir(dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
    for entry in entries {
        let entry = entry.unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
        let path = entry.path();
        let rel = path.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/");
        let name = entry.file_name();
        let kind = entry.file_type().unwrap_or_else(|e| panic!("{rel}: {e}"));
        if KEPT.iter().any(|kept| rel.starts_with(kept)) {
            continue;
        }
        if kind.is_dir() {
            if name != "target" && !name.to_string_lossy().starts_with('.') {
                checked += scan(root, &path, found);
            }
        } else if kind.is_file() {
            let text = std::fs::read(&path).unwrap_or_else(|e| panic!("{rel}: {e}"));
            checked += 1;
            for (n, line) in text.split(|&b| b == b'\n').enumerate() {
                if has_legacy_code(line) {
                    found.push(format!("{rel}:{}", n + 1));
                }
            }
        }
    }
    checked
}

#[test]
fn live_files_use_the_td_prefix() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let root = root.canonicalize().unwrap_or_else(|e| panic!("{}: {e}", root.display()));
    let mut found = Vec::new();
    let checked = scan(&root, &root, &mut found);
    assert!(checked > 100, "the guard read only {checked} live files");
    assert!(found.is_empty(), "FX-prefixed codes in live files (use TD):\n{}", found.join("\n"));
}

#[test]
fn the_pattern_matches_only_fx_and_four_digits() {
    // Built at run time, so this file does not trip the guard itself.
    let legacy = format!("error: {}0206: x", "FX");
    assert!(has_legacy_code(legacy.as_bytes()));
    assert!(!has_legacy_code(b"TD0206 FX020 FX#### Fx0206 fx-qlek.12 FXB1"));
}
