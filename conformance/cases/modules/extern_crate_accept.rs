//@ rules: REF-0912
use std::collections::BTreeMap;
pub fn empty() -> BTreeMap<u8, u8> { BTreeMap::new() }
