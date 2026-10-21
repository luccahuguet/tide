//@ rules: SINK-10
// `Secret<T>` implements neither `Debug` nor `Display`.
pub fn show(token: &Secret<String>) -> String {
    format!("{:?}", token) //~ E0277
}
