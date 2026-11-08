fn randoms(n: usize) -> Vec<u32> {
    let mut x = 0x6e624eb7u32;
    (0..n)
        .map(|_| {
            x ^= x.wrapping_shl(13);
            x ^= x.wrapping_shr(17);
            x ^= x.wrapping_shl(5);
            x
        })
        .collect()
}
