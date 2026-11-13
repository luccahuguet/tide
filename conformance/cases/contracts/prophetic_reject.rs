//@ rules: CLAUSE-4
// Shared input lifetimes, output captures, and left-hand outlives bounds all escape.
#[ensures(*x == *y)] //~ FX0803
pub fn shared<'a>(x: &'a mut u32, y: &'a u32) { *x = *y; }

#[ensures(*x > 0)] //~ FX0803
pub fn bound<'a: 'b, 'b>(x: &'a mut u32, _tag: std::marker::PhantomData<&'b ()>) {
    *x = 1;
}

#[ensures(*x > 0)] //~ FX0803
pub fn captured<'a>(x: &'a mut u32) -> impl Sized + use<'a> { *x = 1; }

pub struct Counter(u32);
impl Counter {
    #[ensures(self.0 == *other)] //~ FX0803
    pub fn shared<'a>(&'a mut self, other: &'a u32) { self.0 = *other; }
}

#[ensures(*x > 0)] //~ FX0803
pub fn nested<'a>(x: &'a mut u32, _reader: fn(&'a u32)) { *x = 1; }

// Well-formedness of the nested reference implies 'a: 'b.
#[ensures(***x == 0)] //~ FX0803
pub fn implied<'a, 'b>(x: &'a mut &'b &'a u32) { let _ = x; }
