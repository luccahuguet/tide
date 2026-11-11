//@ rules: CAP-1, ROOT-1, SCOPE-1
// Independent file contract program; the build test supplies its real directory and symlinks.
// The build test also executes TASK-3 cancellation assertions below.
use std::ffi::{OsStr, OsString};
use std::fs::{Fs, Mode, OpenOptions};
use std::io::{self, ErrorKind as K, Read, Seek, SeekFrom, Write};
use std::path::Path;

fn kind<T>(r: io::Result<T>, want: K) {
    match r {
        Err(e) => assert_eq!(e.kind(), want),
        Ok(_) => panic!("expected {want:?}"),
    }
}

fn invalid_paths(fs: &Fs, path: &str, want: K) {
    kind(fs.dir(path, Mode::Read), want);
    kind(fs.open(path), want);
    kind(fs.create(path), want);
    kind(fs.open_with(path, OpenOptions::new().read(true)), want);
    kind(fs.read(path), want);
    kind(fs.read_to_string(path), want);
    kind(fs.write(path, b"bad"), want);
    kind(fs.write_synced(path, b"bad"), want);
    kind(fs.metadata(path), want);
    kind(fs.exists(path), want);
    kind(fs.read_dir(path), want);
    kind(fs.rename(path, "access"), want);
    kind(fs.rename("access", path), want);
    kind(fs.remove_file(path), want);
    kind(fs.remove_dir(path), want);
    kind(fs.remove_dir_all(path), want);
    kind(fs.create_dir_all(path), want);
    kind(fs.sync_dir(path), want);
}

