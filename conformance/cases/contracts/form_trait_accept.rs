//@ rules: FORM-3
// In v0, `#[invariants]` and loop `invariant!` are allowed on trait and trait-impl
// methods; a `requires` or `ensures` goes on an inherent method the impl calls.
pub trait Tally {
    fn total(&self) -> u64;

    #[invariants]
    fn zeros(&self, xs: &[u32]) -> usize {
        let mut z = 0;
        for i in 0..xs.len() {
            invariant!(z <= i);
            if xs[i] == 0 {
                z += 1;
            }
        }
        z
    }
}

pub struct Bag {
    pub items: Vec<u32>,
}

impl Bag {
    #[ensures(self.items.is_empty() ==> result == 0)]
    pub fn sum(&self) -> u64 {
        self.items.iter().map(|x| *x as u64).sum()
    }
}

impl Tally for Bag {
    fn total(&self) -> u64 {
        self.sum()
    }

    #[invariants]
    fn zeros(&self, xs: &[u32]) -> usize {
        let mut z = 0;
        let mut i = 0;
        while i < xs.len() {
            invariant!(z <= i);
            z += (xs[i] == 0) as usize;
            i += 1;
        }
        z
    }
}
