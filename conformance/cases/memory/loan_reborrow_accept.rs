//@ rules: LOAN-1, LOAN-4
// A reborrow through a shared reference carries only that reference's origin, so
// `rest` outlives the `&mut self` borrow. Where a `&mut T` is expected, a `&mut T`
// is reborrowed rather than moved.
pub struct Lexer<'a> {
    src: &'a str,
    pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn rest(&mut self) -> &'a str {
        &self.src[self.pos..]
    }
}

pub fn both<'a>(lx: &mut Lexer<'a>) -> (&'a str, &'a str) {
    let a = lx.rest();
    lx.pos += 1;
    let b = lx.rest();
    (a, b)
}

fn push_one(v: &mut Vec<u32>) {
    v.push(1);
}

pub fn twice(v: &mut Vec<u32>) {
    push_one(v);
    push_one(v);
    let w: &mut Vec<u32> = v;
    w.push(2);
    v.push(3);
}