fn files(fs: &Fs) -> io::Result<()> {
    fs.write("record", b"ab")?;
    let mut file = fs.open_with("record", OpenOptions::new().read(true).append(true))?;
    file.seek(SeekFrom::Start(0))?;
    file.write_all(b"cd")?;
    let mut borrowed = &file;
    borrowed.seek(SeekFrom::Start(0))?;
    let mut text = String::new();
    borrowed.read_to_string(&mut text)?;
    assert_eq!(text, "abcd");
    file.set_len(1)?;
    file.write_all(b"z")?;
    assert_eq!(fs.read("record")?, b"az");
    assert_eq!(file.metadata()?.len(), 2);
    file.sync()?;
    file.sync_all()?;
    file.sync_data()?;
    fs.rename("record", "renamed")?;
    file.write_all(b"!")?;
    assert_eq!(fs.read("renamed")?, b"az!");
    fs.remove_file("renamed")?;
    file.rewind()?;
    text.clear();
    file.read_to_string(&mut text)?;
    assert_eq!(text, "az!");
    file.close()?;

    let mut write_only = fs.create("access")?;
    kind(write_only.read(&mut [0]), K::PermissionDenied);
    kind(write_only.read_exact(&mut []), K::PermissionDenied);
    write_only.write_all(b"kept")?;
    let mut read_only = fs.read_only().open("access")?;
    assert_eq!(
        format!("{read_only:?}"),
        "File { read: true, write: false }"
    );
    read_only.flush()?;
    (&read_only).flush()?;
    kind(read_only.write(b"bad"), K::PermissionDenied);
    kind(read_only.write_all(b""), K::PermissionDenied);
    kind(read_only.set_len(0), K::PermissionDenied);
    let ro = fs.read_only();
    kind(ro.write("missing", b"x"), K::PermissionDenied);
    kind(ro.create("missing"), K::PermissionDenied);
    kind(
        ro.open_with("missing", OpenOptions::new().append(true).create(true)),
        K::PermissionDenied,
    );
    kind(
        ro.open_with("missing", OpenOptions::new().truncate(true)),
        K::PermissionDenied,
    );
    kind(ro.rename("missing", "other"), K::PermissionDenied);
    kind(ro.remove_file("missing"), K::PermissionDenied);
    kind(ro.remove_dir("missing"), K::PermissionDenied);
    kind(ro.remove_dir_all("missing"), K::PermissionDenied);
    kind(ro.create_dir_all("missing"), K::PermissionDenied);
    kind(ro.write_synced("missing", b"x"), K::PermissionDenied);
    kind(ro.dir("missing", Mode::ReadWrite), K::PermissionDenied);
    kind(ro.write("\0", b"bad"), K::PermissionDenied);
    kind(ro.dir("\0", Mode::ReadWrite), K::PermissionDenied);
    assert_eq!(fs.read("access")?, b"kept");
    assert!(!fs.exists("missing")?);
    fs.dir(".", Mode::Read)?.open("access")?.close()?;

    fs.write_synced("durable", "committed")?;
    fs.sync_dir(".")?;
    assert_eq!(fs.read_to_string("durable")?, "committed");
    kind(fs.sync_dir("durable"), K::NotADirectory);
    kind(fs.dir("durable", Mode::Read), K::NotADirectory);
    kind(fs.read_dir("durable"), K::NotADirectory);
    kind(fs.read(""), K::NotFound);
    kind(fs.read("missing/../access"), K::NotFound);
    kind(fs.read("access/"), K::NotADirectory);
    kind(fs.read("access/child"), K::NotADirectory);
    invalid_paths(fs, "\0", K::InvalidInput);
    invalid_paths(fs, "/absolute", K::PermissionDenied);
    kind(
        fs.open_with("/absolute", &OpenOptions::new()),
        K::PermissionDenied,
    );
    kind(fs.read("x".repeat(256)), K::InvalidFilename);
    fs.write("binary", [255])?;
    kind(fs.read_to_string("binary"), K::InvalidData);
    fs.create_dir_all("nested/leaf")?;
    fs.create_dir_all("")?;
    kind(fs.create_dir_all("access"), K::AlreadyExists);
    kind(fs.create_dir_all("access/child"), K::NotADirectory);
    kind(fs.create_dir_all("access/."), K::NotADirectory);
    kind(fs.remove_file("nested"), K::IsADirectory);
    kind(fs.remove_dir("access"), K::NotADirectory);
    kind(fs.remove_dir("nested"), K::DirectoryNotEmpty);
    kind(fs.read("nested"), K::IsADirectory);
    kind(fs.create("nested"), K::IsADirectory);
    kind(
        fs.open_with("access", OpenOptions::new().write(true).create_new(true)),
        K::AlreadyExists,
    );
    kind(fs.open_with("absent", &OpenOptions::new()), K::InvalidInput);
    kind(
        fs.open_with("absent", OpenOptions::new().append(true).truncate(true)),
        K::InvalidInput,
    );
    for p in [".", "..", "nested/.", "nested/..", "nested/../"] {
        kind(fs.remove_file(p), K::InvalidInput);
        kind(fs.remove_dir(p), K::InvalidInput);
        kind(fs.rename("access", p), K::InvalidInput);
        kind(fs.rename(p, "access"), K::InvalidInput);
    }
    kind(fs.rename("access", "nested"), K::IsADirectory);
    kind(fs.rename("nested", "access"), K::NotADirectory);
    kind(fs.rename("nested", "nested/leaf/new"), K::InvalidInput);
    fs.create_dir_all("empty")?;
    let mut empty_listing = fs.read_dir("empty")?;
    kind(fs.rename("empty", "nested"), K::DirectoryNotEmpty);
    fs.create_dir_all("replacement")?;
    fs.rename("replacement", "empty")?;
    fs.write("source", "new")?;
    fs.write("target", "old")?;
    fs.rename("source", "target")?;
    assert_eq!(fs.read_to_string("target")?, "new");

    fs.create_dir_all("listing")?;
    for name in ["b", "a", "C"] {
        fs.write(Path::new("listing").join(name), name)?;
    }
    for n in 0..300 {
        fs.write(format!("listing/{n:03}"), b"")?;
    }
    let mut names: Vec<OsString> = Vec::new();
    for entry in fs.read_dir("listing")? {
        let entry = entry?;
        assert_ne!(entry.file_name().as_os_str(), OsStr::new("."));
        assert!(entry.file_type()?.is_file());
        assert!(entry.metadata()?.is_file());
        assert_eq!(entry.path(), Path::new("listing").join(entry.file_name()));
        names.push(entry.file_name());
    }
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted);
    assert_eq!(names.len(), 303);

    let child = fs.dir("nested", Mode::ReadWrite)?;
    child.write("inside", "pinned")?;
    let entry = child
        .read_dir(".")?
        .find_map(|e| {
            let e = e.unwrap();
            if e.file_name() == "inside" {
                Some(e)
            } else {
                None
            }
        })
        .unwrap();
    fs.rename("nested", "moved")?;
    assert_eq!(child.read_to_string("inside")?, "pinned");
    assert_eq!(entry.metadata()?.len(), 6);
    kind(child.read("../access"), K::PermissionDenied);
    kind(child.read("/etc/hostname"), K::PermissionDenied);
    kind(child.rename("inside", "../escaped"), K::PermissionDenied);
    kind(child.remove_file("/absolute"), K::PermissionDenied);
    child.create_dir_all("sub")?;
    assert_eq!(child.read_to_string("sub/../inside")?, "pinned");
    let deeper = child.dir("sub", Mode::Read)?;
    kind(deeper.read("../inside"), K::PermissionDenied);
    kind(child.remove_dir_all("."), K::InvalidInput);
    assert!(child.metadata(".")?.is_dir());
    assert_eq!(child.read_dir(".")?.count(), 0);
    fs.remove_dir_all("moved")?;
    fs.remove_dir_all("listing")?;
    assert!(empty_listing.next().is_none());
    Ok(())
}

