use std::task::{TaskError, Tasks};

pub fn find_any(tasks: &Tasks, hays: &[String], needle: &str) -> Option<(usize, usize)> {
    assert!(!needle.is_empty());
    tasks
        .scope(|s| -> Result<_, TaskError> {
            let hs: Vec<_> = hays
                .iter()
                .enumerate()
                .map(|(i, hay)| {
                    s.spawn(move || {
                        for (pos, w) in hay.as_bytes().windows(needle.len()).enumerate() {
                            if pos % 4096 == 0 && tasks.checkpoint().is_err() {
                                return Ok(None);
                            }
                            if w == needle.as_bytes() {
                                s.cancel();
                                return Ok(Some((i, pos)));
                            }
                        }
                        Ok(None)
                    })
                })
                .collect();
            let mut result = None;
            for h in hs {
                let found = h.join()?;
                if result.is_none() {
                    result = found;
                }
            }
            Ok(result)
        })
        .unwrap()
}
