//@ rules: MOVE-2
// Reassigning a moved-out place makes it usable again; a partial move leaves the other
// fields usable; a move on one branch does not affect the other.
pub struct Pair {
    pub a: String,
    pub b: String,
}

pub fn reassign(mut s: String) -> usize {
    let t = s;
    s = String::new();
    s.len() + t.len()
}

pub fn partial(p: Pair) -> usize {
    let a = p.a;
    a.len() + p.b.len()
}

pub fn branch(s: String, c: bool) -> String {
    if c {
        drop(s);
        String::new()
    } else {
        s
    }
}