fn symlinks(fs: &Fs) -> io::Result<()> {
    assert_eq!(fs.read_to_string("links/ok")?, "safe");
    fs.create_dir_all("links/dir")?;
    kind(fs.create_dir_all("links/dangling"), K::AlreadyExists);
    for path in [
        "links/up/outside",
        "links/abs/hostname",
        "../outside",
        "/etc/hostname",
    ] {
        kind(fs.read(path), K::PermissionDenied);
        kind(fs.write(path, b"bad"), K::PermissionDenied);
        kind(fs.write_synced(path, b"bad"), K::PermissionDenied);
    }
    kind(fs.metadata("links/up/outside"), K::PermissionDenied);
    kind(fs.exists("links/up/outside"), K::PermissionDenied);
    kind(fs.read_dir("links/up"), K::PermissionDenied);
    kind(fs.dir("links/up", Mode::Read), K::PermissionDenied);
    kind(fs.create_dir_all("links/up/new"), K::PermissionDenied);
    kind(fs.rename("in.txt", "links/up/renamed"), K::PermissionDenied);
    kind(fs.remove_file("links/up/outside"), K::PermissionDenied);
    kind(fs.remove_dir("links/up/new"), K::PermissionDenied);
    kind(fs.remove_dir_all("links/up/tree"), K::PermissionDenied);
    kind(fs.sync_dir("links/up"), K::PermissionDenied);
    for entry in fs.read_dir("links")? {
        let entry = entry?;
        assert!(entry.metadata()?.is_symlink());
        assert!(entry.file_type()?.is_symlink());
    }
    fs.write_synced("new-link", "linked data")?;
    assert_eq!(fs.read_to_string("synced/created")?, "linked data");
    fs.write_synced("long-link", "long target")?;
    assert_eq!(fs.read_to_string("synced/long-target")?, "long target");
    // A parent-directory link plus 40 final links exceeds Linux's total budget.
    let error = fs.write("alias/l0", "bad").unwrap_err();
    assert_eq!(error.raw_os_error(), Some(40)); // Linux ELOOP has no stable ErrorKind.
    assert_eq!(fs.read_to_string("deep/target")?, "unchanged");
    fs.write_synced("alias/l0", "final-link budget")?;
    assert_eq!(fs.read_to_string("deep/target")?, "final-link budget");
    assert_eq!(
        fs.write_synced("alias/too-many", "bad")
            .unwrap_err()
            .raw_os_error(),
        Some(40)
    );
    assert_eq!(fs.read_to_string("deep/target")?, "final-link budget");
    kind(fs.remove_dir_all("links/up/"), K::NotADirectory);
    fs.remove_dir_all("links")?;
    Ok(())
}

