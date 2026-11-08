use std::{
    sync::atomic::{AtomicBool, Ordering},
    thread,
};
pub fn find_any(hays: &[String], needle: &str) -> Option<(usize, usize)> {
    assert!(!needle.is_empty());
    let done = AtomicBool::new(false);
    thread::scope(|s| {
        let hs: Vec<_> = hays
            .iter()
            .enumerate()
            .map(|(i, hay)| {
                let done = &done;
                s.spawn(move || {
                    for (pos, w) in hay.as_bytes().windows(needle.len()).enumerate() {
                        if pos % 4096 == 0 && done.load(Ordering::SeqCst) {
                            return None;
                        }
                        if w == needle.as_bytes() {
                            done.store(true, Ordering::SeqCst);
                            return Some((i, pos));
                        }
                    }
                    None
                })
            })
            .collect();
        hs.into_iter().filter_map(|h| h.join().unwrap()).next()
    })
}
