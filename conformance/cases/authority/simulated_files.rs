//@ rules: CAP-1, ROOT-1, ROOT-3, SCOPE-1
// One program runs against real and simulated directories; the build test compares observations.
use std::ffi::OsString;
use std::fs::{Fs, Mode, OpenOptions};
use std::io::{self, ErrorKind as K, Read, Seek, SeekFrom, Write};
use std::task::{TaskError, Tasks};

fn error<T>(out: &mut Vec<String>, result: io::Result<T>, expected: K) {
    match result {
        Ok(_) => panic!("expected {expected:?}"),
        Err(e) => {
            assert_eq!(e.kind(), expected);
            out.push(format!("{:?}", e.kind()));
        }
    }
}

fn case(fs: &Fs, tasks: &Tasks) -> Vec<String> {
    let mut out = Vec::new();
    fs.create_dir_all("tree/inner").unwrap();
    out.push(format!("directory length: {}", fs.metadata("tree").unwrap().len()));
    fs.write("value", "ab").unwrap();
    let mut f = fs.open_with("value", OpenOptions::new().read(true).append(true)).unwrap();
    assert_eq!(f.stream_position().unwrap(), 0);
    f.seek(SeekFrom::Start(0)).unwrap();
    assert_eq!(f.write(b"cd").unwrap(), 2);
    f.seek(SeekFrom::Start(1)).unwrap();
    assert_eq!(f.write(b" ").unwrap(), 1);
    f.seek(SeekFrom::Start(1)).unwrap();
    assert_eq!(f.write(b"").unwrap(), 0);
    assert_eq!(f.stream_position().unwrap(), 1);
    f.set_len(1).unwrap();
    f.write_all(b"z").unwrap();
    write!(f, "{}", "!").unwrap();
    f.flush().unwrap();
    f.rewind().unwrap();
    let mut first = [0; 1];
    (&f).read_exact(&mut first).unwrap();
    out.push(format!("{first:?}"));
    f.seek_relative(-1).unwrap();
    let mut text = String::from("prefix:");
    (&f).read_to_string(&mut text).unwrap();
    out.push(text);
    assert_eq!(f.metadata().unwrap().len(), 3);
    f.set_len(5).unwrap();
    f.sync().unwrap(); f.sync_all().unwrap(); f.sync_data().unwrap();
    fs.rename("value", "new").unwrap();
    f.write_all(b"X").unwrap();
    out.push(format!("{:?}", fs.read("new").unwrap()));
    fs.remove_file("new").unwrap();
    f.rewind().unwrap();
    let mut bytes = Vec::new();
    (&f).read_to_end(&mut bytes).unwrap();
    out.push(format!("{bytes:?}"));
    f.close().unwrap();

    fs.write_synced("value", "kept").unwrap();
    fs.sync_dir(".").unwrap();
    out.push(fs.read_to_string("value").unwrap());

    // Raw final dot/parent names retain create_new's existing-target precedence.
    fs.create_dir_all("d/sub").unwrap();
    for p in ["./", "d//..//"] {
        error(&mut out, fs.open_with(p, OpenOptions::new().write(true).create_new(true)), K::AlreadyExists);
    }
    error(&mut out, fs.open_with("d/", OpenOptions::new().write(true).create_new(true)), K::IsADirectory);
    // Synced writes resolve the target beneath its pinned parent, including `..`.
    for p in ["d/sub/..", "d/..", "d//..//"] {
        error(&mut out, fs.write_synced(p, "bad"), K::PermissionDenied);
    }
    // Linux bounds each syscall pathname argument, before walking its components.
    fs.write("f", "kept").unwrap();
    let within_limit = format!("{}././f", "d/../".repeat(818));
    assert_eq!(within_limit.len(), 4095);
    assert_eq!(fs.read(&within_limit).unwrap(), b"kept");
    let at_limit = format!("{}f", "d/../".repeat(819));
    assert_eq!(at_limit.len(), 4096);
    error(&mut out, fs.read(&at_limit), K::InvalidFilename);
    error(&mut out, fs.create(&at_limit), K::InvalidFilename);
    error(&mut out, fs.metadata(&at_limit), K::InvalidFilename);
    error(&mut out, fs.read(format!("missing/{}f", "./".repeat(2050))), K::InvalidFilename);
    // These calls split the original into a 4095-byte parent and a one-byte name.
    fs.write_synced(&at_limit, "split argument").unwrap();
    out.push(fs.read_to_string("f").unwrap());
    fs.rename(&at_limit, "limit-renamed").unwrap();
    fs.rename("limit-renamed", &at_limit).unwrap();
    let long_name = format!("f{}", "/".repeat(4095));
    error(&mut out, fs.remove_file(&long_name), K::InvalidFilename);
    error(&mut out, fs.rename("f", &long_name), K::InvalidFilename);
    error(&mut out, fs.write_synced(&long_name, "bad"), K::InvalidFilename);
    assert_eq!(fs.read("f").unwrap(), b"split argument");
    fs.remove_file(&at_limit).unwrap();
    assert!(!fs.exists("f").unwrap());

    // Recursive removal strips final slashes before Linux checks argument length.
    for len in [4095, 4096] {
        fs.create_dir_all("d/sub").unwrap();
        fs.remove_dir_all(format!("d{}", "/".repeat(len - 1))).unwrap();
        assert!(!fs.exists("d").unwrap());
        fs.write("f", "kept").unwrap();
        error(&mut out, fs.remove_dir_all(format!("f{}", "/".repeat(len - 1))), K::NotADirectory);
        assert_eq!(fs.read("f").unwrap(), b"kept");
        fs.remove_file("f").unwrap();
    }

    out.push(format!("{} {}", fs.exists("value").unwrap(), fs.exists("absent").unwrap()));
    let mut wo = fs.create("write-only").unwrap();
    error(&mut out, wo.read(&mut [0]), K::PermissionDenied);
    error(&mut out, wo.read_exact(&mut []), K::PermissionDenied);
    wo.write_all(b"body").unwrap();
    let mut ro_file = fs.open("value").unwrap();
    error(&mut out, ro_file.write(b"bad"), K::PermissionDenied);
    error(&mut out, ro_file.write_all(b""), K::PermissionDenied);
    error(&mut out, ro_file.set_len(0), K::PermissionDenied);
    error(&mut out, ro_file.seek(SeekFrom::Current(-1)), K::InvalidInput);
    error(&mut out, ro_file.seek(SeekFrom::Start(u64::MAX)), K::InvalidInput);
    ro_file.flush().unwrap();
    out.push(format!("{} {}", fs.metadata("value").unwrap().len(), fs.metadata("tree").unwrap().is_dir()));
    error(&mut out, fs.open_with("value", OpenOptions::new().write(true).create_new(true)), K::AlreadyExists);
    error(&mut out, fs.open_with("absent", &OpenOptions::new()), K::InvalidInput);
    error(&mut out, fs.open_with("absent", OpenOptions::new().append(true).truncate(true)), K::InvalidInput);
    error(&mut out, fs.read("absent"), K::NotFound);
    error(&mut out, fs.read("absent/../value"), K::NotFound);
    error(&mut out, fs.write("absent/value", "bad"), K::NotFound);
    for p in ["value/", "value/child", "value/.", "value/.."] {
        error(&mut out, fs.read(p), K::NotADirectory);
    }
    for p in ["value/", "absent/", "tree/"] {
        error(&mut out, fs.create(p), K::IsADirectory);
        error(&mut out, fs.open_with(p, OpenOptions::new().write(true).create_new(true)), K::IsADirectory);
    }
    let mut directory = fs.open("tree").unwrap();
    assert!(directory.metadata().unwrap().is_dir());
    assert_eq!(directory.metadata().unwrap().len(), 0);
    directory.read_exact(&mut []).unwrap();
    error(&mut out, directory.read(&mut []), K::IsADirectory);
    assert_eq!(directory.seek(SeekFrom::Start(5)).unwrap(), 5);
    assert_eq!(directory.stream_position().unwrap(), 5);
    directory.rewind().unwrap();
    let mut beyond = fs.open("value").unwrap();
    beyond.seek(SeekFrom::Start(100)).unwrap();
    error(&mut out, beyond.read_exact(&mut [0]), K::UnexpectedEof);
    assert_eq!(beyond.stream_position().unwrap(), 100);
    error(&mut out, fs.read(""), K::NotFound);
    error(&mut out, fs.read("\0"), K::InvalidInput);
    error(&mut out, fs.read("x".repeat(256)), K::InvalidFilename);
    error(&mut out, fs.read("tree"), K::IsADirectory);
    error(&mut out, fs.create("tree"), K::IsADirectory);
    error(&mut out, fs.open_with("tree", OpenOptions::new().write(true)), K::IsADirectory);
    error(&mut out, fs.remove_file("tree"), K::IsADirectory);
    error(&mut out, fs.remove_dir("value"), K::NotADirectory);
    error(&mut out, fs.remove_dir("tree"), K::DirectoryNotEmpty);
    error(&mut out, fs.read_dir("value"), K::NotADirectory);
    error(&mut out, fs.dir("value", Mode::Read), K::NotADirectory);
    error(&mut out, fs.sync_dir("value"), K::NotADirectory);
    error(&mut out, fs.create_dir_all("value"), K::AlreadyExists);
    error(&mut out, fs.create_dir_all("value/child"), K::NotADirectory);
    error(&mut out, fs.create_dir_all("value/."), K::NotADirectory);
    error(&mut out, fs.create_dir_all("value/.."), K::NotADirectory);
    error(&mut out, fs.create("absent/."), K::NotFound);
    error(&mut out, fs.create("absent/.."), K::NotFound);
    fs.create_dir_all("").unwrap();
    fs.write("binary", [255]).unwrap();
    error(&mut out, fs.read_to_string("binary"), K::InvalidData);
    let mut invalid = fs.open("binary").unwrap();
    let mut valid = String::from("unchanged");
    error(&mut out, invalid.read_to_string(&mut valid), K::InvalidData);
    out.push(valid);
    error(&mut out, fs.rename("value", "tree"), K::IsADirectory);
    error(&mut out, fs.rename("tree", "value"), K::NotADirectory);
    error(&mut out, fs.rename("tree", "tree/inner/child"), K::InvalidInput);
    fs.create_dir_all("empty").unwrap();
    error(&mut out, fs.rename("empty", "tree"), K::DirectoryNotEmpty);
    fs.create_dir_all("other").unwrap();
    fs.rename("other", "empty").unwrap();
    fs.write("source", "replacement").unwrap();
    fs.rename("source", "value").unwrap();
    out.push(fs.read_to_string("value").unwrap());
    ro_file.rewind().unwrap();
    let mut replaced = String::new(); ro_file.read_to_string(&mut replaced).unwrap(); out.push(replaced);
    for p in [".", "..", "tree/.", "tree/.."] {
        error(&mut out, fs.remove_file(p), K::InvalidInput);
        error(&mut out, fs.remove_dir(p), K::InvalidInput);
        error(&mut out, fs.rename("value", p), K::InvalidInput);
        error(&mut out, fs.rename(p, "value"), K::InvalidInput);
    }
    let ro = fs.read_only();
    error(&mut out, ro.dir(".", Mode::ReadWrite), K::PermissionDenied);
    error(&mut out, ro.write("missing", "bad"), K::PermissionDenied);
    error(&mut out, ro.write_synced("missing", "bad"), K::PermissionDenied);
    error(&mut out, ro.create("missing"), K::PermissionDenied);
    error(&mut out, ro.open_with("missing", OpenOptions::new().append(true).create(true)), K::PermissionDenied);
    error(&mut out, ro.rename("value", "other"), K::PermissionDenied);
    error(&mut out, ro.remove_file("value"), K::PermissionDenied);
    error(&mut out, ro.remove_dir("empty"), K::PermissionDenied);
    error(&mut out, ro.remove_dir_all("tree"), K::PermissionDenied);
    error(&mut out, ro.create_dir_all("missing"), K::PermissionDenied);
    out.push(ro.read_to_string("value").unwrap());
    let child = fs.dir("tree", Mode::ReadWrite).unwrap();
    child.write("pinned", "identity").unwrap();
    let mut listing = child.read_dir(".").unwrap();
    let entry = listing.find_map(|e| { let e = e.unwrap(); if e.file_name() == "pinned" { Some(e) } else { None } }).unwrap();
    fs.rename("tree", "moved").unwrap();
    out.push(child.read_to_string("inner/../pinned").unwrap());
    out.push(format!("{} {} {:?}", entry.metadata().unwrap().len(), entry.file_type().unwrap().is_file(), entry.path()));
    let inner = child.dir("inner", Mode::Read).unwrap();
    error(&mut out, inner.read("../pinned"), K::PermissionDenied);
    error(&mut out, child.read("../value"), K::PermissionDenied);
    error(&mut out, child.read("/absolute"), K::PermissionDenied);
    error(&mut out, child.rename("pinned", "../escape"), K::PermissionDenied);
    error(&mut out, child.rename("../value", "pinned"), K::PermissionDenied);
    error(&mut out, child.remove_dir_all("."), K::InvalidInput);
    assert!(child.metadata(".").unwrap().is_dir());
    error(&mut out, entry.metadata(), K::NotFound);
    fs.remove_dir_all("moved").unwrap();
    error(&mut out, child.read_dir("."), K::NotFound);
    error(&mut out, child.remove_dir_all("."), K::NotFound);
    fs.create_dir_all("listing").unwrap();
    for name in ["z", "b", "A"] { fs.write(format!("listing/{name}"), name).unwrap(); }
    let mut names: Vec<OsString> = Vec::new();
    for e in fs.read_dir("listing").unwrap() {
        let e = e.unwrap();
        assert!(e.metadata().unwrap().is_file());
        assert!(e.file_type().unwrap().is_file());
        names.push(e.file_name());
    }
    out.push(format!("{names:?}"));
    assert_eq!(names, ["A", "b", "z"]);
    fs.remove_dir_all("listing").unwrap();
    // Open directory capabilities remain pinned even if their name is removed.
    error(&mut out, child.create_dir_all("later"), K::NotFound);
    let mut cancelled_file = fs.create("cancel").unwrap();
    let mut entries = fs.read_dir(".").unwrap();
    let entry = entries.next().unwrap().unwrap();
    tasks.scope::<_, TaskError, _>(|scope| {
        scope.cancel();
        error(&mut out, fs.read("value"), K::Cancelled);
        error(&mut out, fs.read_only().write("\0", "bad"), K::Cancelled);
        error(&mut out, fs.open("value"), K::Cancelled);
        error(&mut out, fs.metadata("value"), K::Cancelled);
        error(&mut out, fs.exists("value"), K::Cancelled);
        error(&mut out, fs.read_dir("."), K::Cancelled);
        error(&mut out, fs.rename("value", "lost"), K::Cancelled);
        error(&mut out, fs.remove_dir_all("."), K::Cancelled);
        error(&mut out, fs.create_dir_all("lost"), K::Cancelled);
        error(&mut out, fs.sync_dir("."), K::Cancelled);
        error(&mut out, cancelled_file.write_all(b"bad"), K::Cancelled);
        error(&mut out, cancelled_file.read_exact(&mut []), K::Cancelled);
        error(&mut out, cancelled_file.flush(), K::Cancelled);
        error(&mut out, cancelled_file.seek(SeekFrom::Start(0)), K::Cancelled);
        error(&mut out, cancelled_file.set_len(0), K::Cancelled);
        error(&mut out, cancelled_file.metadata(), K::Cancelled);
        error(&mut out, cancelled_file.sync(), K::Cancelled);
        error(&mut out, entry.metadata(), K::Cancelled);
        error(&mut out, entry.file_type(), K::Cancelled);
        error(&mut out, entries.next().unwrap(), K::Cancelled);
        assert!(entries.next().is_none());
        cancelled_file.close().unwrap();
        Ok(())
    }).unwrap();
    out.push(format!("{:?}", fs.read("cancel").unwrap()));
    out
}

