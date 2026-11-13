//@ rules: REF-0913
use std::collections::{BTreeMap as Map, BTreeSet};
pub use std::collections::VecDeque;
pub fn empty() -> (Map<u8, u8>, BTreeSet<u8>) { (Map::new(), BTreeSet::new()) }
