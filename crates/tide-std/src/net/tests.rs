// NET-HTTP-1, -2, -3 and -5 (conformance/rules.md). Real-backend cases run against local
// servers on loopback only, never the internet; TLS uses the test root in `testdata/`.
use super::*;
use std::io::Write as _;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

const ROOT: &[u8] = include_bytes!("testdata/root.der");
const SERVER: &[u8] = include_bytes!("testdata/server.der");
const UNTRUSTED: &[u8] = include_bytes!("testdata/untrusted.der");
const KEY: &[u8] = include_bytes!("testdata/server.key.der");

fn url(s: &str) -> Url {
    Url::parse(s).unwrap()
}
fn real_deadline(d: Duration) -> crate::time::Instant {
    crate::time::Instant(crate::time::monotonic() + d)
}
fn kind(r: &Result<Vec<u8>, NetError>) -> Option<ErrorKind> {
    match r {
        Err(NetError::Io(e)) => Some(e.kind()),
        _ => None,
    }
}

// NET-HTTP-1: one key for entries, URLs and addresses; refused hosts everywhere.
const REFUSED: &[&str] = &[
    "b%C3%BCcher.example",
    "%78n--bcher-kva.example",
    "%EF%BD%81pi.example.com",
    "a..b",
    ".a",
    "a..",
    "xn--bcher-kva.example",
    "XN--a.example",
    "*.example.com",
    "bücher.example",
    "bad\0host.test",
];
#[test]
fn net_host_refuses_hosts_as_urls_entries_and_addresses() {
    for host in REFUSED {
        assert!(Url::parse(&format!("http://{host}/")).is_err(), "{host}");
        assert!(Url::parse(&format!("https://{host}:8443/x")).is_err(), "{host}");
        assert!(host::entry(&format!("{host}:443")).is_none(), "{host}");
        let e = Net::new(None).connect((*host, 80)).unwrap_err();
        assert_eq!(e.kind(), ErrorKind::InvalidInput, "{host}");
        let e = Net::new(None).connect(format!("{host}:80")).unwrap_err();
        assert_eq!(e.kind(), ErrorKind::InvalidInput, "{host}");
    }
    // `%` stays legal in paths and queries, and is sent as written.
    let u = url("http://example.com/a%20b?q=%41#f");
    assert_eq!((u.path(), u.query()), ("/a%20b", Some("q=%41")));
    let (_, key) = u.key().unwrap();
    assert!(http::request(&u, false, &key).starts_with("GET /a%20b?q=%41 HTTP/1.1\r\nHost: example.com\r\n"));
    for entry in ["example.com", "https://example.com", "example.com:0", "[::1]", "::1:80", "[fe80::1%1]:80"] {
        assert!(host::entry(entry).is_none(), "{entry}");
    }
}
#[test]
fn net_host_keys_aliases_as_one_destination() {
    let key = |e: &str| host::entry(e).unwrap();
    for alias in ["0x7f.1:80", "2130706433:80", "0177.0.0.1:80", "[::ffff:127.0.0.1]:80", "127.1:80"] {
        assert_eq!(key(alias), key("127.0.0.1:80"), "{alias}");
    }
    assert_eq!(key("[0:0:0:0:0:0:0:1]:8080"), key("[::1]:8080"));
    assert_eq!(key("Example.COM.:443"), key("example.com:443"));
    assert_ne!(key("localhost:80"), key("127.0.0.1:80"));
    let only = Net::new(None).only(["example.com:443", "127.0.0.1:80"]);
    assert!(only.allows(&url("https://EXAMPLE.com.:443/x").key().unwrap().1));
    for denied in ["https://www.example.com/", "http://example.com/", "https://93.184.215.14/"] {
        assert!(!only.allows(&url(denied).key().unwrap().1), "{denied}");
    }
    assert!(only.allows(&url("http://0x7f.1/").key().unwrap().1));
    // The host as written is the host the URL crate parses.
    for input in [
        "HTTP://user@Example.COM.:8080/x",
        "http:\\\\example.com\\p",
        "https://[::1]:443/",
        "http://0x7f.1/",
        " http://a.b/?q#f ",
        "https:example.com",
        "http://a@b@c.test/",
    ] {
        let written = host::url_host(input).unwrap();
        let parsed = url(input);
        assert_eq!(host::host(&written), host::host(parsed.host_str().unwrap()), "{input}");
    }
}
/// The idna_adapter decision (research/42 §8): these keys are identical under 1.0.0 and 1.1.0.
#[test]
fn net_host_ascii_keys_are_fixed() {
    let keys: Vec<String> = ["Example.COM.:443", "a-b.c_d.test:1", "0x7f.1:80", "LOCALHOST:9", "[::FFFF:7f00:1]:2"]
        .iter()
        .map(|e| host::entry(e).unwrap().to_string())
        .collect();
    assert_eq!(keys, ["example.com:443", "a-b.c_d.test:1", "127.0.0.1:80", "localhost:9", "127.0.0.1:2"]);
}
#[test]
fn denied_and_refused_shapes_need_no_lookup() {
    let before = resolve::CALLS.with(|n| n.get());
    let net = Net::new(None).only(["example.com:443"]);
    let deadline = real_deadline(Duration::from_secs(5));
    assert!(matches!(net.get(&url("https://evil.test/"), deadline), Err(NetError::Denied)));
    assert!(matches!(net.get(&url("http://example.com/"), deadline), Err(NetError::Denied)));
    for shape in ["ftp://example.com/", "file:///etc/passwd", "https://u:p@example.com/", "data:,x"] {
        assert_eq!(kind(&net.get(&url(shape), deadline)), Some(ErrorKind::InvalidInput), "{shape}");
    }
    assert_eq!(resolve::CALLS.with(|n| n.get()), before);
}
// NET-HTTP-5: no_bind refuses before any resolution; the counter does see a real lookup.
#[test]
fn no_bind_refuses_before_resolution() {
    let before = resolve::CALLS.with(|n| n.get());
    let e = Net::new(None).only([""; 0]).no_bind().bind(("unlisted.test", 8080)).unwrap_err();
    assert_eq!(e.kind(), ErrorKind::PermissionDenied);
    assert_eq!(resolve::CALLS.with(|n| n.get()), before);
    let c = Network::new(None).context();
    let span = std::panic::Location::caller();
    assert!(resolve::lookup(&c, "localhost", 1, None, span).is_ok());
    assert_eq!(resolve::CALLS.with(|n| n.get()), before + 1);
}

