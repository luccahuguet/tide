#[test]
fn case1() {
    with_fs(|fs| {
        fs_mkdir(fs, "out");
        fs_write(fs, "out/a.txt", "old");
        assert_eq!(
            export(fs, &[("a.txt", "1"), ("2026/jan.txt", "2")]).unwrap(),
            2
        );
        assert_eq!(fs_read(fs, "out/a.txt"), "1");
        assert_eq!(fs_read(fs, "out/2026/jan.txt"), "2");
    });
}
#[test]
fn case2() {
    with_fs(|fs| {
        assert!(
            matches!(export(fs,&[("ok","x"),("../evil","y")]),Err(ExportError::BadName(s)) if s=="../evil")
        );
        assert!(!fs_exists(fs, "out/ok"));
        assert!(!fs_exists(fs, "evil"));
    });
}
#[test]
fn case3() {
    for name in ["/abs", "a//b", "a/./b", "", "a\\b"] {
        with_fs(|fs| {
            assert!(matches!(export(fs,&[(name,"x")]),Err(ExportError::BadName(s)) if s==name));
        });
    }
}
#[test]
fn case4() {
    with_fs(|fs| {
        assert!(
            matches!(export(fs,&[("a","1"),("a","2")]),Err(ExportError::Duplicate(s)) if s=="a")
        );
    });
}
#[test]
fn case5() {
    with_fs(|fs| {
        fs_write(fs, "out", "file");
        assert!(matches!(export(fs, &[("a", "1")]), Err(ExportError::Io(_))));
    });
}
