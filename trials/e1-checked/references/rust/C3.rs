use std::{sync::mpsc::sync_channel, thread};
#[derive(Debug, PartialEq)]
pub enum MergeError {
    Unsorted { input: usize, index: usize },
}
pub fn merge_streams(inputs: &[Vec<i64>], capacity: usize) -> Result<Vec<i64>, MergeError> {
    assert!(capacity > 0);
    thread::scope(|s| {
        let receivers: Vec<_> = inputs
            .iter()
            .enumerate()
            .map(|(input, values)| {
                let (tx, rx) = sync_channel(capacity);
                s.spawn(move || {
                    for (index, &v) in values.iter().enumerate() {
                        if index > 0 && v < values[index - 1] {
                            let _ = tx.send(Err(MergeError::Unsorted { input, index }));
                            return;
                        }
                        if tx.send(Ok(v)).is_err() {
                            return;
                        }
                    }
                });
                rx
            })
            .collect();
        let mut error = None;
        let mut take = |i: usize| match receivers[i].recv() {
            Ok(Ok(v)) => Some(v),
            Ok(Err(e)) => {
                if error.as_ref().is_none_or(|old| match (old, &e) {
                    (
                        MergeError::Unsorted { input: a, .. },
                        MergeError::Unsorted { input: b, .. },
                    ) => b < a,
                }) {
                    error = Some(e);
                }
                None
            }
            Err(_) => None,
        };
        let mut heads: Vec<_> = (0..receivers.len()).map(&mut take).collect();
        let mut out = Vec::new();
        while let Some(i) = (0..heads.len())
            .filter(|&i| heads[i].is_some())
            .min_by_key(|&i| heads[i])
        {
            out.push(heads[i].unwrap());
            heads[i] = take(i);
        }
        match error {
            Some(e) => Err(e),
            None => Ok(out),
        }
    })
}