/// A loopback server that reads one request head, then runs `respond` on the socket.
fn serve(
    respond: impl FnOnce(std::net::TcpStream, String) + Send + 'static,
) -> (u16, std::thread::JoinHandle<()>) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let handle = std::thread::spawn(move || {
        let (mut s, _) = listener.accept().unwrap();
        let head = read_head(&mut s);
        respond(s, head);
    });
    (port, handle)
}
fn read_head(s: &mut impl std::io::Read) -> String {
    let mut head = Vec::new();
    let mut byte = [0];
    while !head.ends_with(b"\r\n\r\n") && s.read(&mut byte).unwrap_or(0) == 1 {
        head.push(byte[0]);
    }
    String::from_utf8(head).unwrap()
}
fn plain(bytes: &'static [u8], then: &'static str) -> impl FnOnce(std::net::TcpStream, String) + Send {
    move |mut s, _| {
        s.write_all(bytes).unwrap();
        if then == "hold" {
            std::thread::sleep(Duration::from_secs(3));
        }
    }
}
fn get_plain(bytes: &'static [u8], then: &'static str) -> Result<Vec<u8>, NetError> {
    let (port, server) = serve(plain(bytes, then));
    let r = Net::new(None).get(&url(&format!("http://127.0.0.1:{port}/")), real_deadline(Duration::from_secs(10)));
    drop(server);
    r
}

