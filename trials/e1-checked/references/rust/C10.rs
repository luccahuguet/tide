use std::{
    io::{self, BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    thread,
};
fn answer(stream: TcpStream) -> io::Result<u64> {
    let mut reader = BufReader::new(&stream);
    let mut writer = &stream;
    let mut count = 0;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            break;
        }
        if !line.ends_with('\n') {
            break;
        }
        line.pop();
        if line == "QUIT" {
            break;
        }
        writeln!(writer, "{}", line.chars().rev().collect::<String>())?;
        count += 1;
    }
    Ok(count)
}
pub fn serve(listener: &TcpListener, clients: usize) -> io::Result<u64> {
    thread::scope(|s| {
        let mut hs = Vec::new();
        for _ in 0..clients {
            let (stream, _) = listener.accept()?;
            hs.push(s.spawn(move || answer(stream)));
        }
        hs.into_iter()
            .try_fold(0, |n, h| Ok(n + h.join().unwrap()?))
    })
}
pub fn exchange(addr: &str, lines: &[&str]) -> io::Result<Vec<String>> {
    let stream = TcpStream::connect(addr)?;
    let mut writer = &stream;
    let mut reader = BufReader::new(&stream);
    let mut out = Vec::new();
    for line in lines {
        writeln!(writer, "{line}")?;
        let mut reply = String::new();
        reader.read_line(&mut reply)?;
        if reply.ends_with('\n') {
            reply.pop();
        }
        out.push(reply);
    }
    writeln!(writer, "QUIT")?;
    Ok(out)
}
