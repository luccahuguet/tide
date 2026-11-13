//@ rules: CLAUSE-4
// old snapshots, independent lifetimes, and right-hand outlives bounds are allowed.
#[ensures(old(*x) <= *y)]
pub fn snapshot<'a>(x: &'a mut u32, y: &'a u32) { *x = *y; }

#[ensures(*x == *y)]
pub fn independent<'a, 'b>(x: &'a mut u32, y: &'b u32) { *x = *y; }

#[ensures(*x > 0)]
pub fn bound<'a, 'b: 'a>(x: &'a mut u32, _tag: std::marker::PhantomData<&'b ()>) {
    *x = 1;
}

#[ensures(*x > 0)]
pub fn precise<'a>(x: &'a mut u32) -> impl Sized + use<> { *x = 1; }

#[ensures((|x: u32| x == 1)(1))]
pub fn shadowed<'a>(x: &'a mut u32, _y: &'a u32) { *x = 1; }

// The pointee only implies 'b: 'a, with the outer lifetime on the right.
#[ensures(**x == 0)]
pub fn implied<'a, 'b>(x: &'a mut &'b u32) { let _ = x; }
