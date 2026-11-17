//@ rules: LOCK-4
// A `with` closure cannot capture a `Formatter`, directly or through the builder that
// `debug_struct` returns, which holds it (02 §9): copy the data out, then format.
use std::fmt;
use std::sync::Shared;

pub struct Hits {
    count: Shared<u64>,
}

impl fmt::Debug for Hits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = f.debug_struct("Hits");
        let _ = self.count.with(|n| { out.field("count", n); }); //~ TD0603
        out.finish()
    }
}

impl fmt::Display for Hits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let _ = self.count.with(|n| write!(f, "{n}")); //~ TD0603
        let n = self.count.with(|n| *n).map_err(|_| fmt::Error)?;
        write!(f, "{n}")
    }
}
