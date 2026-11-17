// Owner's guide, chapters 2 and 7: an `unsafe` block. Without the operator's
// `--tide-grant=unsafe` admission rejects it (TD0901); with the grant it is admitted and
// listed as a trust hole in the crate's trust artifact.

/// Reads the first byte without a bounds check.
pub fn first_byte(bytes: &[u8]) -> u8 {
    assert!(!bytes.is_empty());
    // SAFETY: the assert above guarantees index 0 is in bounds.
    unsafe { *bytes.get_unchecked(0) }
}

/// Safe code: no trust hole.
pub fn last_byte(bytes: &[u8]) -> Option<u8> {
    bytes.last().copied()
}
