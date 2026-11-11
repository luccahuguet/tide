//@ rules: FORM-3
// In v0 a `requires` or `ensures` on a trait method or a trait-impl method is
// FX0802 (v1 refines trait contracts). The production macro leaves a bodiless
// method to admission; rustc accepts these placements.
pub trait Stack {
    #[requires(n > 0)] fn reserve(&mut self, n: usize); //~ FX0802

    #[ensures(result <= 1)] //~ FX0802
    fn top(&self) -> usize {
        0
    }
}

pub trait Grow {
    fn grow(&mut self, n: usize);
}

pub struct Bag {
    pub items: Vec<u32>,
}

impl Grow for Bag {
    #[requires(n > 0)] //~ FX0802
    fn grow(&mut self, n: usize) {
        self.items.reserve(n);
    }
}
