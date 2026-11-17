//@ rules: PAR-4
// An Ord payload can still carry authority; par_sort must enforce Pure through pure.rs.
use std::cmp::Ordering;
use std::time::Clock;

pub struct Key {
    rank: usize,
    clock: Clock,
}
impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        self.rank == other.rank
    }
}
impl Eq for Key {}
impl PartialOrd for Key {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Key {
    fn cmp(&self, other: &Self) -> Ordering {
        self.rank.cmp(&other.rank)
    }
}
pub fn sort(xs: &mut [Key]) {
    xs.par_sort(); //~ TD0612
}
