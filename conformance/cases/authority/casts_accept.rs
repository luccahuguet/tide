//@ rules: REF-0301
// Each lossless family in reference 4.2, including portable pointer-sized widths.
#[repr(i16)]
pub enum Signed { Negative = -1, Large = 256 }
#[repr(u128)]
pub enum Unsigned { Huge = u128::MAX }
pub enum Small { Zero, One }
pub fn casts(u: u8, i: i16, n: u32, p: usize, s: isize, c: char, f: f32) {
    let _ = u as u16;
    let _ = i as i32;
    let _ = u as i16;
    let _ = n as usize;
    let _ = p as u64;
    let _ = p as i128;
    let _ = s as i64;
    let _ = true as u8;
    let _ = u as char;
    let _ = c as u32;
    let _ = c as i64;
    let _ = i as f32;
    let _ = n as f64;
    let _ = f as f64;
    let _ = Signed::Large as i16;
    let _ = Unsigned::Huge as u128;
    let _ = Small::One as u8;
    let _ = n as u32;
    let _ = p as usize;
    let _ = u8::try_from(n);
    let _ = i.cast_unsigned();
}
