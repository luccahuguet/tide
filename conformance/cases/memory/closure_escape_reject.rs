//@ rules: CLOSURE-1
// A borrowing closure cannot be returned or stored where its origins are not named, even with `move`.
pub struct Row { n: u32 }
pub struct Hook { pred: Box<dyn Fn(&Row) -> bool> }

pub fn boxed(min: &u32) -> Box<dyn Fn(&Row) -> bool> { Box::new(move |r| r.n > *min) } //~ error TD0203
pub fn hook(min: &u32) -> Hook { Hook { pred: Box::new(move |r| r.n > *min) } } //~ error TD0203
pub fn counter() -> impl FnMut() -> u32 { let mut n = 0; || { n += 1; n } } //~ E0373
