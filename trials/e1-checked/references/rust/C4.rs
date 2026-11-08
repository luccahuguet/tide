use std::{collections::BTreeMap, sync::Mutex, thread};
#[derive(Debug, PartialEq)]
pub enum ReserveError {
    UnknownItem(String),
    Insufficient(String),
}
pub struct Warehouse {
    items: BTreeMap<String, Mutex<u32>>,
}
fn aggregate<'a>(order: &[(&'a str, u32)]) -> Vec<(&'a str, u64)> {
    let mut lines: Vec<(&str, u64)> = Vec::new();
    for &(name, n) in order {
        if let Some(line) = lines.iter_mut().find(|(k, _)| *k == name) {
            line.1 += u64::from(n);
        } else {
            lines.push((name, u64::from(n)));
        }
    }
    lines
}
impl Warehouse {
    pub fn new(stock: &[(&str, u32)]) -> Self {
        Self {
            items: stock
                .iter()
                .map(|&(k, v)| (k.into(), Mutex::new(v)))
                .collect(),
        }
    }
    pub fn reserve(&self, order: &[(&str, u32)]) -> Result<(), ReserveError> {
        let lines = aggregate(order);
        let mut held: BTreeMap<_, _> = self
            .items
            .iter()
            .filter(|(k, _)| lines.iter().any(|(n, _)| n == k))
            .map(|(k, v)| (k.as_str(), v.lock().unwrap()))
            .collect();
        for &(name, n) in &lines {
            match held.get(name) {
                None => return Err(ReserveError::UnknownItem(name.into())),
                Some(v) if u64::from(**v) < n => {
                    return Err(ReserveError::Insufficient(name.into()));
                }
                _ => (),
            }
        }
        for (name, n) in lines {
            **held.get_mut(name).unwrap() -= u32::try_from(n).unwrap();
        }
        Ok(())
    }
    pub fn stock(&self, item: &str) -> Option<u32> {
        Some(*self.items.get(item)?.lock().unwrap())
    }
}
pub fn run_orders(
    w: &Warehouse,
    orders: &[Vec<(&str, u32)>],
    workers: usize,
) -> Vec<Result<(), ReserveError>> {
    assert!(workers > 0);
    let mut out = Vec::new();
    for batch in orders.chunks(workers) {
        out.extend(thread::scope(|s| {
            let hs: Vec<_> = batch
                .iter()
                .map(|o| s.spawn(move || w.reserve(o)))
                .collect();
            hs.into_iter()
                .map(|h| h.join().unwrap())
                .collect::<Vec<_>>()
        }));
    }
    out
}