// NET-HTTP-2 on the real backend.
#[test]
fn real_get_sends_the_fixed_request_and_frames_responses() {
    let (port, server) = serve(|mut s, head| {
        s.write_all(format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{head}", head.len()).as_bytes()).unwrap();
    });
    let body = Net::new(None)
        .get(&url(&format!("http://127.0.0.1:{port}/p?q=1#frag")), real_deadline(Duration::from_secs(10)))
        .unwrap();
    server.join().unwrap();
    assert_eq!(
        String::from_utf8(body).unwrap(),
        format!("GET /p?q=1 HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nUser-Agent: tide\r\nAccept: */*\r\nConnection: close\r\n\r\n")
    );
    let start = std::time::Instant::now();
    assert_eq!(get_plain(b"HTTP/1.1 204 No Content\r\nContent-Length: 5\r\n\r\n", "hold").unwrap(), b"");
    assert!(start.elapsed() < Duration::from_secs(2), "204 waited for close");
    assert!(matches!(get_plain(b"HTTP/1.1 304 Not Modified\r\nContent-Length: 10\r\n\r\n", "close"), Err(NetError::Status(304))));
    assert_eq!(get_plain(b"HTTP/1.1 100 Continue\r\n\r\nHTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok", "close").unwrap(), b"ok");
    assert!(matches!(get_plain(b"HTTP/1.1 100 Continue\r\n\r\n", "close"), Err(NetError::Reset)));
    assert_eq!(kind(&get_plain(b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: x\r\n\r\n", "close")), Some(ErrorKind::InvalidData));
    for bad in [
        &b"HTTP/1.1 200 OK\r\nTransfer-Encoding: gzip\r\n\r\nx"[..],
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: gzip, chunked\r\n\r\n0\r\n\r\n",
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nContent-Length: 1\r\n\r\n0\r\n\r\n",
        b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nContent-Length: 6\r\n\r\nhello!",
        b"HTTP/1.1 200 OK\r\nContent-Length: -1\r\n\r\n",
        b"HTTP/1.1 200 OK\r\nContent-Length: 67108865\r\n\r\n",
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\nzz\r\n",
        b"HTTP/1.1 2000 OK\r\n\r\n",
    ] {
        let leaked: &'static [u8] = Box::leak(bad.to_vec().into_boxed_slice());
        assert_eq!(kind(&get_plain(leaked, "close")), Some(ErrorKind::InvalidData), "{}", String::from_utf8_lossy(bad));
    }
    assert_eq!(get_plain(b"HTTP/1.1 200 OK\r\nContent-Length: 5, 5\r\n\r\nhello", "close").unwrap(), b"hello");
    assert_eq!(
        get_plain(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: Chunked\r\n\r\n3;x=y\r\nabc\r\n2\r\nde\r\n0\r\nT: v\r\n\r\n", "hold").unwrap(),
        b"abcde"
    );
    assert_eq!(get_plain(b"HTTP/1.0 200 OK\r\n\r\nto close", "close").unwrap(), b"to close");
    assert!(matches!(get_plain(b"HTTP/1.1 200 OK\r\nContent-Length: 9\r\n\r\nshort", "close"), Err(NetError::Reset)));
    assert!(matches!(get_plain(b"HTTP/1.1 301 Moved\r\nLocation: /x\r\n\r\n", "close"), Err(NetError::Status(301))));
    assert!(matches!(get_plain(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n", "close"), Err(NetError::Status(404))));
}
#[test]
fn real_get_bounds_bodies_deadlines_and_connections() {
    for (len, ok) in [(BODY_LIMIT, true), (BODY_LIMIT + 1, false)] {
        for framed in [true, false] {
            let (port, server) = serve(move |mut s, _| {
                let head = if framed { format!("HTTP/1.1 200 OK\r\nContent-Length: {len}\r\n\r\n") } else { "HTTP/1.1 200 OK\r\n\r\n".into() };
                let _ = s.write_all(head.as_bytes());
                let _ = s.write_all(&vec![b'x'; len]);
            });
            let r = Net::new(None).get(&url(&format!("http://127.0.0.1:{port}/")), real_deadline(Duration::from_secs(60)));
            assert_eq!(r.as_ref().map(Vec::len).ok(), ok.then_some(len), "{len} {framed}");
            if !ok {
                assert_eq!(kind(&r), Some(ErrorKind::InvalidData));
            }
            let _ = server.join();
        }
    }
    let (port, _server) = serve(plain(b"", "hold"));
    let start = std::time::Instant::now();
    let r = Net::new(None).get(&url(&format!("http://127.0.0.1:{port}/")), real_deadline(Duration::from_millis(300)));
    assert!(matches!(r, Err(NetError::Timeout)), "{r:?}");
    assert!(start.elapsed() < Duration::from_secs(2));
    assert!(matches!(Net::new(None).get(&url("http://127.0.0.1:9/"), real_deadline(Duration::ZERO)), Err(NetError::Timeout)));
    let closed = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap();
    assert!(matches!(Net::new(None).get(&url(&format!("http://{closed}/")), real_deadline(Duration::from_secs(5))), Err(NetError::Refused)));
    // Mixed attempts: the first non-refusal error; all refused: Refused.
    let c = Network::new(None).context();
    let span = std::panic::Location::caller();
    let link_local: SocketAddr = "[fe80::1]:80".parse().unwrap(); // no scope: EINVAL, not refusal
    let until = crate::time::monotonic() + Duration::from_secs(5);
    let r = http::dial(&c, vec![closed, link_local, closed], span, until);
    assert!(matches!(&r, Err(NetError::Io(e)) if e.kind() != ErrorKind::ConnectionRefused), "{r:?}");
    assert!(matches!(http::dial(&c, vec![closed, closed], span, until), Err(NetError::Refused)));
}

fn assert_get_cancelled(
    net: Net,
    url: Url,
    equal_deadlines: bool,
    request: std::sync::mpsc::Receiver<String>,
) {
    use crate::task::{TaskError, Tasks};

    let start = std::time::Instant::now();
    let result = Tasks::real().scope::<_, TaskError, _>(|scope| {
        let now = crate::time::monotonic();
        let scope_deadline = crate::time::Instant(now + Duration::from_millis(500));
        let get_deadline = if equal_deadlines {
            scope_deadline
        } else {
            crate::time::Instant(now + Duration::from_secs(10))
        };
        scope.set_deadline(scope_deadline);
        Ok(net.get(&url, get_deadline))
    }).unwrap();
    let elapsed = start.elapsed();
    let head = request.recv_timeout(Duration::from_secs(3)).unwrap();
    assert!(head.ends_with("\r\n\r\n"), "{head:?}");
    assert!(matches!(result, Err(NetError::Cancelled)), "{result:?}");
    assert!(elapsed < Duration::from_secs(3), "{elapsed:?}");
}

fn real_get_cancelled(equal_deadlines: bool) {
    // The zero-capacity channel keeps the peer open until the caller has a result.
    let (sent, request) = std::sync::mpsc::sync_channel(0);
    let (port, server) = serve(move |_socket, head| {
        let _ = sent.send(head);
    });
    assert_get_cancelled(Net::new(None), url(&format!("http://127.0.0.1:{port}/")), equal_deadlines, request);
    server.join().unwrap();
}

#[test]
fn real_get_scope_deadline_cancels_a_pending_response() {
    real_get_cancelled(false);
}

#[test]
fn real_get_scope_deadline_wins_at_the_same_instant() {
    real_get_cancelled(true);
}

type TlsStream = rustls::StreamOwned<rustls::ServerConnection, std::net::TcpStream>;

fn serve_tls<T: Send + 'static>(
    host: &str,
    cert: &'static [u8],
    respond: impl FnOnce(TlsStream, String) -> T + Send + 'static,
) -> (u16, std::thread::JoinHandle<T>) {
    let listener = std::net::TcpListener::bind((host, 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let config = rustls::ServerConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(
            vec![cert.to_vec().into()],
            rustls::pki_types::PrivateKeyDer::Pkcs8(KEY.to_vec().into()),
        )
        .unwrap();
    let server = rustls::ServerConnection::new(Arc::new(config)).unwrap();
    let handle = std::thread::spawn(move || {
        let (tcp, _) = listener.accept().unwrap();
        let mut tls = rustls::StreamOwned::new(server, tcp);
        let head = read_head(&mut tls);
        respond(tls, head)
    });
    (port, handle)
}

/// A TLS server on loopback; `close` decides how a close-delimited body ends.
fn tls_serve(cert: &'static [u8], body: &'static [u8], close: &'static str) -> (u16, std::thread::JoinHandle<(Option<String>, String)>) {
    serve_tls("127.0.0.1", cert, move |mut tls, head| {
        let sni = tls.conn.server_name().map(str::to_owned);
        if head.is_empty() {
            return (sni, head);
        }
        let _ = tls.write_all(body);
        let _ = tls.flush();
        if close == "notify" {
            tls.conn.send_close_notify();
            let _ = tls.conn.complete_io(&mut tls.sock);
        }
        (sni, head)
    })
}

#[test]
fn tls_get_scope_deadline_cancels_a_pending_response() {
    let (sent, request) = std::sync::mpsc::sync_channel(0);
    let (port, server) = serve_tls("127.0.0.1", SERVER, move |_tls, head| {
        let _ = sent.send(head);
    });
    let net = Net::new(None).with_test_root(ROOT);
    assert_get_cancelled(net, url(&format!("https://127.0.0.1:{port}/")), false, request);
    server.join().unwrap();
}
#[test]
fn tls_get_verifies_frames_and_needs_close_notify() {
    let net = Net::new(None).with_test_root(ROOT);
    let get = |host: &str, port: u16| net.get(&url(&format!("https://{host}:{port}/x")), real_deadline(Duration::from_secs(10)));
    let framed = b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nhello";
    let (port, server) = tls_serve(SERVER, framed, "bare");
    assert_eq!(get("localhost", port).unwrap(), b"hello");
    let (sni, head) = server.join().unwrap();
    assert_eq!(sni.as_deref(), Some("localhost"));
    assert!(head.contains(&format!("\r\nHost: localhost:{port}\r\n")), "{head}");
    let (port, server) = tls_serve(SERVER, b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n2\r\nhi\r\n0\r\n\r\n", "bare");
    assert_eq!(get("127.0.0.1", port).unwrap(), b"hi");
    assert_eq!(server.join().unwrap().0, None, "no SNI for an IP host");
    let delimited = b"HTTP/1.1 200 OK\r\n\r\nto close";
    let (port, server) = tls_serve(SERVER, delimited, "notify");
    assert_eq!(get("localhost", port).unwrap(), b"to close");
    server.join().unwrap();
    let (port, server) = tls_serve(SERVER, delimited, "bare");
    assert!(matches!(get("localhost", port), Err(NetError::Reset)));
    server.join().unwrap();
    // An untrusted certificate fails before any HTTP data is sent.
    let (port, server) = tls_serve(UNTRUSTED, framed, "bare");
    assert_eq!(kind(&get("localhost", port)), Some(ErrorKind::InvalidData));
    assert_eq!(server.join().unwrap().1, "");
    // Compiled-in roots do not trust the test root.
    let (port, server) = tls_serve(SERVER, framed, "bare");
    let r = Net::new(None).get(&url(&format!("https://localhost:{port}/")), real_deadline(Duration::from_secs(10)));
    assert_eq!(kind(&r), Some(ErrorKind::InvalidData));
    server.join().unwrap();
}

#[test]
fn tls_get_rejects_an_ip_outside_the_certificate_sans() {
    let (port, server) = serve_tls("127.0.0.2", SERVER, |_, head| head);
    let net = Net::new(None).with_test_root(ROOT);
    let dest = url(&format!("https://127.0.0.2:{port}/"));
    let result = net.get(&dest, real_deadline(Duration::from_secs(10)));
    assert_eq!(kind(&result), Some(ErrorKind::InvalidData), "{result:?}");
    assert_eq!(server.join().unwrap(), "");
}

/// Proxy, trust-store and key-log variables change nothing: the TLS test reruns in a child with
/// them set, and no key-log file appears.
#[test]
fn environment_changes_nothing() {
    let dir = std::env::temp_dir().join(format!("tide-keylog-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let keylog = dir.join("keys.log");
    let out = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "net::http_tests::tls_get_verifies_frames_and_needs_close_notify", "--test-threads=1"])
        .envs(["HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY", "http_proxy", "https_proxy", "all_proxy"].map(|k| (k, "http://127.0.0.1:9")))
        .env("NO_PROXY", "")
        .env("SSL_CERT_FILE", dir.join("none.pem"))
        .env("SSL_CERT_DIR", &dir)
        .env("SSLKEYLOGFILE", &keylog)
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&out.stdout).contains("1 passed"), "{}", String::from_utf8_lossy(&out.stdout));
    assert!(!keylog.exists());
    let _ = std::fs::remove_dir(&dir);
}

// NET-HTTP-3: simulated peers.
fn sim_deadline(sim: &crate::sim::SimRoot, d: Duration) -> crate::time::Instant {
    sim.clock.now() + d
}
#[test]
fn simulated_get_follows_the_peer_timeline() {
    use crate::sim::{Reply, SimRoot};
    let sim = SimRoot::new(3);
    let calls = Arc::new(AtomicUsize::new(0));
    let seen = calls.clone();
    sim.world.http("fast.test:443", move |req| {
        seen.fetch_add(1, Ordering::SeqCst);
        assert_eq!(req.url().as_str(), "https://fast.test/p?q");
        Reply::ok("hello").after(Duration::from_millis(20))
    });
    sim.world.http("127.0.0.1:8080", |req| match req.url().path() {
        "/204" => Reply::status(204),
        "/404" => Reply::status(404),
        "/reset" => Reply::reset(),
        "/tls" => Reply::tls_error(),
        "/hang" => Reply::hang(),
        "/max" => Reply::ok(vec![0; BODY_LIMIT]),
        _ => Reply::ok(vec![0; BODY_LIMIT + 1]),
    });
    let net = &sim.net;
    let get = |u: &str, d: u64| net.get(&url(u), sim_deadline(&sim, Duration::from_secs(d)));
    assert_eq!(get("https://fast.test/p?q#secret", 5).unwrap(), b"hello");
    let t = sim.elapsed();
    assert!(t >= Duration::from_millis(22) && t <= Duration::from_millis(40), "{t:?}");
    assert_eq!(get("http://127.0.0.1:8080/204", 5).unwrap(), b"");
    assert!(matches!(get("http://127.0.0.1:8080/404", 5), Err(NetError::Status(404))));
    assert!(matches!(get("http://127.0.0.1:8080/reset", 5), Err(NetError::Reset)));
    assert_eq!(kind(&get("http://127.0.0.1:8080/tls", 5)), Some(ErrorKind::InvalidData));
    assert_eq!(get("http://127.0.0.1:8080/max", 5).unwrap().len(), BODY_LIMIT);
    assert_eq!(kind(&get("http://127.0.0.1:8080/over", 5)), Some(ErrorKind::InvalidData));
    let before = sim.elapsed();
    assert!(matches!(get("http://127.0.0.1:8080/hang", 2), Err(NetError::Timeout)));
    assert_eq!(sim.elapsed() - before, Duration::from_secs(2), "Timeout exactly at the deadline");
    // Denied before any peer; Dns at once; Refused after one round trip, even with a listener.
    assert!(matches!(net.only(["other.test:443"]).get(&url("https://fast.test/"), sim_deadline(&sim, Duration::from_secs(5))), Err(NetError::Denied)));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    let before = sim.elapsed();
    assert!(matches!(get("https://unknown.test/", 5), Err(NetError::Dns)));
    assert_eq!(sim.elapsed(), before);
    let _listener = net.bind("127.0.0.1:7000").unwrap();
    for refused in ["http://127.0.0.1:7000/", "http://localhost:7001/", "https://fast.test:8443/"] {
        let before = sim.elapsed();
        assert!(matches!(get(refused, 5), Err(NetError::Refused)), "{refused}");
        let rtt = sim.elapsed() - before;
        assert!(rtt >= Duration::from_millis(2) && rtt <= Duration::from_millis(20), "{rtt:?}");
    }
    assert!(!sim.trace().contains("net.accept"));
    assert!(!sim.trace().contains("secret") && !sim.trace().contains("/p?q"));
    // Setup: listener-held, duplicate and invalid addresses panic; peers reserve their port.
    for addr in ["127.0.0.1:7000", "localhost:7000", "0.0.0.0:7000", "fast.test:443", "fast.test", "a..b:1"] {
        assert!(catch_unwind(AssertUnwindSafe(|| sim.world.http(addr, |_| Reply::hang()))).is_err(), "{addr}");
    }
    sim.world.http("localhost:9090", |_| Reply::hang());
    for bind in ["127.0.0.1:9090", "0.0.0.0:9090"] {
        assert_eq!(net.bind(bind).unwrap_err().kind(), ErrorKind::AddrInUse, "{bind}");
    }
    for code in [100, 101, 199, 600] {
        assert!(catch_unwind(|| Reply::status(code)).is_err(), "{code}");
    }
}
#[test]
fn simulated_get_cancellation_wins_and_replays() {
    use crate::sim::{Reply, SimRoot};
    let sim = SimRoot::new(4);
    sim.world.http("slow.test:80", |_| Reply::hang());
    let u = url("http://slow.test/");
    let r = sim.tasks.scope::<_, crate::task::TaskError, _>(|s| {
        s.cancel();
        Ok(sim.net.get(&u, sim_deadline(&sim, Duration::from_secs(1))))
    });
    assert!(matches!(r, Ok(Err(NetError::Cancelled))), "{r:?}");
    let r = sim.tasks.scope::<_, crate::task::TaskError, _>(|s| {
        let at = sim_deadline(&sim, Duration::from_secs(1));
        s.set_deadline(at);
        Ok(sim.net.get(&u, at))
    });
    assert!(matches!(r, Ok(Err(NetError::Cancelled))), "{r:?}");
    drop(sim);
    let run = |seed| {
        let sim = SimRoot::new(seed);
        sim.world.http("a.test:443", |_| Reply::ok("x").after(Duration::from_millis(5)));
        let r = sim.net.get(&url("https://a.test/secretpath"), sim_deadline(&sim, Duration::from_secs(1)));
        assert_eq!(r.unwrap(), b"x");
        sim.trace()
    };
    let trace = run(9);
    assert_eq!(trace, run(9));
    assert!(trace.contains("net.get") && trace.contains("world.http") && !trace.contains("secretpath"), "{trace}");
}

#[test]
fn simulated_peer_callbacks_can_schedule_and_cancel() {
    use crate::sim::Reply;
    for seed in 0..8 {
        let sim = crate::Root::sim(seed);
        let clock = sim.clock.clone();
        sim.world.http("a.test:80", move |_| {
            clock.sleep(Duration::from_millis(20)).unwrap();
            Reply::ok("x")
        });
        let u = url("http://a.test/");
        let at = sim_deadline(&sim, Duration::from_secs(1));
        sim.tasks.scope::<_, crate::task::TaskError, _>(|s| {
            let a = s.spawn(|| Ok(sim.net.get(&u, at).unwrap()));
            let b = s.spawn(|| Ok(sim.net.get(&u, at).unwrap()));
            assert_eq!(a.join()?, b"x");
            assert_eq!(b.join()?, b"x");
            Ok(())
        }).unwrap();
        // The peer returns a cached reply after its scope deadline has cancelled the task.
        let clock = sim.clock.clone();
        sim.world.http("b.test:80", move |_| {
            assert!(clock.sleep(Duration::from_secs(1)).is_err());
            Reply::ok("must not escape cancellation")
        });
        sim.tasks.scope::<_, crate::task::TaskError, _>(|s| {
            s.set_deadline(sim_deadline(&sim, Duration::from_millis(30)));
            assert!(matches!(sim.net.get(&url("http://b.test/"), at), Err(NetError::Cancelled)));
            Ok(())
        }).unwrap();
    }
}

#[test]
fn real_retry_checks_deadline_before_ready_io() {
    let c = Network::new(None).context();
    let s = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let result: io::Result<()> = tcp_real::retry(&c, &s, false, std::panic::Location::caller(), Some(Duration::ZERO), || {
        panic!("expired calls must not attempt I/O")
    });
    assert_eq!(result.unwrap_err().kind(), ErrorKind::TimedOut);
}

#[test]
fn tls_get_sends_a_request_larger_than_the_plaintext_buffer() {
    let (port, server) = tls_serve(SERVER, b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok", "bare");
    let path = format!("/{}", "x".repeat(128 << 10));
    let u = url(&format!("https://localhost:{port}{path}"));
    let result = Net::new(None).with_test_root(ROOT).get(&u, real_deadline(Duration::from_secs(10)));
    let (_, head) = server.join().unwrap();
    assert_eq!(result.unwrap(), b"ok");
    assert!(head.starts_with(&format!("GET {path} HTTP/1.1\r\n")));
}

#[test]
fn real_get_keeps_opaque_headers_and_rejects_non_ascii_framing() {
    assert_eq!(get_plain(b"HTTP/1.1 200 OK\r\nX-Note: \xff\r\nContent-Length: 2\r\n\r\nok", "close").unwrap(), b"ok");
    for response in [
        &b"HTTP/1.1 200 OK\r\nContent-Length: \xc2\xa05\r\n\r\nhello"[..],
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: \xc2\xa0chunked\r\n\r\n0\r\n\r\n",
    ] {
        assert_eq!(kind(&get_plain(Box::leak(response.to_vec().into_boxed_slice()), "close")), Some(ErrorKind::InvalidData));
    }
    assert_eq!(get_plain(b"HTTP/1.1 200 OK\r\nContent-Length: \t2 \t, \t2\r\n\r\nok", "close").unwrap(), b"ok");
}

#[test]
fn real_get_rejects_malformed_chunked_trailers() {
    for response in [
        &b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n0\r\nnot-a-header\r\n\r\n"[..],
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n0\r\nX: \0\r\n\r\n",
    ] {
        assert_eq!(kind(&get_plain(Box::leak(response.to_vec().into_boxed_slice()), "close")), Some(ErrorKind::InvalidData));
    }
}

#[test]
fn non_success_responses_still_validate_the_body() {
    assert!(matches!(get_plain(b"HTTP/1.1 404 Missing\r\nContent-Length: 5\r\n\r\nab", "close"), Err(NetError::Reset)));
    assert_eq!(kind(&get_plain(b"HTTP/1.1 404 Missing\r\nContent-Length: 67108865\r\n\r\n", "close")), Some(ErrorKind::InvalidData));
    let net = Net::new(None).with_test_root(ROOT);
    for (close, reset) in [("bare", true), ("notify", false)] {
        let (port, server) = tls_serve(SERVER, b"HTTP/1.1 404 Missing\r\n\r\nerror body", close);
        let result = net.get(&url(&format!("https://localhost:{port}/")), real_deadline(Duration::from_secs(10)));
        server.join().unwrap();
        assert!(if reset { matches!(result, Err(NetError::Reset)) } else { matches!(result, Err(NetError::Status(404))) }, "{result:?}");
    }
}

#[test]
fn simulated_peer_waits_obey_the_get_deadline_without_cancelling_the_scope() {
    use crate::sim::Reply;
    for seed in 0..8 {
        let sim = crate::Root::sim(seed);
        let clock = sim.clock.clone();
        sim.world.http("sleep.test:80", move |_| {
            clock.sleep(Duration::from_secs(1)).unwrap();
            Reply::ok("too late")
        });
        let (_tx, rx) = crate::sync::mpsc::sync_channel::<u8>(1, &sim.tasks);
        sim.world.http("wait.test:80", move |_| {
            rx.recv().unwrap();
            Reply::ok("never ready")
        });
        let calls = AtomicUsize::new(0);
        sim.world.http("panic.test:80", move |_| {
            if calls.fetch_add(1, Ordering::SeqCst) == 0 { panic!("peer panic"); }
            Reply::ok("restored")
        });
        sim.tasks.scope::<_, crate::task::TaskError, _>(|_| {
            for peer in ["sleep.test", "wait.test"] {
                let at = sim_deadline(&sim, Duration::from_millis(30));
                assert!(matches!(sim.net.get(&url(&format!("http://{peer}/")), at), Err(NetError::Timeout)));
                assert_eq!(sim.clock.now(), at);
                sim.tasks.checkpoint().unwrap();
            }
            let at = sim_deadline(&sim, Duration::from_secs(1));
            let u = url("http://panic.test/");
            assert!(catch_unwind(AssertUnwindSafe(|| sim.net.get(&u, at))).is_err());
            assert_eq!(sim.net.get(&u, at).unwrap(), b"restored");
            sim.tasks.checkpoint().unwrap();
            Ok(())
        }).unwrap();
    }
}

#[test]
fn simulated_peers_release_captured_capabilities_on_root_exit_and_unwind() {
    use crate::sim::Reply;
    for panic in [false, true] {
        let captured = Arc::new(());
        let weak = Arc::downgrade(&captured);
        let result = catch_unwind(AssertUnwindSafe(move || {
            let sim = crate::Root::sim(3);
            let net = sim.net.clone();
            sim.world.http("forward.test:80", move |_| {
                let _ = (&net, &captured);
                Reply::ok("x")
            });
            if panic { panic!("root unwind"); }
        }));
        assert_eq!(result.is_err(), panic);
        assert!(weak.upgrade().is_none(), "HTTP callback retained its Net and payload");
    }
}


#[test]
fn real_get_rejects_chunk_size_lines_without_digits() {
    for line in [&b""[..], b";x=y"] {
        let mut response = b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n3\r\nabc\r\n".to_vec();
        response.extend_from_slice(line);
        response.extend_from_slice(b"\r\n\r\n");
        assert_eq!(kind(&get_plain(Box::leak(response.into_boxed_slice()), "close")), Some(ErrorKind::InvalidData));
    }
}

#[test]
fn real_get_validates_chunk_extensions() {
    for line in [
        &b"1;x=\0"[..], b"1;", b"1;=v", b"1;x=", b"1;x=\"unterminated",
        b"1;x=\"bad\0\"", b"1;x=\"bad\\\0\"", b"1;x=\"v\"junk", b"1;x=a/b",
    ] {
        let mut response = b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n".to_vec();
        response.extend_from_slice(line);
        response.extend_from_slice(b"\r\nx\r\n0\r\n\r\n");
        assert_eq!(kind(&get_plain(Box::leak(response.into_boxed_slice()), "close")), Some(ErrorKind::InvalidData), "{line:?}");
    }
    for line in [
        &b"1;x=y"[..], b"1;flag", b"1 \t; \tfoo \t= \tbar;empty=\"\"",
        b"1;x=\"a;\\\"b\\\\c\t\xff\";flag",
    ] {
        let mut response = b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n".to_vec();
        response.extend_from_slice(line);
        response.extend_from_slice(b"\r\nx\r\n0;end=\"ok\"\r\n\r\n");
        assert_eq!(get_plain(Box::leak(response.into_boxed_slice()), "close").unwrap(), b"x", "{line:?}");
    }
}

#[test]
fn real_get_refuses_invalid_status_codes_before_body_framing() {
    for code in ["000", "099", "600", "999"] {
        let response = format!("HTTP/1.1 {code} Invalid\r\nContent-Length: 0\r\n\r\n");
        assert_eq!(kind(&get_plain(Box::leak(response.into_bytes().into_boxed_slice()), "close")), Some(ErrorKind::InvalidData), "{code}");
    }
    let start = std::time::Instant::now();
    assert_eq!(kind(&get_plain(b"HTTP/1.1 600 Invalid\r\n\r\n", "hold")), Some(ErrorKind::InvalidData));
    assert!(start.elapsed() < Duration::from_secs(2), "invalid status waited for a body");
    assert!(matches!(get_plain(b"HTTP/1.1 599 Unrecognized\r\nContent-Length: 0\r\n\r\n", "close"), Err(NetError::Status(599))));
    assert_eq!(get_plain(b"HTTP/1.1 299 Unrecognized\r\nContent-Length: 2\r\n\r\nok", "close").unwrap(), b"ok");
}

#[test]
fn simulated_unrepresentable_reply_arrivals_time_out() {
    use crate::sim::Reply;
    for answer in 0..4 {
        let sim = crate::Root::sim(0);
        sim.world.http("late.test:80", move |_| match answer {
            0 => Reply::ok("late"),
            1 => Reply::status(404),
            2 => Reply::reset(),
            _ => Reply::hang(),
        }.after(Duration::MAX));
        let at = sim_deadline(&sim, Duration::MAX);
        let result = sim.net.get(&url("http://late.test/"), at);
        assert!(matches!(result, Err(NetError::Timeout)), "{answer}: {result:?}");
        assert_eq!(sim.clock.now(), at);
    }
}

#[test]
fn simulated_packet_arrival_overflow_obeys_deadlines_and_cancellation() {
    use crate::sim::Reply;
    // Inspect the net sub-stream in a disposable root to place the *reply* overflow exactly;
    // sleeping in a fresh same-seed root does not consume packet-delay draws.
    let request_delay = {
        let sim = crate::Root::sim(0);
        sim.net.network.context().latency()
    };
    for (remaining, peer, cancel) in [
        (Duration::from_nanos(1), true, false),
        (request_delay + Duration::from_nanos(1), false, false),
        (Duration::from_nanos(1), true, true),
    ] {
        let sim = crate::Root::sim(0);
        if peer { sim.world.http("127.0.0.1:80", |_| Reply::ok("late")); }
        let at = sim_deadline(&sim, Duration::MAX);
        sim.clock.sleep_until(at - remaining).unwrap();
        sim.tasks.scope::<_, crate::task::TaskError, _>(|scope| {
            if cancel { scope.set_deadline(at); }
            let result = sim.net.get(&url("http://127.0.0.1/"), at);
            assert!(if cancel { matches!(result, Err(NetError::Cancelled)) } else { matches!(result, Err(NetError::Timeout)) }, "{result:?}");
            Ok(())
        }).unwrap();
        assert_eq!(sim.clock.now(), at);
    }
    let sim = crate::Root::sim(0);
    sim.world.http("fast.test:80", |_| Reply::ok("on time").after(Duration::from_millis(1)));
    assert_eq!(sim.net.get(&url("http://fast.test/"), sim_deadline(&sim, Duration::from_secs(1))).unwrap(), b"on time");
}
