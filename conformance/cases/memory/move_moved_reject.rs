//@ rules: MOVE-2
// Reading or borrowing a moved-out place, wholly, partly, or on one path, is an error.
pub struct Pair {
    pub a: String,
    pub b: String,
}

pub fn read_after(s: String) -> usize {
    let t = s;
    drop(t);
    s.len() //~ E0382
}

pub fn borrow_after(s: String) {
    let _t = s;
    let _r = &s; //~ E0382
}

pub fn whole_after_partial(p: Pair) {
    let _a = p.a;
    let _q = &p; //~ E0382
}

pub fn maybe_moved(s: String, c: bool) -> usize {
    if c {
        drop(s);
    }
    s.len() //~ E0382
}
