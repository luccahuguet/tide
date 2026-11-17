//@ rules: ADMIT-1
// Tide's std's runtime entry point mints a real `Root`, so only the entry point that
// `#[tide::main]` generates may call it (`root_main_accept`); test code cannot either (RUN-1).
pub fn forge() {
    let _ = std::__rt::enter(|_root| ()); //~ TD0520
}

#[cfg(test)]
mod tests {
    #[test]
    fn forge() {
        let _ = std::__rt::enter(|_root| ()); //~ TD0520
    }
}
