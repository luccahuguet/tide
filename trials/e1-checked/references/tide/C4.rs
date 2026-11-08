use std::{
    collections::BTreeMap,
    sync::Shared,
    task::{TaskError, Tasks},
};
#[derive(Debug, PartialEq)]
pub enum ReserveError {
    UnknownItem(String),
    Insufficient(String),
}
pub struct Warehouse {
    items: BTreeMap<String, Shared<u32>>,
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
    pub fn new(tasks: &Tasks, stock: &[(&str, u32)]) -> Self {
        Self {
            items: stock
                .iter()
                .map(|&(k, v)| (k.into(), Shared::new(v, tasks)))
                .collect(),
        }
    }
    pub fn reserve(&self, order: &[(&str, u32)]) -> Result<(), ReserveError> {
        let lines = aggregate(order);
        let names: Vec<_> = self
            .items
            .keys()
            .filter(|k| lines.iter().any(|(n, _)| n == k))
            .map(String::as_str)
            .collect();
        let cells: Vec<_> = names.iter().map(|n| self.items[*n].clone()).collect();
        Shared::with_all(&cells, |stocks| {
            for &(name, n) in &lines {
                let i = names
                    .iter()
                    .position(|k| *k == name)
                    .ok_or_else(|| ReserveError::UnknownItem(name.into()))?;
                if u64::from(*stocks[i]) < n {
                    return Err(ReserveError::Insufficient(name.into()));
                }
            }
            for &(name, n) in &lines {
                let i = names.iter().position(|k| *k == name).unwrap();
                *stocks[i] -= u32::try_from(n).unwrap();
            }
            Ok(())
        })
        .unwrap()
    }
    pub fn stock(&self, item: &str) -> Option<u32> {
        Some(self.items.get(item)?.with(|n| *n).unwrap())
    }
}
pub fn run_orders(
    tasks: &Tasks,
    w: &Warehouse,
    orders: &[Vec<(&str, u32)>],
    workers: usize,
) -> Vec<Result<(), ReserveError>> {
    tasks
        .limit(workers)
        .scope(|s| -> Result<_, TaskError> {
            let hs: Vec<_> = orders
                .iter()
                .map(|o| s.spawn(move || Ok(w.reserve(o))))
                .collect();
            hs.into_iter().map(|h| h.join()).collect()
        })
        .unwrap()
}
