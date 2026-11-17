//@ rules: CLOSURE-3
// A trait method runs the selected impl's body, which assumes that impl's header:
// `Pair<'a, 'b>` implies `'b: 'a`, although `Select::select`'s own signature implies nothing.
// Through a type parameter's bound the impl is not known, so that coercion is rejected too. A
// closure that calls the method is accepted.
pub struct Pair<'a, 'b>(pub &'a &'b ());

pub trait Select {
    fn select(value: u32) -> u32;
}

impl<'a, 'b> Select for Pair<'a, 'b> {
    fn select(value: u32) -> u32 { value }
}

pub fn selected() -> fn(u32) -> u32 {
    <Pair<'static, 'static> as Select>::select //~ TD0206
}

pub fn unknown<S: Select>() -> fn(u32) -> u32 {
    S::select //~ TD0206
}

pub fn closure() -> fn(u32) -> u32 {
    |v| <Pair<'static, 'static> as Select>::select(v)
}