#[tide::main]
fn main(root: Root) {
    let fs = root.fs.dir("files", Mode::ReadWrite).unwrap();
    println!("OBS:{:?}", case(&fs, &root.tasks));
}

#[cfg(test)]
mod simulated {
    use super::*;
    use std::sim::SimRoot;
    use std::time::Duration;

    #[test]
    fn equivalent_program() {
        let observations = {
            let sim = Root::sim(43);
            sim.fs.create_dir_all("/t").unwrap();
            case(&sim.fs.dir("/t", Mode::ReadWrite).unwrap(), &sim.tasks)
        };
        println!("OBS:{observations:?}");
    }

    fn replay(seed: u64) -> String {
        let sim = Root::sim(seed);
        sim.world.file("/a/b/file", "start");
        let before = sim.elapsed();
        assert_eq!(before, Duration::ZERO);
        assert_eq!(sim.fs.read_to_string("a/b/file").unwrap(), "start");
        sim.tasks.scope::<_, TaskError, _>(|scope| {
            for name in ["first", "second"] {
                let fs = &sim.fs;
                scope.spawn(move || {
                    fs.write(format!("/a/b/{name}"), name).unwrap();
                    fs.open(format!("/a/b/{name}")).unwrap().close().unwrap();
                    Ok(())
                });
            }
            Ok(())
        }).unwrap();
        let trace = sim.trace();
        assert!(trace.contains("\"kind\":\"world\""));
        assert_eq!(trace.lines().filter(|l| l.contains("\"kind\":\"fs\"")).count(), 7);
        assert!(sim.elapsed() > before);
        assert!(sim.elapsed() <= Duration::from_millis(7));
        trace
    }

