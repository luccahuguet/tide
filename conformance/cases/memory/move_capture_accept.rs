//@ rules: MOVE-4
// A closure captures each place path in the weakest mode its body needs, so the
// parent may still use what the closure does not conflict with.
pub struct Point {
    pub x: u32,
    pub y: u32,
}

pub fn disjoint(p: &mut Point) -> u32 {
    let mut bump = || p.x += 1;
    let y = p.y;
    bump();
    y
}

pub fn shared(v: Vec<u32>) -> usize {
    let len = || v.len();
    let n = v.len() + len();
    n + v.len()
}

pub fn by_move(v: Vec<u32>) -> usize {
    let f = move || v.len();
    f()
}
