//@ rules: SINK-10
// `expose()` reads the value.
pub fn length(token: &Secret<String>) -> usize {
    token.expose().len()
}
