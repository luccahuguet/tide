pub fn f() -> String {
    std::env::var("HOME").unwrap()
}
