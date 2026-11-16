//@ rules: CLOSURE-3
// Each route by which a fn item becomes a `fn` pointer, for items that imply `'b: 'a`: an `as`
// cast, the arms of an `if` and of a `match`, a tuple-struct constructor, and an inherent
// method whose impl header implies the bound.
pub struct Pair<'a, 'b>(pub &'a &'b u32);

impl<'a, 'b> Pair<'a, 'b> {
    pub fn get(n: &'b u32) -> &'b u32 { n }
}

fn pick<'a, 'b>(_: &'a &'b (), v: &'b u32) -> &'a u32 { v }
fn other<'a, 'b>(_: &'a &'b (), v: &'b u32) -> &'a u32 { v }

pub fn cast<'a>(unit: &'a &'a (), x: &'a u32) -> &'a u32 {
    (pick as fn(&'a &'a (), &'a u32) -> &'a u32)(unit, x) //~ FX0206
}

pub fn arms<'a>(c: bool, unit: &'a &'a (), x: &'a u32) -> &'a u32 {
    // rustc reifies each arm's block, so the diagnostics point at the blocks.
    let f = if c { //~ FX0206
        pick
    } else { //~ FX0206
        other
    };
    f(unit, x)
}

pub fn matched<'a>(n: u8, unit: &'a &'a (), x: &'a u32) -> &'a u32 {
    let f = match n {
        0 => pick, //~ FX0206
        _ => other, //~ FX0206
    };
    f(unit, x)
}

pub fn ctor<'x>(n: &'x &'x u32) -> Pair<'x, 'x> {
    let f: fn(&'x &'x u32) -> Pair<'x, 'x> = Pair; //~ FX0206
    f(n)
}

pub fn header<'x>(n: &'x u32) -> &'x u32 {
    let f: fn(&'x u32) -> &'x u32 = Pair::<'x, 'x>::get; //~ FX0206
    f(n)
}
