use tide_std::io;

#[test]
fn buf_reader_seek_relative_needs_no_seek_import() {
    use io::{BufReader, Cursor, Read};

    let mut reader = BufReader::with_capacity(4, Cursor::new(b"abcdef"));
    reader.read_exact(&mut [0; 1]).unwrap();
    reader.seek_relative(-1).unwrap();
    assert_eq!(reader.buffer(), b"abcd");
    assert_eq!(reader.get_ref().position(), 4);
}

#[test]
fn io_prelude_and_reservation_error_conversion() {
    use io::prelude::*;

    fn reserve() -> io::Result<()> {
        Vec::<u8>::new().try_reserve(usize::MAX)?;
        Ok(())
    }
    let error = reserve().unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::OutOfMemory);
    assert_eq!(error.to_string(), "out of memory");
    assert!(std::error::Error::source(&error).is_none());
    assert!(error.get_ref().is_none());

    let mut cursor = io::Cursor::new(Vec::new());
    cursor.write_all(b"hello\n").unwrap();
    cursor.rewind().unwrap();
    let mut reader = io::BufReader::new(cursor);
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    assert_eq!(line, "hello\n");
    assert_eq!(reader.bytes().count(), 0);
}
