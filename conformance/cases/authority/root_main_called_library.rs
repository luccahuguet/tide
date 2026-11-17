//@ rules: ROOT-1, ROOT-5, ADMIT-1
// Built as a test, a library is executable, so the driver trusts its crate-root `#[tide::main]`
// expansion as it does a binary's; calling that `main` mints a real `Root`, so its tests cannot.
// Outside tests the expansion is not trusted (`root_main_library`).
#[tide::main] fn main() {} //~ TD0520

#[cfg(test)]
mod tests {
    #[test]
    fn mints() {
        let _ = super::main(); //~ TD0506
    }
}
