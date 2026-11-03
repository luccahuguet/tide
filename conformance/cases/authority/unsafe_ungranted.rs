//@ rules: UNSAFE-1
// Without the `unsafe` grant: an `unsafe fn`, with a body or not (a required trait method), an
// `unsafe extern` block, an empty one too, and every item in one, `safe` ones included, an
// `unsafe` block, an unsafe attribute, and `unsafe impl` of a user's trait and of `Sync`.
// Declaring an `unsafe trait` needs no grant, and calling a `safe` foreign function needs no
// `unsafe` block.
pub unsafe fn raw_len(p: &[u8]) -> usize { //~ FX0901
    p.len()
}

pub fn len(p: &[u8]) -> usize {
    // SAFETY: `raw_len` reads only the slice's length.
    unsafe { raw_len(p) } //~ FX0901
}

#[unsafe(no_mangle)] //~ FX0901
pub fn tide_probe() -> u32 {
    7
}

pub trait RawRead {
    unsafe fn read_raw(&self) -> u8; //~ FX0901
}

unsafe extern "C" { //~ FX0901
    fn tide_probe_ext(); //~ FX0901
    pub safe fn abs(x: i32) -> i32; //~ FX0901
    pub safe static TIDE_PROBE_COUNT: u32; //~ FX0901
}

pub fn magnitude(x: i32) -> i32 {
    abs(x)
}

#[link(name = "m")]
unsafe extern "C" {} //~ FX0901

pub unsafe trait Zeroable {}

// SAFETY: every bit pattern is a valid `u32`.
unsafe impl Zeroable for u32 {} //~ FX0901

pub struct Token(std::cell::Cell<u8>);

// SAFETY: no method reads or writes the `Cell`.
unsafe impl Sync for Token {} //~ FX0901