    #[test]
    fn fixed_seed_trace() { assert_eq!(replay(7), replay(7)); }

    #[test]
    fn cancellation_during_latency_returns_the_operation_result() {
        let sim = Root::sim(19);
        sim.tasks.scope::<_, TaskError, _>(|scope| {
            let child = scope.spawn(|| {
                sim.fs.write("value", "completed").unwrap();
                Ok(())
            });
            let mut blocked = false;
            for _ in 0..1000 {
                if sim.trace().lines().any(|line| line.contains("\"operation\":\"fs.write\"") && line.contains("\"wait\":\"blocked\"")) {
                    blocked = true;
                    break;
                }
                sim.tasks.yield_now().unwrap();
            }
            assert!(blocked, "child never entered its file wait");
            assert_eq!(sim.elapsed(), Duration::ZERO);
            scope.cancel();
            child.join().unwrap();
            assert!(sim.elapsed() > Duration::ZERO && sim.elapsed() <= Duration::from_millis(1));
            Ok(())
        }).unwrap();
        assert_eq!(sim.fs.read("value").unwrap(), b"completed");
    }

    #[sim_test(seeds = 10)]
    fn setup_time_and_cancellation(sim: SimRoot) {
        sim.world.file("/parents/child/file", "old");
        sim.world.file("parents/child/file", "new");
        assert_eq!(sim.elapsed(), Duration::ZERO);
        assert_eq!(sim.fs.read("/parents/child/file").unwrap(), b"new");
        let elapsed = sim.elapsed();
        assert!(elapsed > Duration::ZERO && elapsed <= Duration::from_millis(1));
        let mut listing = sim.fs.read_dir("/parents/child").unwrap();
        let entry = listing.next().unwrap().unwrap();
        let file = sim.fs.open("/parents/child/file").unwrap();
        sim.tasks.scope::<_, TaskError, _>(|scope| {
            scope.cancel();
            let before = sim.elapsed();
            let mut out = Vec::new();
            error(&mut out, sim.fs.write("/parents/child/file", "bad"), K::Cancelled);
            error(&mut out, entry.metadata(), K::Cancelled);
            error(&mut out, listing.next().unwrap(), K::Cancelled);
            assert_eq!(sim.elapsed(), before);
            file.close().unwrap();
            assert!(sim.elapsed() > before && sim.elapsed() <= before + Duration::from_millis(1));
            Ok(())
        }).unwrap();
        assert_eq!(sim.fs.read("/parents/child/file").unwrap(), b"new");
        let trace = sim.trace();
        assert!(trace.contains("\"latency\":0,\"result\":\"Cancelled\""));
        assert!(!trace.contains("new"));
    }
}
