// Shared production/conformance process streams. Rust owns the real global stdin lock/buffer.
crate::capability!(
    /// Authority to read standard input; cancellation is checked at entry.
    Stdin
);

pub(crate) fn from_std(error: std::io::Error) -> Error {
    if let Some(code) = error.raw_os_error() {
        Error::from_raw_os_error(code)
    } else {
        Error::new(ErrorKind::from_rust(error.kind()), error.to_string())
    }
}

impl Stdin {
    fn check(&self) -> Result<()> {
        match &self.0 {
            Some(tasks) => tasks.check_simulation(),
            None => crate::task::outside_simulation(),
        }
        if crate::task::observe_cancellation() {
            Err(ErrorKind::Cancelled.into())
        } else {
            Ok(())
        }
    }
    #[track_caller]
    fn line_bytes(&self) -> Result<Vec<u8>> {
        self.check()?;
        match &self.0 {
            Some(tasks) => tasks.input_op("stdin.read_line", |inputs| {
                let n = inputs
                    .stdin
                    .iter()
                    .position(|b| *b == b'\n')
                    .map_or(inputs.stdin.len(), |n| n + 1);
                Ok(inputs.stdin.drain(..n).collect())
            }),
            None => {
                let mut bytes = Vec::new();
                std::io::BufRead::read_until(&mut std::io::stdin().lock(), b'\n', &mut bytes)
                    .map_err(from_std)?;
                Ok(bytes)
            }
        }
    }
    /// Appends one line, including its newline, or returns zero at EOF.
    #[track_caller]
    pub fn read_line(&self, buf: &mut String) -> Result<usize> {
        append_string(buf, self.line_bytes()?)
    }
    #[track_caller]
    pub fn lines(&self) -> Lines<StdinReader> {
        StdinReader {
            stdin: self.clone(),
            bytes: Vec::new(),
            pos: 0,
        }
        .lines()
    }
    #[track_caller]
    pub fn read_to_end(&self, buf: &mut Vec<u8>) -> Result<usize> {
        self.check()?;
        match &self.0 {
            Some(tasks) => tasks.input_op("stdin.read_to_end", |inputs| {
                let n = inputs.stdin.len();
                buf.extend(inputs.stdin.drain(..));
                Ok(n)
            }),
            None => std::io::Read::read_to_end(&mut std::io::stdin().lock(), buf).map_err(from_std),
        }
    }
    #[track_caller]
    pub fn read_to_string(&self, buf: &mut String) -> Result<usize> {
        append_to_string(buf, |bytes| self.read_to_end(bytes))
    }
    #[track_caller]
    pub fn is_terminal(&self) -> bool {
        match &self.0 {
            Some(tasks) => tasks.input_op("stdin.is_terminal", |_| false),
            None => {
                crate::task::outside_simulation();
                std::io::IsTerminal::is_terminal(&std::io::stdin())
            }
        }
    }
}

impl Read for &Stdin {
    #[track_caller]
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.check()?;
        if buf.is_empty() {
            return Ok(0);
        }
        match &self.0 {
            Some(tasks) => tasks.input_op("stdin.read", |inputs| {
                let n = buf.len().min(inputs.stdin.len());
                for (out, byte) in buf[..n].iter_mut().zip(inputs.stdin.drain(..n)) {
                    *out = byte;
                }
                Ok(n)
            }),
            None => std::io::Read::read(&mut std::io::stdin().lock(), buf).map_err(from_std),
        }
    }
}

impl Read for Stdin {
    #[track_caller]
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        (&*self).read(buf)
    }
}

fn append_string(buf: &mut String, bytes: Vec<u8>) -> Result<usize> {
    let text = std::str::from_utf8(&bytes).map_err(|_| Error::from(ErrorKind::InvalidData))?;
    buf.push_str(text);
    Ok(bytes.len())
}

/// The authority-carrying reader used by `Stdin::lines`. Each fill takes exactly one line
/// under the global lock, so dropping `Lines` never discards a later line.
pub struct StdinReader {
    stdin: Stdin,
    bytes: Vec<u8>,
    pos: usize,
}
impl Read for StdinReader {
    #[track_caller]
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        let bytes = self.fill_buf()?;
        let n = bytes.len().min(buf.len());
        buf[..n].copy_from_slice(&bytes[..n]);
        self.consume(n);
        Ok(n)
    }
}
impl BufRead for StdinReader {
    #[track_caller]
    fn read_line(&mut self, buf: &mut String) -> Result<usize> {
        let bytes = self.fill_buf()?.to_vec();
        self.consume(bytes.len());
        // Preserve BufRead's UTF-8 error while avoiding its untracked I/O closure.
        append_string(buf, bytes).map_err(|_| Error::INVALID_UTF8)
    }

    #[track_caller]
    fn fill_buf(&mut self) -> Result<&[u8]> {
        if self.pos == self.bytes.len() {
            self.bytes = self.stdin.line_bytes()?;
            self.pos = 0;
        }
        Ok(&self.bytes[self.pos..])
    }
    fn consume(&mut self, n: usize) {
        self.pos += n.min(self.bytes.len() - self.pos);
    }
}

