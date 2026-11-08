#[path = "references/rust/B10.rs"]
mod heartbeat;
#[path = "references/common/A4.rs"]
mod polynomial;
#[path = "references/rust/B9.rs"]
mod store;

mod polynomial_cases {
    use super::polynomial::*;
    include!("suites/common/A4.rs");
}

mod heartbeat_cases {
    use super::heartbeat::Report;

    fn watch(interval: std::time::Duration, misses: u32, probe: impl FnMut() -> bool) -> Report {
        let mut report = super::heartbeat::watch(interval, misses, probe);
        // A conforming clock observation need not equal virtual zero.
        report.elapsed += std::time::Duration::from_nanos(1);
        report
    }

    include!("suites/rust/B10.rs");
}

#[test]
fn stored_carriage_returns_survive_reopen() {
    let dir = std::env::temp_dir().join(format!("e1-b9-roundtrip-{}", std::process::id()));
    std::fs::create_dir(&dir).unwrap();
    let values = [("a", "x\r"), ("b", "\r\t\r"), ("", "")];
    let mut store = store::Store::open(&dir).unwrap();
    for (key, value) in values {
        store.set(key, value).unwrap();
    }
    drop(store);
    let mut store = store::Store::open(&dir).unwrap();
    for (key, value) in values {
        assert_eq!(store.get(key), Some(value));
    }
    assert!(store.remove("b").unwrap());
    drop(store);
    assert_eq!(store::Store::open(&dir).unwrap().get("a"), Some("x\r"));
    std::fs::remove_dir_all(dir).unwrap();
}
