// Task 10: tally -- src/lib.rs
//
// Word counts: authority-free and data-parallel in `freq_all`, shared through
// a `Shared` cell across tasks in `Tally`.

use std::collections::BTreeMap;
use std::sync::{LockError, Shared};
use std::task::{TaskError, Tasks};

pub type Counts = BTreeMap<String, u64>;

#[derive(Debug, Error, PartialEq)]
pub enum TallyError {
    #[error("count for {0:?} overflowed")]
    Overflow(String),
    #[error(transparent)]
    Lock(#[from] LockError),
    #[error(transparent)]
    Task(#[from] TaskError),
}

/// Counts of the lowercased ASCII words of `text`; every character that is
/// not ASCII alphanumeric separates words.
pub fn freq(text: &str) -> Counts {
    let mut counts = Counts::new();
    for word in text.split(|c: char| !c.is_ascii_alphanumeric()).filter(|w| !w.is_empty()) {
        *counts.entry(word.to_ascii_lowercase()).or_insert(0) += 1;
    }
    counts
}

/// Every document's counts, computed in parallel and merged in order.
pub fn freq_all(docs: &[String]) -> Counts {
    let per_doc: Vec<Counts> = docs.par_map(|doc| freq(doc));
    let mut total = Counts::new();
    for counts in per_doc {
        merge(&mut total, counts);
    }
    total
}

fn merge(into: &mut Counts, from: Counts) {
    for (word, n) in from {
        *into.entry(word).or_insert(0) += n;
    }
}

/// A word counter that several tasks update concurrently. Each method is one
/// atomic step on the cell.
pub struct Tally {
    counts: Shared<Counts>,
}

impl Tally {
    pub fn new(tasks: &Tasks) -> Tally {
        Tally { counts: Shared::new(Counts::new(), tasks) }
    }

    pub fn add(&self, word: &str, n: u64) -> Result<(), TallyError> {
        let word = word.to_string();
        self.counts.with(move |m| -> Result<(), TallyError> {
            let total = m.get(&word).copied().unwrap_or(0).checked_add(n);
            let total = total.ok_or_else(|| TallyError::Overflow(word.clone()))?;
            m.insert(word, total); // the only write, after the check
            Ok(())
        })?
    }

    pub fn snapshot(&self) -> Result<Counts, LockError> {
        self.counts.with(|m| m.clone())
    }

    /// Moves all of `self`'s counts into `other` in one atomic step; merging
    /// a tally into itself is a no-op.
    pub fn merge_into(&self, other: &Tally) -> Result<(), TallyError> {
        if Shared::same(&self.counts, &other.counts) {
            return Ok(());
        }
        Shared::with2(&self.counts, &other.counts, |from, to| -> Result<(), TallyError> {
            let mut merged = Vec::with_capacity(from.len());
            for (word, &n) in from.iter() {
                let total = to.get(word).copied().unwrap_or(0).checked_add(n);
                merged.push((word.clone(), total.ok_or_else(|| TallyError::Overflow(word.clone()))?));
            }
            // Every check passed; nothing below can fail or trap.
            for (word, total) in merged {
                to.insert(word, total);
            }
            from.clear();
            Ok(())
        })?
    }
}

/// One child per document adds that document's counts to a shared `Tally`;
/// returns the final snapshot.
pub fn count_concurrently(tasks: &Tasks, docs: &[String]) -> Result<Counts, TallyError> {
    let tally = Tally::new(tasks);
    tasks.scope(|s| -> Result<(), TallyError> {
        let tally = &tally;
        let handles: Vec<_> = docs
            .iter()
            .map(|doc| {
                s.spawn(move || {
                    for (word, n) in freq(doc) {
                        tally.add(&word, n)?;
                    }
                    Ok(())
                })
            })
            .collect();
        for h in handles {
            h.join()?;
        }
        Ok(())
    })?;
    Ok(tally.snapshot()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sim::SimRoot;

    fn docs() -> Vec<String> {
        vec![
            "The cat, the hat.".to_string(),
            "".to_string(),
            "HAT-trick: x2 x2 x2".to_string(),
            "the end".to_string(),
        ]
    }

    #[test]
    fn freq_lowercases_and_splits_on_non_alphanumerics() {
        let want: Counts = Counts::from([
            ("cat".to_string(), 1),
            ("hat".to_string(), 1),
            ("the".to_string(), 2),
        ]);
        assert_eq!(freq("The cat, the hat."), want);
        assert_eq!(freq(" ,; "), Counts::new());
    }

    #[test]
    fn freq_all_equals_the_sequential_merge(docs: Vec<String>) {
        let mut want = Counts::new();
        for d in &docs {
            merge(&mut want, freq(d));
        }
        assert_eq!(freq_all(&docs), want);
    }

    #[test]
    fn freq_all_on_fixed_documents() {
        let got = freq_all(&docs());
        assert_eq!(got.get("the"), Some(&3));
        assert_eq!(got.get("hat"), Some(&2));
        assert_eq!(got.get("x2"), Some(&3));
        assert_eq!(got.get("trick"), Some(&1));
    }

    #[sim_test]
    fn count_concurrently_gives_the_same_totals_on_every_seed(sim: SimRoot) {
        let docs = docs();
        assert_eq!(count_concurrently(&sim.tasks, &docs), Ok(freq_all(&docs)));
    }

    #[test]
    fn merging_into_itself_is_a_no_op() {
        let sim = Root::sim(1);
        let t = Tally::new(&sim.tasks);
        t.add("a", 2).unwrap();
        t.merge_into(&t).unwrap();
        assert_eq!(t.snapshot(), Ok(Counts::from([("a".to_string(), 2)])));
    }

    #[test]
    fn merge_moves_counts() {
        let sim = Root::sim(2);
        let a = Tally::new(&sim.tasks);
        let b = Tally::new(&sim.tasks);
        a.add("x", 1).unwrap();
        a.add("y", 2).unwrap();
        b.add("y", 3).unwrap();
        a.merge_into(&b).unwrap();
        assert_eq!(a.snapshot(), Ok(Counts::new()));
        assert_eq!(b.snapshot(), Ok(Counts::from([("x".to_string(), 1), ("y".to_string(), 5)])));
    }

    #[test]
    fn overflow_is_an_error_and_leaves_counts_unchanged() {
        let sim = Root::sim(3);
        let a = Tally::new(&sim.tasks);
        let b = Tally::new(&sim.tasks);
        a.add("w", u64::MAX).unwrap();
        assert_eq!(a.add("w", 1), Err(TallyError::Overflow("w".to_string())));
        b.add("w", 1).unwrap();
        assert_eq!(a.merge_into(&b), Err(TallyError::Overflow("w".to_string())));
        assert_eq!(a.snapshot(), Ok(Counts::from([("w".to_string(), u64::MAX)])));
        assert_eq!(b.snapshot(), Ok(Counts::from([("w".to_string(), 1)])));
    }
}
