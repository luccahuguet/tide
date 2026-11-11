//@ rules: CLAUSE-4
// A shared reference snapshots its referent; cloning only &Token is insufficient.
pub struct Token(pub u32);
#[ensures(old(t).0 == result)] //~ E0277
pub fn snapshot(t: &Token) -> u32 { t.0 }
