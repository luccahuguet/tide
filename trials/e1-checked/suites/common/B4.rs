#[test]
fn case1() {
    with_fs(|fs| {
        let mut log = RotLog::open(fs, 10).unwrap();
        for line in ["aaaa", "bbbb", "cc"] {
            log.append(line).unwrap();
        }
        assert_eq!(fs_read(fs, "old.1.log"), "aaaa\nbbbb\n");
        assert_eq!(fs_read(fs, "current.log"), "cc\n");
        assert_eq!(read_all(fs).unwrap(), vec!["aaaa", "bbbb", "cc"]);
    });
}
#[test]
fn case2() {
    with_fs(|fs| {
        let mut log = RotLog::open(fs, 10).unwrap();
        for line in ["aaaa", "bbbb", "cc", "dddddddd", "e"] {
            log.append(line).unwrap();
        }
        assert_eq!(read_all(fs).unwrap(), vec!["cc", "dddddddd", "e"]);
    });
}
#[test]
fn case3() {
    with_fs(|fs| {
        let mut log = RotLog::open(fs, 10).unwrap();
        log.append("before").unwrap();
        assert_eq!(
            log.append("a\nb").unwrap_err().kind(),
            std::io::ErrorKind::InvalidInput
        );
        assert_eq!(fs_read(fs, "current.log"), "before\n");
        assert!(!fs_exists(fs, "old.1.log"));
        assert!(!fs_exists(fs, "old.2.log"));
    });
}
#[test]
fn case4() {
    with_fs(|fs| {
        let mut log = RotLog::open(fs, 10).unwrap();
        log.append("abcdefghijklmnopqrst").unwrap();
        assert_eq!(fs_read(fs, "current.log"), "abcdefghijklmnopqrst\n");
        log.append("x").unwrap();
        assert_eq!(fs_read(fs, "old.1.log"), "abcdefghijklmnopqrst\n");
        assert_eq!(fs_read(fs, "current.log"), "x\n");
    });
}
#[test]
fn case5() {
    with_fs(|fs| {
        {
            let mut log = RotLog::open(fs, 10).unwrap();
            log.append("a").unwrap();
        }
        let mut log = RotLog::open(fs, 10).unwrap();
        log.append("b").unwrap();
        assert_eq!(fs_read(fs, "current.log"), "a\nb\n");
    });
}
