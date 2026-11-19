//@ rules: CAP-1, NET-RESOLVE-1
// One value/NetError suite runs unchanged against production real and simulated `Net::resolve`.
// It sends no live resolver query: every real call is a literal or stops before any lookup.
use std::net::{IpAddr, Net, NetError};
use std::task::{TaskError, Tasks};
use std::time::{Clock, Duration};

fn shown(result: Result<Vec<IpAddr>, NetError>) -> String {
    match result {
        Ok(addrs) => format!("{addrs:?}"),
        Err(NetError::Io(e)) => format!("Io({:?})", e.kind()),
        Err(e) => format!("{e:?}"),
    }
}

fn suite(net: &Net, tasks: &Tasks, clock: &Clock) -> Vec<String> {
    let soon = clock.now() + Duration::from_secs(60);
    let past = clock.now();
    let mut seen = Vec::new();
    // Every NET-HOST spelling of an IP literal, and localhost, with no lookup.
    for host in ["127.0.0.1", "0x7f.1", "[::1]", "::1", "[::ffff:127.0.0.1]", "::ffff:127.0.0.1", "LOCALHOST"] {
        seen.push(shown(net.resolve(host, soon)));
    }
    for host in ["", ".", "a..b", "example.com:443", "http://x/", "b%C3%BCcher.example", "xn--bcher-kva.example", "*.test"] {
        seen.push(shown(net.resolve(host, soon)));
    }
    let named = net.only(["svc.test:443"]);
    let address = net.only(["127.0.0.1:80"]);
    let local = net.only(["localhost:8080"]);
    seen.push(shown(named.resolve("other.test", soon)));
    seen.push(shown(local.resolve("127.0.0.1", soon)));
    seen.push(shown(address.resolve("localhost", soon)));
    seen.push(shown(address.resolve("0x7f.1", soon)));
    seen.push(shown(address.resolve("[::ffff:127.0.0.1]", soon)));
    seen.push(shown(local.resolve("localhost", soon)));
    for host in ["127.0.0.1", "localhost", "svc.test"] {
        seen.push(shown(net.only([""; 0]).resolve(host, soon)));
    }
    // Order witnesses: shape precedes `only`, and `only` precedes the deadline.
    seen.push(shown(net.only([""; 0]).resolve("", past)));
    seen.push(shown(net.only([""; 0]).resolve("svc.test", past)));
    // A port-scoped entry grants its host on any port: past `only`, the passed deadline stops
    // the call before any lookup. The empty chained intersection grants nothing.
    seen.push(shown(named.resolve("SVC.test.", past)));
    seen.push(shown(net.only(["svc.test:80"]).resolve("svc.test", past)));
    seen.push(shown(named.only(["svc.test:80"]).resolve("svc.test", soon)));
    seen.push(shown(net.resolve("127.0.0.1", past)));
    seen.push(shown(net.resolve("svc.test", past)));
    let cancelled = tasks
        .scope::<_, TaskError, _>(|s| {
            s.cancel();
            Ok(shown(net.resolve("svc.test", past)))
        })
        .unwrap();
    seen.push(cancelled);
    let invalid = "Io(InvalidInput)";
    let expected = [
        "[127.0.0.1]", "[127.0.0.1]", "[::1]", "[::1]", "[127.0.0.1]", "[127.0.0.1]", "[127.0.0.1]",
        invalid, invalid, invalid, invalid, invalid, invalid, invalid, invalid,
        "Denied", "Denied", "Denied", "[127.0.0.1]", "[127.0.0.1]", "[127.0.0.1]",
        "Denied", "Denied", "Denied",
        invalid, "Denied",
        "Timeout", "Timeout", "Denied", "Timeout", "Timeout", "Cancelled",
    ];
    assert_eq!(seen, expected);
    seen
}

#[cfg(test)]
fn simulated(seed: u64) -> (Vec<String>, String) {
    let sim = Root::sim(seed);
    let values = suite(&sim.net, &sim.tasks, &sim.clock);
    let soon = sim.clock.now() + Duration::from_secs(60);
    assert_eq!(shown(sim.net.resolve("nothing.invalid", soon)), "Dns");
    (values, sim.trace())
}

#[tide::main]
fn main(root: Root) {
    let real = suite(&root.net, &root.tasks, &root.clock);
    println!("RESOLVE_VALUES:{real:?}");
    println!("RESOLVE_RAN");
}

#[test]
fn simulated_resolve() {
    for seed in 0..4 {
        let (values, trace) = simulated(seed);
        // The trusted harness compares each seed's observations with the real run.
        println!("RESOLVE_VALUES:{values:?}");
        assert_eq!(trace, simulated(seed).1);
        assert_eq!(trace.matches("net.resolve").count(), 33);
    }
}
