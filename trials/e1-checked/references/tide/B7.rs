use std::{
    collections::{BTreeMap, HashSet},
    fs::Fs,
    io,
};
#[derive(Debug, PartialEq)]
pub enum PlanError {
    DuplicateSource(String),
    DuplicateTarget(String),
}
#[pure]
pub fn plan(moves: &[(&str, &str)]) -> Result<Vec<(String, String)>, PlanError> {
    let (mut sources, mut targets) = (HashSet::new(), HashSet::new());
    let mut pending = BTreeMap::new();
    for &(a, b) in moves {
        if !sources.insert(a) {
            return Err(PlanError::DuplicateSource(a.into()));
        }
        if !targets.insert(b) {
            return Err(PlanError::DuplicateTarget(b.into()));
        }
        if a != b {
            pending.insert(a.to_string(), b.to_string());
        }
    }
    let mut out = Vec::new();
    while !pending.is_empty() {
        if let Some(a) = pending
            .iter()
            .find(|(_, b)| !pending.contains_key(*b))
            .map(|(a, _)| a.clone())
        {
            let b = pending.remove(&a).unwrap();
            out.push((a, b));
        } else {
            let a = pending.first_key_value().unwrap().0.clone();
            let b = pending.remove(&a).unwrap();
            let mut temp = format!("{a}.tmp");
            while sources.contains(temp.as_str()) || targets.contains(temp.as_str()) {
                temp.push_str(".tmp");
            }
            out.push((a, temp.clone()));
            pending.insert(temp, b);
        }
    }
    Ok(out)
}
pub fn apply(fs: &Fs, steps: &[(String, String)]) -> io::Result<()> {
    let mut available = HashSet::new();
    let mut removed = HashSet::new();
    for (a, b) in steps {
        if !available.contains(a) && (removed.contains(a) || !fs.exists(a)?) {
            return Err(io::ErrorKind::NotFound.into());
        }
        available.remove(a);
        removed.insert(a);
        available.insert(b.clone());
        removed.remove(b);
    }
    for (a, b) in steps {
        fs.rename(a, b)?;
    }
    Ok(())
}