/// Write-only output sink. Each write emits all its bytes; `flush` is a no-op.
#[derive(Clone, Copy, Debug)]
pub struct Stdout(());
#[derive(Clone, Copy, Debug)]
pub struct Stderr(());
pub fn stdout() -> Stdout {
    Stdout(())
}
pub fn stderr() -> Stderr {
    Stderr(())
}
macro_rules! output {
    ($ty:ident, $stream:literal, $sink:ident) => {
        impl Write for &$ty {
            #[track_caller]
            fn write(&mut self, bytes: &[u8]) -> Result<usize> {
                crate::__sink::bytes($stream, bytes);
                Ok(bytes.len())
            }
            #[track_caller]
            fn write_all(&mut self, bytes: &[u8]) -> Result<()> {
                crate::__sink::bytes($stream, bytes);
                Ok(())
            }
            #[track_caller]
            fn write_fmt(&mut self, args: std::fmt::Arguments<'_>) -> Result<()> {
                crate::__sink::$sink(args, false);
                Ok(())
            }
            fn flush(&mut self) -> Result<()> {
                Ok(())
            }
        }
        impl Write for $ty {
            #[track_caller]
            fn write(&mut self, bytes: &[u8]) -> Result<usize> {
                (&*self).write(bytes)
            }
            #[track_caller]
            fn write_all(&mut self, bytes: &[u8]) -> Result<()> {
                (&*self).write_all(bytes)
            }
            #[track_caller]
            fn write_fmt(&mut self, args: std::fmt::Arguments<'_>) -> Result<()> {
                (&*self).write_fmt(args)
            }
            fn flush(&mut self) -> Result<()> {
                Ok(())
            }
        }
    };
}
output!(Stdout, "out", out);
output!(Stderr, "err", err);

#[cfg(test)]
mod stdio_tests {
    use super::*;
    #[test]
    fn stdin_lines_keep_the_default_bufread_utf8_error() {
        let sim = crate::Root::sim(7);
        sim.world.stdin(b"\xff\n");
        let actual = sim.stdin.lines().next().unwrap().unwrap_err();
        let expected = (&b"\xff\n"[..]).lines().next().unwrap().unwrap_err();
        assert_eq!(format!("{actual:?}"), format!("{expected:?}"));
    }

    #[test]
    fn stdio_trace_spans_point_to_callers() {
        let sim = crate::Root::sim(7);
        sim.world.stdin("one\ntwo\n");
        let out_line = line!() + 1;
        stdout().write_all(b"out\n").unwrap();
        let err_line = line!() + 1;
        (&stderr()).write_all(b"err\n").unwrap();
        let mut lines = sim.stdin.lines();
        let lines_line = line!() + 1;
        assert_eq!(lines.next().unwrap().unwrap(), "one");
        let mut reader = &sim.stdin;
        let read_line = line!() + 1;
        assert_eq!(reader.read(&mut [0; 1]).unwrap(), 1);
        let mut line_reader = StdinReader {
            stdin: sim.stdin.clone(),
            bytes: Vec::new(),
            pos: 0,
        };
        let reader_line = line!() + 1;
        assert_eq!(line_reader.read(&mut [0; 1]).unwrap(), 1);
        let trace = sim.trace();
        let events: Vec<_> = trace
            .lines()
            .filter(|event| {
                event.contains("\"kind\":\"sink\"") || event.contains("\"kind\":\"cap\"")
            })
            .collect();
        for (event, line) in
            events
                .iter()
                .zip([out_line, err_line, lines_line, read_line, reader_line])
        {
            assert!(
                event.contains(&format!("\"file\":{:?},\"line\":{line},", file!())),
                "{event}"
            );
        }
        assert_eq!(events.len(), 5);
    }

    #[test]
    fn handle_format_failure_emits_no_partial_record() {
        struct Bad;
        impl std::fmt::Display for Bad {
            fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                Err(std::fmt::Error)
            }
        }
        let (result, records) = crate::__sink::capture(|| {
            write!(stdout(), "prefix {Bad}").unwrap();
        });
        assert!(result.is_err());
        assert!(records.is_empty());
    }

    #[test]
    fn records_assemble_bytes_per_stream_without_losing_utf8_or_tails() {
        fn run() -> (Vec<crate::sim::Record>, String) {
            let sim = crate::Root::sim(7);
            stdout().write_all(b"a\xc3").unwrap();
            stderr().write_all(b"err\n").unwrap();
            stdout().write_all(b"\xa9\n\xfftail").unwrap();
            crate::__sink::log(
                crate::log::Level::Trace,
                format_args!("disabled but captured"),
            );
            let records = sim.records();
            assert_eq!(
                records
                    .iter()
                    .map(|r| r.message.as_str())
                    .collect::<Vec<_>>(),
                ["err", "aé", "disabled but captured", "�tail"]
            );
            assert_eq!(records[0].sink, crate::sim::Sink::Err);
            assert_eq!(records[3].sink, crate::sim::Sink::Out);
            let trace = sim.trace();
            assert!(trace.contains("195"));
            assert!(trace.contains("255"));
            (records, trace)
        }
        assert_eq!(run(), run());
    }
}
