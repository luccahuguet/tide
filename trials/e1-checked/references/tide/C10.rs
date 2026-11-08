use std::{
    io::{self, BufRead, BufReader, Write},
    net::{Net, TcpListener, TcpStream},
    task::{TaskError, Tasks},
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
pub fn serve(tasks: &Tasks, listener: &TcpListener, clients: usize) -> io::Result<u64> {
    tasks
        .scope(|s| -> Result<_, TaskError> {
            let mut hs = Vec::new();
            for _ in 0..clients {
                let (stream, _) = match listener.accept() {
                    Ok(pair) => pair,
                    Err(e) => return Ok(Err(e)),
                };
                hs.push(s.spawn(move || Ok(answer(stream))));
            }
            let results = hs
                .into_iter()
                .map(|h| h.join())
                .collect::<Result<Vec<_>, _>>()?;
            Ok(results.into_iter().try_fold(0, |n, r| Ok(n + r?)))
        })
        .unwrap()
}
pub fn exchange(net: &Net, addr: &str, lines: &[&str]) -> io::Result<Vec<String>> {
    let stream = net.connect(addr)?;
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
