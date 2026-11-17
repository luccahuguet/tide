// Benchmark pair `tcp`, Rust arm: `tide.rs` in plain Rust, through `std::net` and a thread.
// A server thread binds 127.0.0.1 on a port the system picks and accepts one connection; the
// client sends `count` lines one at a time and waits for each reply. The port is never printed.
// Usage: `tcp [count]`; the default count is the small size the equivalence check runs.
use std::io::{self, BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

const DEFAULT_COUNT: u64 = 200;

#[derive(Debug)]
pub enum RunError {
    Size(String),
    Reply(String),
    Io(io::Error),
    ServerPanicked,
}

impl From<io::Error> for RunError {
    fn from(e: io::Error) -> Self {
        RunError::Io(e)
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

pub fn run(count: u64) -> Result<(), RunError> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let addr = listener.local_addr()?;
    // The server thread owns the listener, so no failure leaves either side waiting forever: a
    // server that fails closes the listener and its stream, which ends the client's reads, and
    // a client that fails returns from `run` without waiting for the server, ending the process.
    let server = std::thread::spawn(move || serve(&listener));
    let (mut replied, mut total) = (0, 0);
    {
        let stream = TcpStream::connect(addr)?;
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
    // The client's stream is closed, so the server sees end of input and returns.
    let served = server.join().map_err(|_| RunError::ServerPanicked)??;
    println!("messages: {count}");
    println!("served: {served}");
    println!("replies: {replied}");
    println!("total: {total}");
    Ok(())
}

fn main() -> Result<(), RunError> {
    let count = match std::env::args().nth(1) {
        Some(arg) => arg.parse().map_err(|_| RunError::Size(arg))?,
        None => DEFAULT_COUNT,
    };
    run(count)
}
