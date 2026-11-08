#[test]
fn case1() {
    with_fs(|fs| {
        tree(fs);
        assert_eq!(
            summarize_at(fs, "data").unwrap(),
            Summary {
                files: 3,
                dirs: 3,
                bytes: 25,
                largest: Some(("sub/b.bin".into(), 10))
            }
        );
    });
}
#[test]
fn case2() {
    with_fs(|fs| {
        tree(fs);
        assert_eq!(
            top_at(fs, "data", 2).unwrap(),
            vec![("sub/b.bin".into(), 10), ("sub/deeper/c".into(), 10)]
        );
        assert!(top_at(fs, "data", 0).unwrap().is_empty());
    });
}
#[test]
fn case3() {
    with_fs(|fs| {
        tree(fs);
        assert_eq!(
            summarize_at(fs, "data/empty").unwrap(),
            Summary {
                files: 0,
                dirs: 0,
                bytes: 0,
                largest: None
            }
        );
    });
}
#[test]
fn case4() {
    with_fs(|fs| {
        assert_eq!(
            summarize_at(fs, "missing").unwrap_err().kind(),
            std::io::ErrorKind::NotFound
        );
    });
}
fn tree(fs: &Fixture) {
    fs_mkdir(fs, "data/sub/deeper");
    fs_mkdir(fs, "data/empty");
    fs_write(fs, "data/a.txt", "12345");
    fs_write(fs, "data/sub/b.bin", "1234567890");
    fs_write(fs, "data/sub/deeper/c", "1234567890");
}
