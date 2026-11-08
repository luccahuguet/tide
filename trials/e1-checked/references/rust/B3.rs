use std::{fs, io, path::Path};
#[derive(Debug, PartialEq)]
pub struct Summary {
    pub files: u64,
    pub dirs: u64,
    pub bytes: u64,
    pub largest: Option<(String, u64)>,
}
fn walk(root: &Path, dir: &Path, files: &mut Vec<(String, u64)>, dirs: &mut u64) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let e = entry?;
        let meta = e.metadata()?;
        if meta.is_dir() {
            *dirs += 1;
            walk(root, &e.path(), files, dirs)?;
        } else if meta.is_file() {
            files.push((
                e.path()
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/"),
                meta.len(),
            ));
        }
    }
    Ok(())
}
fn listing(dir: &Path) -> io::Result<(Vec<(String, u64)>, u64)> {
    let mut files = Vec::new();
    let mut dirs = 0;
    walk(dir, dir, &mut files, &mut dirs)?;
    files.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    Ok((files, dirs))
}
pub fn summarize(dir: &Path) -> io::Result<Summary> {
    let (files, dirs) = listing(dir)?;
    Ok(Summary {
        files: u64::try_from(files.len()).unwrap(),
        dirs,
        bytes: files.iter().map(|f| f.1).sum(),
        largest: files.first().cloned(),
    })
}
pub fn top_n(dir: &Path, n: usize) -> io::Result<Vec<(String, u64)>> {
    let (mut files, _) = listing(dir)?;
    files.truncate(n);
    Ok(files)
}
