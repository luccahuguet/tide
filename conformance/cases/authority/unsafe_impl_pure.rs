//@ rules: UNSAFE-1
//@ grants: unsafe
// No grant admits `unsafe impl Pure`: it overlaps std's blanket impl (D4), and
// `#[trusted_pure]` is the spelling.
pub struct Tag {
    p: *const u8,
}

// SAFETY: present, so the overlap is the only error.
unsafe impl Pure for Tag {} //~ E0119 TD0402
