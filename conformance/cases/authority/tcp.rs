//@ rules: CAP-1, TCP-1
// One value/ErrorKind suite runs unchanged against production real and simulated TCP.
use std::io::{ErrorKind, Read, Write};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, Net, Shutdown, SocketAddr, TcpListener, TcpStream};
use std::task::{TaskError, Tasks};
use std::time::{Clock, Duration};

fn payload(len: usize) -> Vec<u8> {
    (0..len)
        .map(|i| {
            let position = u64::try_from(i).unwrap();
            let mixed = position.wrapping_mul(0x9e3779b97f4a7c15);
            u8::try_from((mixed ^ (mixed >> 29) ^ (position >> 16)) & 255).unwrap()
        })
        .collect()
}

fn suite(net: &Net, tasks: &Tasks, clock: &Clock) -> Vec<ErrorKind> {
    let mut observations = Vec::new();
    observations.push(net.no_bind().bind("127.0.0.1:0").unwrap_err().kind());
    observations.push(
        net.only(["127.0.0.1:1"])
            .connect("127.0.0.1:2")
            .unwrap_err()
            .kind(),
    );
    observations.push(
        net.only(["LOCALHOST:1"])
            .only(["localhost:2"])
            .connect(("localhost", 1))
            .unwrap_err()
            .kind(),
    );
    observations.push(net.bind(":7000").unwrap_err().kind());
    observations.push(net.connect("missing-port").unwrap_err().kind());
    assert_eq!(
        observations,
        vec![
            ErrorKind::PermissionDenied,
            ErrorKind::PermissionDenied,
            ErrorKind::PermissionDenied,
            ErrorKind::InvalidInput,
            ErrorKind::InvalidInput
        ]
    );
    let listener: TcpListener = net
        .only(["unrelated.invalid:1"])
        .bind((Ipv4Addr::LOCALHOST, 0))
        .unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    assert_eq!(addr.ip(), IpAddr::V4(Ipv4Addr::LOCALHOST));
    assert_ne!(addr.port(), 0);
    let v6: IpAddr = Ipv6Addr::LOCALHOST.into();
    assert!(v6.is_ipv6());
    tasks
        .scope::<_, TaskError, _>(|s| {
            s.spawn(|| {
                let (mut socket, peer) = listener.accept().unwrap();
                assert_eq!(socket.local_addr().unwrap(), addr);
                assert_eq!(socket.peer_addr().unwrap(), peer);
                assert_eq!(peer.ip(), addr.ip());
                let mut data = Vec::new();
                socket.read_to_end(&mut data).unwrap();
                assert_eq!(data, payload(1_048_576));
                socket.write_all(b"done").unwrap();
                socket.flush().unwrap();
                socket.shutdown(Shutdown::Write).unwrap();
                if let Err(e) = socket.shutdown(Shutdown::Write) {
                    assert_eq!(e.kind(), ErrorKind::NotConnected);
                }
                Ok(())
            });
            s.spawn(|| {
                let socket: TcpStream = net
                    .only([format!("LOCALHOST:{}", addr.port())])
                    .connect(("localhost", addr.port()))
                    .unwrap();
                assert_eq!(socket.peer_addr().unwrap(), addr);
                assert_eq!(socket.local_addr().unwrap().ip(), addr.ip());
                let mut borrowed = &socket;
                borrowed.write_all(&payload(1_048_576)).unwrap();
                borrowed.flush().unwrap();
                socket.shutdown(Shutdown::Write).unwrap();
                let mut reply = [0; 4];
                borrowed.read_exact(&mut reply).unwrap();
                assert_eq!(&reply, b"done");
                assert_eq!(borrowed.read(&mut [0; 1]).unwrap(), 0);
                Ok(())
            });
            Ok(())
        })
        .unwrap();

    // Connect completes before accept; both address argument forms and incoming execute.
    let mut client = net.connect(&addr).unwrap();
    let server = listener.incoming().next().unwrap().unwrap();
    tasks
        .scope::<_, TaskError, _>(|s| {
            s.set_deadline(clock.now() + Duration::from_secs(1));
            assert_eq!((&server).read(&mut []).unwrap(), 0);
            Ok(())
        })
        .unwrap();
    client.write_all(b"kept").unwrap();
    tasks
        .scope::<_, TaskError, _>(|s| {
            s.cancel();
            assert!(net.no_bind().bind(":bad").unwrap_err().is_cancelled());
            assert!(net.connect("bad").unwrap_err().is_cancelled());
            assert!(listener.accept().unwrap_err().is_cancelled());
            let mut incoming = listener.incoming();
            assert!(incoming.next().unwrap().unwrap_err().is_cancelled());
            assert!(incoming.next().is_none());
            assert!(incoming.next().is_none());
            let mut borrowed = &server;
            assert!(borrowed.read(&mut []).unwrap_err().is_cancelled());
            assert!(borrowed.read(&mut [0; 4]).unwrap_err().is_cancelled());
            assert!(borrowed.write(b"lost").unwrap_err().is_cancelled());
            assert!(server.shutdown(Shutdown::Both).unwrap_err().is_cancelled());
            assert_eq!(server.local_addr().unwrap(), addr);
            assert_eq!(client.peer_addr().unwrap(), addr);
            borrowed.flush().unwrap();
            tasks.shield(|| {
                let mut data = [0; 4];
                borrowed.read_exact(&mut data).unwrap();
                assert_eq!(&data, b"kept");
            });
            Ok(())
        })
        .unwrap();
    tasks
        .scope::<_, TaskError, _>(|s| {
            s.set_deadline(clock.now() + Duration::from_millis(2));
            assert!((&server).read(&mut [0; 1]).unwrap_err().is_cancelled());
            Ok(())
        })
        .unwrap();
    server.shutdown(Shutdown::Read).unwrap();
    assert_eq!((&server).read(&mut [0; 1]).unwrap(), 0);
    client.write_all(b"discarded").unwrap();
    drop(server);
    // Linux may accept a first write after close, or report either of these errors.
    match client.write(b"after close") {
        Ok(_) => {}
        Err(e) => assert!(matches!(
            e.kind(),
            ErrorKind::BrokenPipe | ErrorKind::ConnectionReset
        )),
    }
    drop(client);

    // Only the harness timeout bounds a broken wake; no deadline can rescue cancellation.
    let result: Result<(), TaskError> = tasks.scope::<_, TaskError, _>(|s| {
        s.spawn(|| {
            assert_eq!(listener.accept().unwrap_err().kind(), ErrorKind::Cancelled);
            Ok(())
        });
        clock.sleep(Duration::from_millis(20)).unwrap();
        Err(TaskError::Cancelled)
    });
    assert!(result.is_err());
    tasks
        .scope::<_, TaskError, _>(|s| {
            s.set_deadline(clock.now() + Duration::from_millis(2));
            assert!(listener.accept().unwrap_err().is_cancelled());
            Ok(())
        })
        .unwrap();
    let doomed = net.bind("127.0.0.1:0").unwrap();
    let closed = doomed.local_addr().unwrap();
    assert!(
        tasks
            .spawn_isolated::<()>(move || {
                let _listener = doomed;
                panic!("tcp listener unwind");
            })
            .is_err()
    );
    assert_eq!(
        net.connect(closed).unwrap_err().kind(),
        ErrorKind::ConnectionRefused
    );
    let doomed = net.connect(addr).unwrap();
    let (mut peer, _) = listener.accept().unwrap();
    tasks
        .scope::<_, TaskError, _>(|s| {
            s.set_deadline(clock.now() + Duration::from_secs(2));
            s.spawn(|| {
                let mut bytes = Vec::new();
                assert_eq!(peer.read_to_end(&mut bytes).unwrap(), 0);
                Ok(())
            });
            clock.sleep(Duration::from_millis(20)).unwrap();
            assert!(
                tasks
                    .spawn_isolated::<()>(move || {
                        let _stream = doomed;
                        panic!("tcp stream unwind");
                    })
                    .is_err()
            );
            Ok(())
        })
        .unwrap();
    drop(listener);
    observations.push(net.connect(addr.to_string()).unwrap_err().kind());
    assert_eq!(observations.last(), Some(&ErrorKind::ConnectionRefused));
    observations
}

