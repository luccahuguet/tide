#[test]
fn case1() {
    with_fs(|fs| {
        {
            let mut s = Store::open(fs).unwrap();
            s.set("a", "1").unwrap();
            s.set("b", "2").unwrap();
        }
        assert_eq!(Store::open(fs).unwrap().get("a"), Some("1"));
        assert_eq!(fs_read(fs, "store.kv"), "a\t1\nb\t2\n");
    });
}
#[test]
fn case2() {
    with_fs(|fs| {
        {
            let mut s = Store::open(fs).unwrap();
            s.set("a", "1").unwrap();
            assert!(s.remove("a").unwrap());
            assert!(!s.remove("absent").unwrap());
        }
        assert_eq!(Store::open(fs).unwrap().get("a"), None);
    });
}
#[test]
fn case3() {
    with_fs(|fs| {
        let mut s = Store::open(fs).unwrap();
        s.set("a", "1").unwrap();
        let before = fs_read(fs, "store.kv");
        assert_eq!(
            s.set("x\ny", "2").unwrap_err().kind(),
            std::io::ErrorKind::InvalidInput
        );
        assert_eq!(fs_read(fs, "store.kv"), before);
    });
}
#[test]
fn case4() {
    with_fs(|fs| {
        fs_write(fs, "store.kv", "oops\n");
        assert!(matches!(Store::open(fs),Err(e) if e.kind()==std::io::ErrorKind::InvalidData));
    });
}
#[test]
fn case5() {
    with_fs(|fs| {
        fs_write(fs, "store.kv.tmp", "garbage");
        let mut s = Store::open(fs).unwrap();
        s.set("a", "1").unwrap();
        assert!(!fs_exists(fs, "store.kv.tmp"));
    });
}
