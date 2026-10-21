//@ rules: LOAN-1, LOAN-4
// A reborrow through `&mut` references carries each one's origin, so it cannot
// outlive the outer borrow. A `&mut T` passed where no `&mut T` is expected moves.
pub fn through_mut<'a, 'b>(r: &'a mut &'b mut String) -> &'b mut String {
    &mut **r //~ error
}

fn take<T>(_: T) {}

pub fn generic(v: &mut Vec<u32>) {
    take(v);
    v.push(1); //~ E0382
}

pub fn unannotated(v: &mut Vec<u32>) {
    let w = v;
    w.push(2);
    v.push(3); //~ E0382
}
