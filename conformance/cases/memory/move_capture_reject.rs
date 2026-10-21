//@ rules: MOVE-4
// A capture conflicts with the parent's use of the same place: a mutating
// closure borrows `p.x` mutably, and `move` takes `v` by value.
pub struct Point {
    pub x: u32,
    pub y: u32,
}

pub fn mutating(p: &mut Point) -> u32 {
    let mut bump = || p.x += 1;
    let x = p.x; //~ E0503
    bump();
    x
}

pub fn pushing(mut v: Vec<u32>) -> usize {
    let mut add = || v.push(1);
    let n = v.len(); //~ E0502
    add();
    n
}

pub fn moved(v: Vec<u32>) -> usize {
    let f = move || v.len();
    f() + v.len() //~ E0382
}
