//@ rules: REF-0301
// The source type's range decides, even when a particular constant fits.
#[repr(i16)]
pub enum Signed { Negative = -1, Large = 256 }
#[repr(u128)]
pub enum Unsigned { Huge = u128::MAX }
pub fn casts(u: u64, i: i32, p: usize, f: f64, s: Signed, big: Unsigned) {
    let _ = 1u64 as u8; //~ TD0301
    let _ = u as usize; //~ TD0301
    let _ = p as u32; //~ TD0301
    let _ = p as isize; //~ TD0301
    let _ = i as u32; //~ TD0301
    let _ = 1u32 as f32; //~ TD0301
    let _ = u as f64; //~ TD0301
    let _ = p as f64; //~ TD0301
    let _ = f as f32; //~ TD0301
    let _ = f as i64; //~ TD0301
    let _ = s as u16; //~ TD0301
    let _ = Signed::Large as u8; //~ TD0301
    let _ = big as i128; //~ TD0301
}
