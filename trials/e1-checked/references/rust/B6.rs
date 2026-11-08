use std::{collections::HashSet, fs, io, path::Path};
#[derive(Debug)]
pub enum ExportError {
    BadName(String),
    Duplicate(String),
    Io(io::Error),
}
fn confined(dir: &Path, components: &[&str]) -> io::Result<()> {
    let mut p = dir.to_path_buf();
    for c in components {
        p.push(c);
        match fs::symlink_metadata(&p) {
            Ok(m) if m.file_type().is_symlink() => {
                return Err(io::ErrorKind::PermissionDenied.into());
            }
            Ok(_) => (),
            Err(e) if e.kind() == io::ErrorKind::NotFound => (),
            Err(e) => return Err(e),
        }
    }
    Ok(())
}
pub fn export(root: &Path, reports: &[(&str, &str)]) -> Result<usize, ExportError> {
    let mut seen = HashSet::new();
    for &(name, _) in reports {
        if name.contains('\\')
            || name
                .split('/')
                .any(|c| c.is_empty() || c == "." || c == "..")
        {
            return Err(ExportError::BadName(name.into()));
        }
        if !seen.insert(name) {
            return Err(ExportError::Duplicate(name.into()));
        }
    }
    confined(root, &["out"]).map_err(ExportError::Io)?;
    let out = root.join("out");
    fs::create_dir_all(&out).map_err(ExportError::Io)?;
    for &(name, body) in reports {
        confined(&out, &name.split('/').collect::<Vec<_>>()).map_err(ExportError::Io)?;
        let path = out.join(name);
        fs::create_dir_all(path.parent().unwrap()).map_err(ExportError::Io)?;
        // Replacing the entry also avoids writing through an existing hard link.
        match fs::remove_file(&path) {
            Ok(()) => (),
            Err(e) if e.kind() == io::ErrorKind::NotFound => (),
            Err(e) => return Err(ExportError::Io(e)),
        }
        fs::write(path, body).map_err(ExportError::Io)?;
    }
    Ok(reports.len())
}
