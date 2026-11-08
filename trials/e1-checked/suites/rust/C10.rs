use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    thread,
};
#[test]
fn case1() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    thread::scope(|s| {
        let server = s.spawn(|| serve(&listener, 3));
        let a = s.spawn(|| exchange(&addr, &["abc", "héllo"]));
        let b = s.spawn(|| exchange(&addr, &["x"]));
        let c = s.spawn(|| exchange(&addr, &[]));
        assert_eq!(a.join().unwrap().unwrap(), vec!["cba", "olléh"]);
        assert_eq!(b.join().unwrap().unwrap(), vec!["x"]);
        assert_eq!(c.join().unwrap().unwrap(), Vec::<String>::new());
        assert_eq!(server.join().unwrap().unwrap(), 3);
    });
}
#[test]
fn case2() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    thread::scope(|s| {
        let server = s.spawn(|| serve(&listener, 2));
        let first = TcpStream::connect(&addr).unwrap();
        let mut writer = &first;
        let mut reader = BufReader::new(&first);
        writeln!(writer, "first").unwrap();
        let mut reply = String::new();
        reader.read_line(&mut reply).unwrap();
        assert_eq!(reply, "tsrif\n");
        assert_eq!(exchange(&addr, &["second"]).unwrap(), vec!["dnoces"]);
        writeln!(writer, "QUIT").unwrap();
        assert_eq!(server.join().unwrap().unwrap(), 2);
    });
}
