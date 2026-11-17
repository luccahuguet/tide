//@ rules: GLOBAL-4
//@ grants: unsafe
// Under the `unsafe` grant an address counts as a `usize` (D3), so an address cast that may
// lose bits is lossy `as`, as `usize as u32` is.
pub fn truncate(p: *const u8) -> u8 {
    p as u8 //~ TD0301
}

pub fn narrow(p: *const u8) -> u32 {
    p as u32 //~ TD0301
}

pub fn signed(p: *mut u8) -> isize {
    p as isize //~ TD0301
}

pub fn code(f: fn()) -> u32 {
    f as u32 //~ TD0301
}

pub fn wide(n: u64) -> *const u8 {
    n as *const u8 //~ TD0301
}

pub fn wider(n: u128) -> *mut u8 {
    n as *mut u8 //~ TD0301
}

pub fn from_signed(n: isize) -> *const u8 {
    n as *const u8 //~ TD0301
}

// The spellings that lose no bits are accepted.
pub fn spelled(p: *const u8, n: u64, s: isize) -> Option<(u32, *const u8, *const u8)> {
    let low = u32::try_from(p.addr()).ok()?;
    Some((low, usize::try_from(n).ok()? as *const u8, s.cast_unsigned() as *const u8))
}
