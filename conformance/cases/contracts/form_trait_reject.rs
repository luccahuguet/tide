//@ rules: FORM-3
// In v0 a `requires` or `ensures` on a trait method or a trait-impl method is
// FX0802 (v1 refines trait contracts). rustc accepts the D18 spike's lowering of a
// method with a body; a bodiless trait method does not parse as a function.
pub trait Stack {
    #[requires(n > 0)] fn reserve(&mut self, n: usize); //~ error FX0802

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
