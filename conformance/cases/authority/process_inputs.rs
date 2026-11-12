//@ rules: CAP-1, ROOT-1, ROOT-3, SINK-1, SINK-2, SINK-3, SINK-4, SINK-6, INPUT-1
//! The same process-input client executes against production and simulated capabilities.
use std::env::{Env, VarError};
use std::io::{self, Read, Stdin, Write};

fn inputs(env: &Env, stdin: &Stdin) -> io::Result<()> {
    let mut args = env.args();
    assert_eq!(args.len(), 3);
    assert!(args.next().is_some());
    assert_eq!(args.next_back().unwrap(), "two words");
    assert_eq!(args.next().unwrap(), "one");
    assert_eq!(env.var("TIDE_INPUT").unwrap(), "value");
    assert!(matches!(
        env.only(["TIDE_INPUT"]).var("OTHER"),
        Err(VarError::NotPresent)
    ));
    assert_eq!(env.only(["TIDE_INPUT"]).only(["OTHER"]).vars().len(), 0);
    assert!(
        env.vars()
            .any(|pair| pair == ("TIDE_INPUT".into(), "value".into()))
    );
    let mut vars = env.only(["TIDE_INPUT"]).vars();
    assert_eq!(
        vars.next_back(),
        Some(("TIDE_INPUT".into(), "value".into()))
    );
    assert_eq!(env.args_only().args().len(), 3);
    assert_eq!(env.args_only().vars().len(), 0);
    assert_eq!(
        env.args_only().current_dir().unwrap_err().kind(),
        io::ErrorKind::PermissionDenied
    );
    assert!(env.current_dir()?.is_absolute());
    assert!(!stdin.is_terminal());
    assert_eq!((&*stdin).read(&mut [])?, 0);
    assert_eq!(stdin.clone().read(&mut [])?, 0);
    let mut line = String::from("prefix:");
    assert_eq!(stdin.read_line(&mut line)?, 4);
    assert_eq!(line, "prefix:one\n");
    let mut lines = stdin.lines();
    assert_eq!(lines.next().unwrap()?, "two");
    drop(lines);
    let mut rest = String::new();
    assert_eq!(stdin.read_to_string(&mut rest)?, 6);
    assert_eq!(rest, "three\n");
    let mut bytes = Vec::new();
    assert_eq!(stdin.read_to_end(&mut bytes)?, 0);
    print!("a");
    println!("b");
    eprintln!("c");
    io::stdout().write_all(b"d\n")?;
    io::stderr().write_all(b"e\n")?;
    info!("ready");
    debug!("detail");
    Ok(())
}

#[tide::main]
fn main(root: Root) -> io::Result<()> {
    inputs(&root.env, &root.stdin)
}

#[test]
fn simulated_inputs() {
    fn run() -> (Vec<std::sim::Record>, String) {
        let sim = Root::sim(7);
        sim.world.args(["program", "one"]);
        sim.world.args(["two words"]);
        sim.world.env_var("TIDE_INPUT", "old");
        sim.world.env_var("TIDE_INPUT", "value");
        sim.world.stdin(b"one\ntwo\r\n");
        sim.world.stdin("three\n");
        inputs(&sim.env, &sim.stdin).unwrap();
        assert_eq!(
            sim.env.current_dir().unwrap(),
            std::path::PathBuf::from("/")
        );
        let records = sim.records();
        assert_eq!(
            records
                .iter()
                .map(|r| r.message.as_str())
                .collect::<Vec<_>>(),
            ["ab", "c", "d", "e", "ready", "detail"]
        );
        assert_eq!(records[0].sink, std::sim::Sink::Out);
        assert_eq!(records[4].sink, std::sim::Sink::Log(std::log::Level::Info));
        assert!(records.iter().all(|r| r.node == "main"));
        assert_eq!(sim.elapsed(), std::time::Duration::ZERO);
        (sim.records(), sim.trace())
    }
    assert_eq!(run(), run());
}

#[test]
fn simulated_stdin_preserves_bad_lines_and_unread_bytes() {
    let sim = Root::sim(9);
    sim.world.stdin(b"a\n\xff\nb\r\ntail");
    let mut lines = sim.stdin.lines();
    assert_eq!(lines.next().unwrap().unwrap(), "a");
    assert_eq!(
        lines.next().unwrap().unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
    assert_eq!(lines.next().unwrap().unwrap(), "b");
    drop(lines);
    let mut rest = vec![b'x'];
    assert_eq!(sim.stdin.read_to_end(&mut rest).unwrap(), 4);
    assert_eq!(rest, b"xtail");
    assert!(sim.stdin.lines().next().is_none());
    assert_eq!(sim.env.args().len(), 0);
    assert_eq!(sim.env.vars().len(), 0);
    error!("error");
    warn!("warning");
    trace!("trace");
    assert_eq!(
        sim.records()
            .iter()
            .map(|r| r.sink.clone())
            .collect::<Vec<_>>(),
        [
            std::sim::Sink::Log(std::log::Level::Error),
            std::sim::Sink::Log(std::log::Level::Warn),
            std::sim::Sink::Log(std::log::Level::Trace),
        ]
    );
}
