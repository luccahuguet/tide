use std::{
    collections::HashSet,
    fs::{Fs, Mode},
    io,
    path::Path,
};
#[derive(Debug)]
pub enum ExportError {
    BadName(String),
    Duplicate(String),
    Io(io::Error),
}
pub fn export(fs: &Fs, reports: &[(&str, &str)]) -> Result<usize, ExportError> {
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
    fs.create_dir_all("out").map_err(ExportError::Io)?;
    let out = fs.dir("out", Mode::ReadWrite).map_err(ExportError::Io)?;
    for &(name, body) in reports {
        out.create_dir_all(Path::new(name).parent().unwrap())
            .map_err(ExportError::Io)?;
        // Replace the entry rather than writing through an existing hard link.
        match out.remove_file(name) {
            Ok(()) => (),
            Err(e) if e.kind() == io::ErrorKind::NotFound => (),
            Err(e) => return Err(ExportError::Io(e)),
        }
        out.write(name, body).map_err(ExportError::Io)?;
    }
    Ok(reports.len())
}
