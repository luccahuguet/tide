use std::io::{BufRead, BufReader, Write};
use std::task::TaskError;
#[sim_test(seeds = 20)]
fn case1(sim: std::sim::SimRoot) {
    let listener = sim.net.bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    sim.tasks
        .scope(|s| -> Result<(), TaskError> {
            let server = s.spawn(|| Ok(serve(&sim.tasks, &listener, 3)));
            let a = s.spawn(|| Ok(exchange(&sim.net, &addr, &["abc", "héllo"])));
            let b = s.spawn(|| Ok(exchange(&sim.net, &addr, &["x"])));
            let c = s.spawn(|| Ok(exchange(&sim.net, &addr, &[])));
            assert_eq!(a.join()?.unwrap(), vec!["cba", "olléh"]);
            assert_eq!(b.join()?.unwrap(), vec!["x"]);
            assert_eq!(c.join()?.unwrap(), Vec::<String>::new());
            assert_eq!(server.join()?.unwrap(), 3);
            Ok(())
        })
        .unwrap();
}
#[sim_test(seeds = 20)]
fn case2(sim: std::sim::SimRoot) {
    let listener = sim.net.bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    sim.tasks
        .scope(|s| -> Result<(), TaskError> {
            let server = s.spawn(|| Ok(serve(&sim.tasks, &listener, 2)));
            let first = sim.net.connect(&addr).unwrap();
            let mut writer = &first;
            let mut reader = BufReader::new(&first);
            writeln!(writer, "first").unwrap();
            let mut reply = String::new();
            reader.read_line(&mut reply).unwrap();
            assert_eq!(reply, "tsrif\n");
            assert_eq!(
                exchange(&sim.net, &addr, &["second"]).unwrap(),
                vec!["dnoces"]
            );
            writeln!(writer, "QUIT").unwrap();
            assert_eq!(server.join()?.unwrap(), 2);
            Ok(())
        })
        .unwrap();
}