#[tide::main]
fn main(root: Root) -> io::Result<()> {
    let fs = root.fs.dir("files", Mode::ReadWrite)?;
    files(&fs)?;
    symlinks(&fs)?;
    fs.write("cancel", "unchanged")?;
    let mut file = fs.open_with("cancel", OpenOptions::new().read(true).write(true))?;
    let mut read_only = fs.open("cancel")?;
    let mut listing = fs.read_dir(".")?;
    fs.create_dir_all("cancel-empty")?;
    let mut empty = fs.read_dir("cancel-empty")?;
    let entry = listing.next().unwrap()?;
    root.tasks
        .scope(|scope| -> Result<(), std::task::TaskError> {
            scope.cancel();
            invalid_paths(&fs, "\0", K::Cancelled);
            kind(fs.read("cancel"), K::Cancelled);
            kind(fs.read_to_string("cancel"), K::Cancelled);
            kind(fs.write("cancel", "bad"), K::Cancelled);
            kind(fs.write_synced("cancel", "bad"), K::Cancelled);
            kind(fs.open("cancel"), K::Cancelled);
            kind(fs.create("cancel"), K::Cancelled);
            kind(
                fs.open_with("cancel", OpenOptions::new().append(true)),
                K::Cancelled,
            );
            kind(fs.dir(".", Mode::Read), K::Cancelled);
            kind(fs.metadata("cancel"), K::Cancelled);
            kind(fs.exists("cancel"), K::Cancelled);
            kind(fs.read_dir("."), K::Cancelled);
            kind(fs.rename("cancel", "lost"), K::Cancelled);
            kind(fs.create_dir_all("lost"), K::Cancelled);
            kind(fs.remove_file("cancel"), K::Cancelled);
            kind(fs.remove_dir("."), K::Cancelled);
            kind(fs.remove_dir_all("."), K::Cancelled);
            kind(fs.sync_dir("."), K::Cancelled);
            kind(fs.read_only().write("missing", "bad"), K::Cancelled);
            kind(file.read(&mut [0]), K::Cancelled);
            kind(file.read_exact(&mut []), K::Cancelled);
            kind(file.write(b"bad"), K::Cancelled);
            kind(file.write_all(b""), K::Cancelled);
            kind(file.write_fmt(format_args!("")), K::Cancelled);
            kind(file.flush(), K::Cancelled);
            kind(read_only.flush(), K::Cancelled);
            kind(file.seek(SeekFrom::Start(0)), K::Cancelled);
            kind(file.set_len(0), K::Cancelled);
            kind(file.metadata(), K::Cancelled);
            kind(file.sync(), K::Cancelled);
            kind(file.sync_all(), K::Cancelled);
            kind(file.sync_data(), K::Cancelled);
            kind(entry.metadata(), K::Cancelled);
            kind(entry.file_type(), K::Cancelled);
            kind(listing.next().unwrap(), K::Cancelled);
            assert!(listing.next().is_none());
            kind(empty.next().unwrap(), K::Cancelled);
            assert!(empty.next().is_none());
            file.close().unwrap();
            Ok(())
        })
        .unwrap();
    assert_eq!(fs.read_to_string("cancel")?, "unchanged");
    println!("FILES_RAN");
    Ok(())
}
