// Benchmark pair `tcp`, Tide arm: TCP on loopback through the `Net` capability. A server task
// binds 127.0.0.1 on a port the system picks and accepts one connection; the client sends
// `count` lines one at a time and waits for each reply. The port is never printed.
// Usage: `tcp [count]`; the default count is the small size the equivalence check runs.
use std::io::{self, BufRead, BufReader, Write};
use std::net::{Net, TcpListener};
use std::task::{TaskError, Tasks};

const DEFAULT_COUNT: u64 = 200;

#[derive(Debug)]
pub enum RunError {
    Size(String),
    Reply(String),
    Io(io::Error),
    Task(TaskError),
}

impl From<io::Error> for RunError {
    fn from(e: io::Error) -> Self {
        RunError::Io(e)
    }
}

impl From<TaskError> for RunError {
    fn from(e: TaskError) -> Self {
        RunError::Task(e)
    }
}

/// The server's reply to `line`: its length and its byte sum. Each message is one write, so
/// the exchange never waits on Nagle's algorithm.
pub fn reply(line: &str) -> String {
    let sum: u64 = line.bytes().map(u64::from).sum();
    format!("{} {sum}\n", line.len())
}

/// Serves one connection until the client closes it; returns the lines it answered.
pub fn serve(listener: &TcpListener) -> Result<u64, RunError> {
    let (stream, _) = listener.accept()?;
    let mut reader = BufReader::new(&stream);
    let mut writer = &stream;
    let (mut lines, mut line) = (0, String::new());
    while reader.read_line(&mut line)? > 0 {
        writer.write_all(reply(line.trim_end()).as_bytes())?;
        lines += 1;
        line.clear();
    }
    Ok(lines)
}

pub fn run(net: &Net, tasks: &Tasks, count: u64) -> Result<(), RunError> {
    let listener = net.bind("127.0.0.1:0")?;
    let addr = listener.local_addr()?;
    let (served, replied, total) = tasks.scope(|s| {
        let server = s.spawn(|| serve(&listener));
        let (mut replied, mut total) = (0, 0);
        {
            let stream = net.connect(addr)?;
            let mut reader = BufReader::new(&stream);
            let mut writer = &stream;
            let mut line = String::new();
            for i in 0..count {
                writer.write_all(format!("message {i}\n").as_bytes())?;
                line.clear();
                if reader.read_line(&mut line)? == 0 {
                    return Err(RunError::Reply("the server closed early".into()));
                }
                let (len, sum) = line
                    .trim_end()
                    .split_once(' ')
                    .ok_or_else(|| RunError::Reply(line.clone()))?;
                let parse = |n: &str| n.parse::<u64>().map_err(|_| RunError::Reply(line.clone()));
                replied += 1;
                total += parse(len)? + parse(sum)?;
            }
        }
        Ok((server.join()?, replied, total))
    })?;
    println!("messages: {count}");
    println!("served: {served}");
    println!("replies: {replied}");
    println!("total: {total}");
    Ok(())
}

#[tide::main]
fn main(root: Root) -> Result<(), RunError> {
    let count = match root.env.args().nth(1) {
        Some(arg) => arg.parse().map_err(|_| RunError::Size(arg))?,
        None => DEFAULT_COUNT,
    };
    run(&root.net, &root.tasks, count)
}
