//! `std::io` (reference 1, 6.1): the `Stdin` capability, and the core that Tide's std owns
//! (bead `fx-ecb`): the `Read`, `Write`, `Seek`, and `BufRead` traits with Rust's provided
//! methods and adapters (`Bytes`, `Chain`, `Take`, `Split`, `Lines`), `BufReader`, `BufWriter`,
//! `IntoInnerError`, `Error`, `ErrorKind`, `Result`, and Rust's `Cursor` and `SeekFrom`, with
//! the traits implemented for `&[u8]`, `Vec<u8>`, `VecDeque<u8>`, `&mut [u8]`, and `Cursor`.
//!
//! The supported items follow Rust's API with `BufWriter<W: Sized>`. Per TASK-6, its Drop
//! discards pending bytes and logs a warning with their count; call `flush` or `into_inner` to write them. `ErrorKind`
//! adds `Cancelled`, which `Error::is_cancelled()` tests (reference 7.2; 03 §1.1), and `Lines`,
//! `Split`, and `Bytes` end after yielding a cancellation error. Rust's `ErrorKind` cannot
//! gain one, so `Error` and the traits that return it are Tide's too.
//! `io::Error::new` and `other` keep Rust's signatures; the admission driver checks that their
//! payload is `Pure` (02 §9, `TD0517`), and rejects `Error::last_os_error` (`TD0520`).
//!
//! `shared.rs` holds these items; the conformance corpus's fixture std includes it too, so they
//! have one definition. `Stdin` reads real or simulated input, checking cancellation at entry;
//! a real read already blocked in the kernel finishes on input/EOF.
//! Owned and borrowed `Stdin` implement `Read`. `lines()` pulls one line
//! per fill under the global input lock. `stdout()`/`stderr()` share the print sinks, always
//! return `Ok`, and provide no `lock()`; use `BufWriter::new(stdout())` for batching.
//! `net::TcpStream` implements these `Read`/`Write` traits, owned and borrowed.
//! Not provided here: vectored I/O, `LineWriter`, `copy`, `empty`,
//! `sink`, `repeat`, `io::read_to_string`, `BufWriter::into_parts`, `BufError`, and `BufRead`
//! for `VecDeque<u8>` (it implements `Read` and `Write` only).

include!("shared.rs");