fn simulated(seed: u64) -> (Vec<ErrorKind>, String) {
    let sim = Root::sim(seed);
    let values = suite(&sim.net, &sim.tasks, &sim.clock);
    (values, sim.trace())
}

fn simulation_model(seed: u64) {
    let sim = Root::sim(seed);
    let listener = sim.net.bind("0.0.0.0:0").unwrap();
    let bound = listener.local_addr().unwrap();
    assert!(bound.ip().is_unspecified());
    assert!((49152..=65535).contains(&bound.port()));
    assert_eq!(
        sim.net.bind(bound).unwrap_err().kind(),
        ErrorKind::AddrInUse
    );
    let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), bound.port());
    assert_eq!(
        sim.net.connect("192.0.2.1:1").unwrap_err().kind(),
        ErrorKind::ConnectionRefused
    );
    assert_eq!(
        sim.net.connect("absent.invalid:1").unwrap_err().kind(),
        ErrorKind::InvalidInput
    );
    sim.tasks
        .scope::<_, TaskError, _>(|s| {
            s.set_deadline(sim.clock.now() + Duration::from_micros(1));
            assert!(sim.net.connect(addr).unwrap_err().is_cancelled());
            Ok(())
        })
        .unwrap();
    let before = sim.elapsed();
    let mut client = sim.net.connect(addr).unwrap();
    let connect_time = sim.elapsed() - before;
    assert!((Duration::from_millis(2)..=Duration::from_millis(20)).contains(&connect_time));
    let (mut server, _) = listener.accept().unwrap();
    assert_eq!(server.local_addr().unwrap(), addr);
    assert_eq!(client.write(&payload(1_048_576)).unwrap(), 65_536);
    sim.tasks
        .scope::<_, TaskError, _>(|s| {
            s.set_deadline(sim.clock.now() + Duration::from_millis(30));
            assert!(client.write(b"no space").unwrap_err().is_cancelled());
            Ok(())
        })
        .unwrap();
    client.shutdown(Shutdown::Write).unwrap();
    let mut data = Vec::new();
    server.read_to_end(&mut data).unwrap();
    assert_eq!(data, payload(65_536));
    drop(client);
    drop(server);

    let doomed = sim.net.bind("127.0.0.1:0").unwrap();
    let reusable = doomed.local_addr().unwrap();
    assert!(
        sim.tasks
            .spawn_isolated::<()>(move || {
                let _listener = doomed;
                panic!("tcp rebind unwind");
            })
            .is_err()
    );
    let rebound = sim.net.bind(reusable).unwrap();
    assert_eq!(rebound.local_addr().unwrap(), reusable);
    drop(rebound);

    let mut clients = Vec::new();
    for _ in 0..128 {
        clients.push(sim.net.connect(addr).unwrap());
    }
    sim.tasks
        .scope::<_, TaskError, _>(|s| {
            let connector = s.spawn(|| {
                sim.net.connect(addr).unwrap();
                Ok(sim.clock.now())
            });
            sim.clock.sleep(Duration::from_millis(30)).unwrap();
            let accept_at = sim.clock.now();
            let (stream, _) = listener.accept().unwrap();
            assert!(
                connector.join()? >= accept_at,
                "129th connect completed before accept"
            );
            drop(stream);
            Ok(())
        })
        .unwrap();
    drop(listener);
    // Unaccepted connections are reset and the listener port is immediately reusable.
    assert_eq!(
        clients[1].read(&mut [0; 1]).unwrap_err().kind(),
        ErrorKind::ConnectionReset
    );
    let rebound = sim.net.bind(bound).unwrap();
    assert_eq!(rebound.local_addr().unwrap(), bound);
    drop(rebound);
    assert_eq!(
        sim.net.connect(addr).unwrap_err().kind(),
        ErrorKind::ConnectionRefused
    );
}

#[tide::main]
fn main(root: Root) {
    let real = suite(&root.net, &root.tasks, &root.clock);
    for seed in 0..4 {
        let (values, trace) = simulated(seed);
        assert_eq!(real, values);
        assert_eq!(trace, simulated(seed).1);
        assert!(trace.contains("net.connect"));
        assert!(trace.contains("net.read"));
        assert!(!trace.contains("done"));
        simulation_model(seed);
    }
    println!("TCP_RAN");
}

#[test]
fn simulated_tcp() {
    assert_eq!(simulated(7), simulated(7));
    simulation_model(7);
}