include!("stdio.rs");

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error as _;

    #[derive(Debug, PartialEq)]
    struct Corrupt(u64);
    impl fmt::Display for Corrupt {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "corrupt at {}", self.0)
        }
    }
    impl error::Error for Corrupt {}

    #[test]
    fn errors_keep_their_kind_payload_and_rust_formatting() {
        let e: Error = ErrorKind::NotFound.into();
        assert_eq!(e.kind(), ErrorKind::NotFound);
        assert_eq!(e.to_string(), "entity not found");
        assert_eq!(format!("{e:?}"), "Kind(NotFound)");
        assert!(e.get_ref().is_none() && e.raw_os_error().is_none());

        let e = Error::new(ErrorKind::InvalidData, "bad header");
        assert_eq!(e.kind(), ErrorKind::InvalidData);
        assert_eq!(e.to_string(), "bad header");
        assert_eq!(
            format!("{e:?}"),
            r#"Custom { kind: InvalidData, error: "bad header" }"#
        );

        let mut e = Error::other(Corrupt(7));
        assert_eq!(e.kind(), ErrorKind::Other);
        assert_eq!(e.to_string(), "corrupt at 7");
        assert!(e.source().is_none());
        assert!(e.get_mut().unwrap().downcast_mut::<Corrupt>().is_some());
        let e = e.downcast::<std::fmt::Error>().unwrap_err();
        assert_eq!(e.downcast::<Corrupt>().unwrap(), Corrupt(7));
        assert_eq!(Error::other("x").into_inner().unwrap().to_string(), "x");

        let e = Error::from_raw_os_error(2);
        assert_eq!((e.raw_os_error(), e.kind()), (Some(2), ErrorKind::NotFound));
        assert!(e.to_string().ends_with("(os error 2)"), "{e}");
        assert_eq!(ErrorKind::BrokenPipe.to_string(), "broken pipe");
        fn send_sync<T: Send + Sync + error::Error>() {}
        send_sync::<Error>();
    }

    #[test]
    fn only_cancelled_errors_are_cancelled() {
        let cancelled = Error::from(ErrorKind::Cancelled);
        assert!(cancelled.is_cancelled());
        assert_eq!(cancelled.to_string(), "operation cancelled");
        assert!(Error::new(ErrorKind::Cancelled, "scope cancelled").is_cancelled());
        assert!(!Error::from(ErrorKind::Interrupted).is_cancelled());
        assert!(!Error::other("cancelled").is_cancelled());
        assert!(!Error::from_raw_os_error(4).is_cancelled());
    }

    /// Fails each read with `kind` until `fails` reaches 0, then reads from `data`.
    struct Flaky<'a> {
        data: &'a [u8],
        fails: usize,
        kind: ErrorKind,
    }
    impl Read for Flaky<'_> {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
            if self.fails > 0 {
                self.fails -= 1;
                return Err(self.kind.into());
            }
            let n = buf.len().min(self.data.len()).min(3);
            buf[..n].copy_from_slice(&self.data[..n]);
            self.data = &self.data[n..];
            Ok(n)
        }
    }

    #[test]
    fn read_retries_interrupted_but_not_cancelled() {
        let mut r = Flaky {
            data: b"hello",
            fails: 2,
            kind: ErrorKind::Interrupted,
        };
        let mut s = String::new();
        assert_eq!(r.read_to_string(&mut s).unwrap(), 5);
        assert_eq!(s, "hello");

        let mut r = Flaky {
            data: b"hello",
            fails: 1,
            kind: ErrorKind::Cancelled,
        };
        let mut buf = [0; 5];
        assert!(r.read_exact(&mut buf).unwrap_err().is_cancelled());
        let mut r = BufReader::new(Flaky {
            data: b"a\n",
            fails: 1,
            kind: ErrorKind::Cancelled,
        });
        let mut lines = r.by_ref().lines();
        assert!(lines.next().unwrap().unwrap_err().is_cancelled());
        assert!(lines.next().is_none());
        assert!(lines.next().is_none());
        let mut line = String::new();
        r.read_line(&mut line).unwrap();
        assert_eq!(line, "a\n");
    }

    #[test]
    fn iterators_end_only_after_cancelled_errors() {
        for kind in [ErrorKind::Cancelled, ErrorKind::Other] {
            let reader = || Flaky {
                data: b"a\n",
                fails: 1,
                kind,
            };
            let mut lines = BufReader::new(reader()).lines();
            let mut split = BufReader::new(reader()).split(b'\n');
            let mut bytes = reader().bytes();
            assert_eq!(lines.next().unwrap().unwrap_err().kind(), kind);
            assert_eq!(split.next().unwrap().unwrap_err().kind(), kind);
            assert_eq!(bytes.next().unwrap().unwrap_err().kind(), kind);
            if kind == ErrorKind::Cancelled {
                for _ in 0..2 {
                    assert!(lines.next().is_none());
                    assert!(split.next().is_none());
                    assert!(bytes.next().is_none());
                }
            } else {
                assert_eq!(lines.next().unwrap().unwrap(), "a");
                assert_eq!(split.next().unwrap().unwrap(), b"a");
                assert_eq!(bytes.next().unwrap().unwrap(), b'a');
            }
        }
    }

    #[test]
    fn slices_vectors_and_cursors_read_and_write() {
        let mut r: &[u8] = b"abcdef";
        let mut buf = [0; 2];
        assert_eq!(r.read(&mut buf).unwrap(), 2);
        assert_eq!(&buf, b"ab");
        let mut rest = Vec::new();
        assert_eq!(r.read_to_end(&mut rest).unwrap(), 4);
        assert_eq!(rest, b"cdef");
        let mut short: &[u8] = b"ab";
        let e = short.read_exact(&mut [0; 3]).unwrap_err();
        assert_eq!(
            (e.kind(), e.to_string().as_str()),
            (ErrorKind::UnexpectedEof, "failed to fill whole buffer")
        );
        let mut invalid: &[u8] = b"\xff";
        let mut s = String::from("kept");
        assert_eq!(
            invalid.read_to_string(&mut s).unwrap_err().kind(),
            ErrorKind::InvalidData
        );
        assert_eq!(s, "kept");

        let mut v = Vec::new();
        v.write_all(b"one").unwrap();
        write!(v, " {}", 2).unwrap();
        writeln!(&mut v, "!").unwrap();
        v.flush().unwrap();
        assert_eq!(v, b"one 2!\n");

        let mut arr = [0u8; 3];
        let mut out: &mut [u8] = &mut arr;
        assert_eq!(out.write(b"ab").unwrap(), 2);
        assert_eq!(
            out.write_all(b"cd").unwrap_err().kind(),
            ErrorKind::WriteZero
        );
        assert_eq!(&arr, b"abc");

        let mut c = Cursor::new(Vec::new());
        c.write_all(b"hello world").unwrap();
        c.seek(SeekFrom::Start(6)).unwrap();
        c.write_all(b"tide!").unwrap();
        c.seek(SeekFrom::End(2)).unwrap();
        c.write_all(b"x").unwrap();
        assert_eq!(c.get_ref(), b"hello tide!\0\0x");
        c.rewind().unwrap();
        let mut word = String::new();
        c.read_line(&mut word).unwrap();
        assert_eq!(word, "hello tide!\0\0x");
        assert_eq!(c.stream_position().unwrap(), 14);
        c.seek_relative(-3).unwrap();
        let mut tail = [0; 3];
        c.read_exact(&mut tail).unwrap();
        assert_eq!(&tail, b"\0\0x");
        assert_eq!(
            c.seek(SeekFrom::Current(-20)).unwrap_err().kind(),
            ErrorKind::InvalidInput
        );

        let mut fixed = Cursor::new([0u8; 4]);
        assert_eq!(fixed.write(b"abcdef").unwrap(), 4);
        assert_eq!(fixed.write(b"g").unwrap(), 0);
        let mut boxed = Cursor::new(vec![0u8; 2].into_boxed_slice());
        boxed.write_all(b"ab").unwrap();
        let mut backing = vec![1, 2];
        Cursor::new(&mut backing).write_all(b"z").unwrap();
        assert_eq!(backing, b"z\x02");
        let mut slice = [0u8; 2];
        Cursor::new(&mut slice[..]).write_all(b"q").unwrap();
        assert_eq!(slice, *b"q\0");
    }

    #[test]
    fn cursor_read_exact_failure_moves_to_end() {
        for position in [1, 10] {
            let mut cursor = Cursor::new(b"abc");
            cursor.set_position(position);
            assert_eq!(
                cursor.read_exact(&mut [0; 4]).unwrap_err().kind(),
                ErrorKind::UnexpectedEof
            );
            assert_eq!(cursor.position(), 3);
        }
    }

    #[test]
    fn buf_read_methods_and_adapters() {
        let mut r = BufReader::with_capacity(4, &b"one\ntwo\r\nthree"[..]);
        let mut line = String::new();
        assert_eq!(r.read_line(&mut line).unwrap(), 4);
        assert_eq!(line, "one\n");
        assert_eq!(r.buffer(), b"");
        assert_eq!(r.capacity(), 4);
        let lines: Vec<String> = r.lines().map(|l| l.unwrap()).collect();
        assert_eq!(lines, ["two", "three"]);

        let parts: Vec<Vec<u8>> = BufRead::split(&b"a,b,,c"[..], b',')
            .map(|p| p.unwrap())
            .collect();
        assert_eq!(parts, [&b"a"[..], b"b", b"", b"c"]);
        let mut r: &[u8] = b"skip;keep";
        assert_eq!(r.skip_until(b';').unwrap(), 5);
        let mut until = Vec::new();
        assert_eq!(r.read_until(b'!', &mut until).unwrap(), 4);
        assert_eq!(until, b"keep");

        let bytes: Vec<u8> = (&b"xy"[..]).bytes().map(|b| b.unwrap()).collect();
        assert_eq!(bytes, b"xy");
        let mut chained = (&b"ab"[..]).chain(&b"\ncd"[..]);
        let mut s = String::new();
        chained.read_line(&mut s).unwrap();
        chained.read_line(&mut s).unwrap();
        assert_eq!(s, "ab\ncd");
        let (first, second) = chained.into_inner();
        assert!(first.is_empty() && second.is_empty());

        let mut taken = (&b"abcdef"[..]).take(4);
        let mut s = String::new();
        taken.read_to_string(&mut s).unwrap();
        assert_eq!((s.as_str(), taken.limit()), ("abcd", 0));
        taken.set_limit(1);
        assert_eq!(taken.fill_buf().unwrap(), b"e");
        assert_eq!(*taken.get_ref(), b"ef");
        assert_eq!(taken.into_inner(), b"ef");

        let mut invalid = BufReader::new(&b"ok\n\xff\n"[..]);
        let mut s = String::new();
        invalid.read_line(&mut s).unwrap();
        assert_eq!(
            invalid.read_line(&mut s).unwrap_err().kind(),
            ErrorKind::InvalidData
        );
        assert_eq!(s, "ok\n");
    }

    #[test]
    fn buf_reader_reads_large_buffers_directly_and_seeks_from_the_caller() {
        let mut r = BufReader::with_capacity(2, Cursor::new(b"abcdefgh".to_vec()));
        let mut one = [0; 1];
        r.read_exact(&mut one).unwrap();
        assert_eq!(r.buffer(), b"b");
        assert_eq!(r.seek(SeekFrom::Current(1)).unwrap(), 2);
        assert_eq!(r.buffer(), b"");
        let mut big = [0; 4];
        assert_eq!(r.read(&mut big).unwrap(), 4);
        assert_eq!(&big, b"cdef");
        assert_eq!(r.get_ref().position(), 6);
        assert_eq!(r.stream_position().unwrap(), 6);
        r.get_mut().set_position(7);
        assert_eq!(r.into_inner().position(), 7);
        assert_eq!(
            format!("{:?}", BufReader::new(&b""[..])),
            "BufReader { reader: [], buffer: 0/8192 }"
        );
    }

    #[test]
    fn buf_reader_position_and_relative_seeks_keep_buffer() {
        let mut r = BufReader::with_capacity(4, Cursor::new(b"abcdefgh"));
        r.read_exact(&mut [0; 1]).unwrap();
        assert_eq!(r.buffer(), b"bcd");
        assert_eq!(r.stream_position().unwrap(), 1);
        assert_eq!(r.buffer(), b"bcd");
        assert_eq!(r.get_ref().position(), 4);
        Seek::seek_relative(&mut r, -1).unwrap();
        assert_eq!(r.buffer(), b"abcd");
        r.seek_relative(3).unwrap();
        assert_eq!(r.buffer(), b"d");
        assert_eq!(r.get_ref().position(), 4);
        r.seek_relative(3).unwrap();
        assert!(r.buffer().is_empty());
        assert_eq!(r.stream_position().unwrap(), 6);
        assert_eq!(
            r.seek_relative(i64::MIN).unwrap_err().kind(),
            ErrorKind::InvalidInput
        );
    }

    #[test]
    #[should_panic(expected = "overflow when subtracting remaining buffer size")]
    fn buf_reader_position_checks_inconsistent_inner_position() {
        let mut r = BufReader::with_capacity(4, Cursor::new(b"abcd"));
        r.fill_buf().unwrap();
        r.get_mut().set_position(0);
        let _ = r.stream_position();
    }

    #[test]
    fn borrowed_and_boxed_io_forward_overrides() {
        struct Overrides;
        impl Read for Overrides {
            fn read(&mut self, _: &mut [u8]) -> Result<usize> {
                panic!("read must not be called")
            }
        }
        impl BufRead for Overrides {
            fn fill_buf(&mut self) -> Result<&[u8]> {
                panic!("fill_buf must not be called")
            }
            fn consume(&mut self, _: usize) {
                panic!("consume must not be called")
            }
            fn skip_until(&mut self, byte: u8) -> Result<usize> {
                assert_eq!(byte, b';');
                Ok(7)
            }
        }
        impl Seek for Overrides {
            fn seek(&mut self, _: SeekFrom) -> Result<u64> {
                panic!("seek must not be called")
            }
            fn rewind(&mut self) -> Result<()> {
                Ok(())
            }
            fn stream_position(&mut self) -> Result<u64> {
                Ok(7)
            }
            fn seek_relative(&mut self, offset: i64) -> Result<()> {
                assert_eq!(offset, -1);
                Ok(())
            }
        }
        fn check(mut io: impl BufRead + Seek) {
            io.rewind().unwrap();
            assert_eq!(io.stream_position().unwrap(), 7);
            io.seek_relative(-1).unwrap();
            assert_eq!(io.skip_until(b';').unwrap(), 7);
        }
        check(&mut Overrides);
        check(Box::new(Overrides));
    }

    /// Accepts at most `room` bytes in total, then writes nothing.
    #[derive(Debug, Default)]
    struct Limited {
        out: Vec<u8>,
        room: usize,
        flushed: bool,
    }
    impl Write for Limited {
        fn write(&mut self, buf: &[u8]) -> Result<usize> {
            let n = buf.len().min(self.room);
            self.room -= n;
            self.out.extend_from_slice(&buf[..n]);
            Ok(n)
        }
        fn flush(&mut self) -> Result<()> {
            self.flushed = true;
            Ok(())
        }
    }

    #[test]
    fn buf_writer_buffers_flushes_and_reports_what_it_could_not_write() {
        let mut w = BufWriter::with_capacity(
            4,
            Limited {
                room: 100,
                ..Default::default()
            },
        );
        w.write_all(b"ab").unwrap();
        assert_eq!(
            (w.buffer(), w.get_ref().out.as_slice()),
            (&b"ab"[..], &b""[..])
        );
        write!(w, "{}", "cd").unwrap();
        assert_eq!(
            (w.buffer(), w.get_ref().out.as_slice()),
            (&b"abcd"[..], &b""[..])
        );
        w.write_all(b"large write").unwrap();
        assert_eq!(w.get_ref().out, b"abcdlarge write");
        w.write_all(b"z").unwrap();
        w.flush().unwrap();
        assert!(w.get_ref().flushed);
        assert_eq!(w.capacity(), 4);
        assert_eq!(w.into_inner().unwrap().out, b"abcdlarge writez");

        let mut w = BufWriter::with_capacity(
            8,
            Limited {
                room: 2,
                ..Default::default()
            },
        );
        w.write_all(b"abcd").unwrap();
        let e = w.into_inner().unwrap_err();
        assert_eq!(e.error().kind(), ErrorKind::WriteZero);
        assert_eq!(e.to_string(), "failed to write the buffered data");
        let (error, w) = e.into_parts();
        assert_eq!(error.kind(), ErrorKind::WriteZero);
        assert_eq!(
            (w.buffer(), w.get_ref().out.as_slice()),
            (&b"cd"[..], &b"ab"[..])
        );
        let e: Error = w.into_inner().unwrap_err().into();
        assert_eq!(e.kind(), ErrorKind::WriteZero);

        let mut c = BufWriter::new(Cursor::new(Vec::new()));
        c.write_all(b"abc").unwrap();
        assert_eq!(c.seek(SeekFrom::Start(1)).unwrap(), 1);
        c.write_all(b"Z").unwrap();
        assert_eq!(c.into_inner().unwrap().into_inner(), b"aZc");
    }

    #[test]
    fn buf_writer_drop_does_not_write_or_flush() {
        let mut backing = Limited {
            room: 100,
            ..Default::default()
        };
        {
            let mut w = BufWriter::new(&mut backing);
            w.write_all(b"on drop").unwrap();
            assert_eq!(w.buffer(), b"on drop");
        }
        assert!(backing.out.is_empty());
        assert!(!backing.flushed);
    }

    #[test]
    fn buf_writer_drains_acknowledged_prefix_on_unwind() {
        struct PanicsAfterPartialWrite {
            out: Vec<u8>,
            calls: usize,
        }
        impl Write for PanicsAfterPartialWrite {
            fn write(&mut self, buf: &[u8]) -> Result<usize> {
                self.calls += 1;
                assert_ne!(self.calls, 2, "writer panicked after partial write");
                let n = buf.len().min(2);
                self.out.extend_from_slice(&buf[..n]);
                Ok(n)
            }
            fn flush(&mut self) -> Result<()> {
                Ok(())
            }
        }
        let mut w = BufWriter::with_capacity(
            8,
            PanicsAfterPartialWrite {
                out: Vec::new(),
                calls: 0,
            },
        );
        w.write_all(b"abcd").unwrap();
        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| w.flush()));
        assert!(panic.is_err());
        assert!(w.panicked);
        assert_eq!(w.get_ref().out, b"ab");
        assert_eq!(w.buffer(), b"cd");
        w.flush().unwrap();
        assert!(!w.panicked);
        assert!(w.buffer().is_empty());
        assert_eq!(w.get_ref().out, b"abcd");
    }
}
