// std::io's core (bead fx-ecb), which Tide's std owns: `Read`, `Write`, `Seek`, and `BufRead`
// with Rust's provided methods, their adapters, `BufReader`, `BufWriter`, `IntoInnerError`,
// `Error`, `ErrorKind` with `Cancelled`, and `Result`, implemented over in-memory readers and
// writers. Rust's `Cursor` and `SeekFrom` are re-exported: they hold no error, and the traits
// here are implemented for `Cursor`. The conformance corpus's fixture std includes this file
// in its `io` module too, so both crates define these items once and key them alike
// (`tide_std::io::Error`). An included file cannot carry inner attributes or `//!` docs; the
// module's docs are in `mod.rs`.
//
// The supported items follow Rust's API with `BufWriter<W: Sized>`. `ErrorKind` adds
// `Cancelled`, which `is_cancelled()` tests; `Lines`, `Split`, and `Bytes` end after yielding
// it. Per TASK-6, `BufWriter` Drop discards pending bytes and warns with their count. Rust's items cannot
// carry cancellation: its `ErrorKind` cannot gain a variant, and the traits return its error.

pub use std::io::{Cursor, SeekFrom};

/// Common I/O traits, as in Rust.
pub mod prelude {
    pub use super::{BufRead, Read, Seek, Write};
}

use std::error;
use std::fmt;

/// Rust's default buffer size for `BufReader` and `BufWriter`, and the chunk `read_to_end`
/// reads in.
const DEFAULT_BUF_SIZE: usize = 8 * 1024;

/// A specialized `Result` for I/O operations, as Rust's.
pub type Result<T> = std::result::Result<T, Error>;

/// The general category of an I/O error: Rust's stable kinds, plus `Cancelled`.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ErrorKind {
    NotFound,
    PermissionDenied,
    ConnectionRefused,
    ConnectionReset,
    HostUnreachable,
    NetworkUnreachable,
    ConnectionAborted,
    NotConnected,
    AddrInUse,
    AddrNotAvailable,
    NetworkDown,
    BrokenPipe,
    AlreadyExists,
    WouldBlock,
    NotADirectory,
    IsADirectory,
    DirectoryNotEmpty,
    ReadOnlyFilesystem,
    StaleNetworkFileHandle,
    InvalidInput,
    InvalidData,
    TimedOut,
    WriteZero,
    StorageFull,
    NotSeekable,
    QuotaExceeded,
    FileTooLarge,
    ResourceBusy,
    ExecutableFileBusy,
    Deadlock,
    CrossesDevices,
    TooManyLinks,
    InvalidFilename,
    ArgumentListTooLong,
    Interrupted,
    Unsupported,
    UnexpectedEof,
    OutOfMemory,
    /// The operation's scope was cancelled (reference 7.2). Cancellation is level-triggered,
    /// so unlike `Interrupted`, a retry fails the same way; no provided method retries it.
    Cancelled,
    Other,
    /// An OS error Rust has no stable kind for, as Rust's own hidden kind.
    /// Programs should not name this variant.
    #[doc(hidden)]
    Uncategorized,
}

impl ErrorKind {
    fn as_str(&self) -> &'static str {
        use ErrorKind::*;
        match *self {
            AddrInUse => "address in use",
            AddrNotAvailable => "address not available",
            AlreadyExists => "entity already exists",
            ArgumentListTooLong => "argument list too long",
            BrokenPipe => "broken pipe",
            Cancelled => "operation cancelled",
            ConnectionAborted => "connection aborted",
            ConnectionRefused => "connection refused",
            ConnectionReset => "connection reset",
            CrossesDevices => "cross-device link or rename",
            Deadlock => "deadlock",
            DirectoryNotEmpty => "directory not empty",
            ExecutableFileBusy => "executable file busy",
            FileTooLarge => "file too large",
            HostUnreachable => "host unreachable",
            Interrupted => "operation interrupted",
            InvalidData => "invalid data",
            InvalidFilename => "invalid filename",
            InvalidInput => "invalid input parameter",
            IsADirectory => "is a directory",
            NetworkDown => "network down",
            NetworkUnreachable => "network unreachable",
            NotADirectory => "not a directory",
            NotConnected => "not connected",
            NotFound => "entity not found",
            NotSeekable => "seek on unseekable file",
            Other => "other error",
            OutOfMemory => "out of memory",
            PermissionDenied => "permission denied",
            QuotaExceeded => "quota exceeded",
            ReadOnlyFilesystem => "read-only filesystem or storage medium",
            ResourceBusy => "resource busy",
            StaleNetworkFileHandle => "stale network file handle",
            StorageFull => "no storage space",
            TimedOut => "timed out",
            TooManyLinks => "too many links",
            Uncategorized => "uncategorized error",
            UnexpectedEof => "unexpected end of file",
            Unsupported => "unsupported",
            WouldBlock => "operation would block",
            WriteZero => "write zero",
        }
    }

    /// Rust's kind for an OS error; its unstable kinds are `Uncategorized`.
    pub(crate) fn from_rust(kind: std::io::ErrorKind) -> ErrorKind {
        use std::io::ErrorKind as R;
        match kind {
            R::NotFound => ErrorKind::NotFound,
            R::PermissionDenied => ErrorKind::PermissionDenied,
            R::ConnectionRefused => ErrorKind::ConnectionRefused,
            R::ConnectionReset => ErrorKind::ConnectionReset,
            R::HostUnreachable => ErrorKind::HostUnreachable,
            R::NetworkUnreachable => ErrorKind::NetworkUnreachable,
            R::ConnectionAborted => ErrorKind::ConnectionAborted,
            R::NotConnected => ErrorKind::NotConnected,
            R::AddrInUse => ErrorKind::AddrInUse,
            R::AddrNotAvailable => ErrorKind::AddrNotAvailable,
            R::NetworkDown => ErrorKind::NetworkDown,
            R::BrokenPipe => ErrorKind::BrokenPipe,
            R::AlreadyExists => ErrorKind::AlreadyExists,
            R::WouldBlock => ErrorKind::WouldBlock,
            R::NotADirectory => ErrorKind::NotADirectory,
            R::IsADirectory => ErrorKind::IsADirectory,
            R::DirectoryNotEmpty => ErrorKind::DirectoryNotEmpty,
            R::ReadOnlyFilesystem => ErrorKind::ReadOnlyFilesystem,
            R::StaleNetworkFileHandle => ErrorKind::StaleNetworkFileHandle,
            R::InvalidInput => ErrorKind::InvalidInput,
            R::InvalidData => ErrorKind::InvalidData,
            R::TimedOut => ErrorKind::TimedOut,
            R::WriteZero => ErrorKind::WriteZero,
            R::StorageFull => ErrorKind::StorageFull,
            R::NotSeekable => ErrorKind::NotSeekable,
            R::QuotaExceeded => ErrorKind::QuotaExceeded,
            R::FileTooLarge => ErrorKind::FileTooLarge,
            R::ResourceBusy => ErrorKind::ResourceBusy,
            R::ExecutableFileBusy => ErrorKind::ExecutableFileBusy,
            R::Deadlock => ErrorKind::Deadlock,
            R::CrossesDevices => ErrorKind::CrossesDevices,
            R::TooManyLinks => ErrorKind::TooManyLinks,
            R::InvalidFilename => ErrorKind::InvalidFilename,
            R::ArgumentListTooLong => ErrorKind::ArgumentListTooLong,
            R::Interrupted => ErrorKind::Interrupted,
            R::Unsupported => ErrorKind::Unsupported,
            R::UnexpectedEof => ErrorKind::UnexpectedEof,
            R::OutOfMemory => ErrorKind::OutOfMemory,
            R::Other => ErrorKind::Other,
            _ => ErrorKind::Uncategorized,
        }
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The error type of the I/O traits and of every capability operation that does I/O. It is
/// `Error + Send + Sync + Pure` (reference 2.2): `new` and `other` box only a `Pure` error or a
/// string, which the admission driver checks at each call (`FX0517`).
pub struct Error {
    repr: Repr,
}

enum Repr {
    Simple(ErrorKind),
    /// One of std's fixed messages, as Rust's `const_error!`.
    Message(ErrorKind, &'static str),
    Custom(ErrorKind, Box<dyn error::Error + Send + Sync>),
    Os(i32),
}

impl Error {
    const READ_EXACT_EOF: Error =
        Error::message(ErrorKind::UnexpectedEof, "failed to fill whole buffer");
    const WRITE_ALL_EOF: Error =
        Error::message(ErrorKind::WriteZero, "failed to write whole buffer");
    const INVALID_UTF8: Error =
        Error::message(ErrorKind::InvalidData, "stream did not contain valid UTF-8");

    const fn message(kind: ErrorKind, message: &'static str) -> Error {
        Error {
            repr: Repr::Message(kind, message),
        }
    }

    /// An error of `kind` carrying `error`: a `Pure` error, a `String`, a `&str`, or a
    /// `Cow<'_, str>` (02 §9).
    pub fn new<E>(kind: ErrorKind, error: E) -> Error
    where
        E: Into<Box<dyn error::Error + Send + Sync>>,
    {
        Error {
            repr: Repr::Custom(kind, error.into()),
        }
    }

    /// `Error::new(ErrorKind::Other, error)`.
    pub fn other<E>(error: E) -> Error
    where
        E: Into<Box<dyn error::Error + Send + Sync>>,
    {
        Error::new(ErrorKind::Other, error)
    }

    /// The calling thread's last OS error. Programs cannot call it (`FX0520`): an OS error
    /// code is read from a capability's result.
    pub fn last_os_error() -> Error {
        Error::from_raw_os_error(std::io::Error::last_os_error().raw_os_error().unwrap_or(0))
    }

    pub fn from_raw_os_error(code: i32) -> Error {
        Error {
            repr: Repr::Os(code),
        }
    }

    pub fn raw_os_error(&self) -> Option<i32> {
        match self.repr {
            Repr::Os(code) => Some(code),
            _ => None,
        }
    }

    pub fn get_ref(&self) -> Option<&(dyn error::Error + Send + Sync + 'static)> {
        match &self.repr {
            Repr::Custom(_, e) => Some(&**e),
            _ => None,
        }
    }

    pub fn get_mut(&mut self) -> Option<&mut (dyn error::Error + Send + Sync + 'static)> {
        match &mut self.repr {
            Repr::Custom(_, e) => Some(&mut **e),
            _ => None,
        }
    }

    pub fn into_inner(self) -> Option<Box<dyn error::Error + Send + Sync>> {
        match self.repr {
            Repr::Custom(_, e) => Some(e),
            _ => None,
        }
    }

    /// The payload as an `E`, if it is one; otherwise the error unchanged.
    pub fn downcast<E>(self) -> std::result::Result<E, Self>
    where
        E: error::Error + Send + Sync + 'static,
    {
        match self.repr {
            Repr::Custom(kind, e) => match e.downcast::<E>() {
                Ok(e) => Ok(*e),
                Err(e) => Err(Error {
                    repr: Repr::Custom(kind, e),
                }),
            },
            repr => Err(Error { repr }),
        }
    }

    pub fn kind(&self) -> ErrorKind {
        match self.repr {
            Repr::Simple(kind) | Repr::Message(kind, _) | Repr::Custom(kind, _) => kind,
            Repr::Os(code) => ErrorKind::from_rust(std::io::Error::from_raw_os_error(code).kind()),
        }
    }

    /// Whether this is a cancellation error: its kind is `Cancelled` (reference 7.2).
    pub fn is_cancelled(&self) -> bool {
        self.kind() == ErrorKind::Cancelled
    }

    fn is_interrupted(&self) -> bool {
        self.kind() == ErrorKind::Interrupted
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error {
            repr: Repr::Simple(kind),
        }
    }
}

impl From<std::collections::TryReserveError> for Error {
    fn from(_: std::collections::TryReserveError) -> Error {
        ErrorKind::OutOfMemory.into()
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.repr {
            Repr::Simple(kind) => f.debug_tuple("Kind").field(kind).finish(),
            Repr::Message(kind, message) => f
                .debug_struct("Error")
                .field("kind", kind)
                .field("message", message)
                .finish(),
            Repr::Custom(kind, error) => f
                .debug_struct("Custom")
                .field("kind", kind)
                .field("error", error)
                .finish(),
            Repr::Os(code) => fmt::Debug::fmt(&std::io::Error::from_raw_os_error(*code), f),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.repr {
            Repr::Simple(kind) => f.write_str(kind.as_str()),
            Repr::Message(_, message) => f.write_str(message),
            Repr::Custom(_, error) => fmt::Display::fmt(error, f),
            Repr::Os(code) => fmt::Display::fmt(&std::io::Error::from_raw_os_error(*code), f),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match &self.repr {
            Repr::Custom(_, error) => error.source(),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------- traits

/// A source of bytes.
pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize> {
        let start = buf.len();
        let mut chunk = [0; DEFAULT_BUF_SIZE];
        loop {
            match self.read(&mut chunk) {
                Ok(0) => return Ok(buf.len() - start),
                Ok(n) => buf.extend_from_slice(&chunk[..n]),
                Err(e) if e.is_interrupted() => {}
                Err(e) => return Err(e),
            }
        }
    }

    fn read_to_string(&mut self, buf: &mut String) -> Result<usize> {
        append_to_string(buf, |bytes| self.read_to_end(bytes))
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        read_exact(self, buf)
    }

    fn by_ref(&mut self) -> &mut Self
    where
        Self: Sized,
    {
        self
    }

    fn bytes(self) -> Bytes<Self>
    where
        Self: Sized,
    {
        Bytes {
            inner: self,
            done: false,
        }
    }

    fn chain<R: Read>(self, next: R) -> Chain<Self, R>
    where
        Self: Sized,
    {
        Chain {
            first: self,
            second: next,
            done_first: false,
        }
    }

    fn take(self, limit: u64) -> Take<Self>
    where
        Self: Sized,
    {
        Take { inner: self, limit }
    }
}

/// Runs `read`, which appends bytes, and appends them to `buf` if they are UTF-8. As Rust's,
/// bytes a failed read appended stay when they are valid; invalid bytes never reach `buf`.
fn append_to_string(
    buf: &mut String,
    read: impl FnOnce(&mut Vec<u8>) -> Result<usize>,
) -> Result<usize> {
    let mut bytes = Vec::new();
    let result = read(&mut bytes);
    match String::from_utf8(bytes) {
        Ok(s) => {
            buf.push_str(&s);
            result
        }
        Err(_) => result.and(Err(Error::INVALID_UTF8)),
    }
}

/// A sink of bytes.
pub trait Write {
    fn write(&mut self, buf: &[u8]) -> Result<usize>;

    fn flush(&mut self) -> Result<()>;

    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        write_all(self, buf)
    }

    /// What `write!` and `writeln!` call. As in Rust, a formatting impl that returns an error
    /// while the writer did not panics.
    fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> Result<()> {
        write_fmt(self, args)
    }

    fn by_ref(&mut self) -> &mut Self
    where
        Self: Sized,
    {
        self
    }
}

/// A cursor that can move within a stream of bytes.
pub trait Seek {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64>;

    fn rewind(&mut self) -> Result<()> {
        self.seek(SeekFrom::Start(0))?;
        Ok(())
    }

    fn stream_position(&mut self) -> Result<u64> {
        self.seek(SeekFrom::Current(0))
    }

    fn seek_relative(&mut self, offset: i64) -> Result<()> {
        self.seek(SeekFrom::Current(offset))?;
        Ok(())
    }
}

/// A reader with an internal buffer, which reads lines and up to delimiters.
pub trait BufRead: Read {
    fn fill_buf(&mut self) -> Result<&[u8]>;

    fn consume(&mut self, amount: usize);

    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> Result<usize> {
        let mut read = 0;
        loop {
            let (done, used) = {
                let available = match self.fill_buf() {
                    Ok(available) => available,
                    Err(e) if e.is_interrupted() => continue,
                    Err(e) => return Err(e),
                };
                match available.iter().position(|&b| b == byte) {
                    Some(i) => {
                        buf.extend_from_slice(&available[..=i]);
                        (true, i + 1)
                    }
                    None => {
                        buf.extend_from_slice(available);
                        (false, available.len())
                    }
                }
            };
            self.consume(used);
            read += used;
            if done || used == 0 {
                return Ok(read);
            }
        }
    }

    fn skip_until(&mut self, byte: u8) -> Result<usize> {
        let mut read = 0;
        loop {
            let (done, used) = {
                let available = match self.fill_buf() {
                    Ok(available) => available,
                    Err(e) if e.is_interrupted() => continue,
                    Err(e) => return Err(e),
                };
                match available.iter().position(|&b| b == byte) {
                    Some(i) => (true, i + 1),
                    None => (false, available.len()),
                }
            };
            self.consume(used);
            read += used;
            if done || used == 0 {
                return Ok(read);
            }
        }
    }

    fn read_line(&mut self, buf: &mut String) -> Result<usize> {
        append_to_string(buf, |bytes| self.read_until(b'\n', bytes))
    }

    fn split(self, byte: u8) -> Split<Self>
    where
        Self: Sized,
    {
        Split {
            inner: self,
            delim: byte,
            done: false,
        }
    }

    fn lines(self) -> Lines<Self>
    where
        Self: Sized,
    {
        Lines {
            inner: self,
            done: false,
        }
    }
}

// ---------------------------------------------------------------- forwarding impls

impl<R: Read + ?Sized> Read for &mut R {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        (**self).read(buf)
    }
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize> {
        (**self).read_to_end(buf)
    }
    fn read_to_string(&mut self, buf: &mut String) -> Result<usize> {
        (**self).read_to_string(buf)
    }
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        (**self).read_exact(buf)
    }
}

impl<R: Read + ?Sized> Read for Box<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        (**self).read(buf)
    }
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize> {
        (**self).read_to_end(buf)
    }
    fn read_to_string(&mut self, buf: &mut String) -> Result<usize> {
        (**self).read_to_string(buf)
    }
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        (**self).read_exact(buf)
    }
}

impl<W: Write + ?Sized> Write for &mut W {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        (**self).write(buf)
    }
    fn flush(&mut self) -> Result<()> {
        (**self).flush()
    }
    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        (**self).write_all(buf)
    }
    fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> Result<()> {
        (**self).write_fmt(args)
    }
}

impl<W: Write + ?Sized> Write for Box<W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        (**self).write(buf)
    }
    fn flush(&mut self) -> Result<()> {
        (**self).flush()
    }
    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        (**self).write_all(buf)
    }
    fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> Result<()> {
        (**self).write_fmt(args)
    }
}

impl<S: Seek + ?Sized> Seek for &mut S {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        (**self).seek(pos)
    }
    fn rewind(&mut self) -> Result<()> {
        (**self).rewind()
    }
    fn stream_position(&mut self) -> Result<u64> {
        (**self).stream_position()
    }
    fn seek_relative(&mut self, offset: i64) -> Result<()> {
        (**self).seek_relative(offset)
    }
}

impl<S: Seek + ?Sized> Seek for Box<S> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        (**self).seek(pos)
    }
    fn rewind(&mut self) -> Result<()> {
        (**self).rewind()
    }
    fn stream_position(&mut self) -> Result<u64> {
        (**self).stream_position()
    }
    fn seek_relative(&mut self, offset: i64) -> Result<()> {
        (**self).seek_relative(offset)
    }
}

impl<B: BufRead + ?Sized> BufRead for &mut B {
    fn fill_buf(&mut self) -> Result<&[u8]> {
        (**self).fill_buf()
    }
    fn consume(&mut self, amount: usize) {
        (**self).consume(amount)
    }
    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> Result<usize> {
        (**self).read_until(byte, buf)
    }
    fn skip_until(&mut self, byte: u8) -> Result<usize> {
        (**self).skip_until(byte)
    }
    fn read_line(&mut self, buf: &mut String) -> Result<usize> {
        (**self).read_line(buf)
    }
}

impl<B: BufRead + ?Sized> BufRead for Box<B> {
    fn fill_buf(&mut self) -> Result<&[u8]> {
        (**self).fill_buf()
    }
    fn consume(&mut self, amount: usize) {
        (**self).consume(amount)
    }
    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> Result<usize> {
        (**self).read_until(byte, buf)
    }
    fn skip_until(&mut self, byte: u8) -> Result<usize> {
        (**self).skip_until(byte)
    }
    fn read_line(&mut self, buf: &mut String) -> Result<usize> {
        (**self).read_line(buf)
    }
}

// ---------------------------------------------------------------- in-memory readers and writers

/// Reads from the front of the slice, which shrinks.
impl Read for &[u8] {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let n = buf.len().min(self.len());
        let (head, tail) = self.split_at(n);
        buf[..n].copy_from_slice(head);
        *self = tail;
        Ok(n)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize> {
        let n = self.len();
        buf.extend_from_slice(self);
        *self = &self[n..];
        Ok(n)
    }

    fn read_to_string(&mut self, buf: &mut String) -> Result<usize> {
        let s = std::str::from_utf8(self).map_err(|_| Error::INVALID_UTF8)?;
        let n = self.len();
        buf.push_str(s);
        *self = &self[n..];
        Ok(n)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        if buf.len() > self.len() {
            *self = &self[self.len()..];
            return Err(Error::READ_EXACT_EOF);
        }
        let (head, tail) = self.split_at(buf.len());
        buf.copy_from_slice(head);
        *self = tail;
        Ok(())
    }
}

impl BufRead for &[u8] {
    fn fill_buf(&mut self) -> Result<&[u8]> {
        Ok(*self)
    }

    fn consume(&mut self, amount: usize) {
        *self = &self[amount..];
    }
}

/// Appends to the vector.
impl Write for Vec<u8> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        self.extend_from_slice(buf);
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

/// Writes into the front of the slice, which shrinks; a write past its end is short.
impl Write for &mut [u8] {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let n = buf.len().min(self.len());
        let (head, tail) = std::mem::take(self).split_at_mut(n);
        head.copy_from_slice(&buf[..n]);
        *self = tail;
        Ok(n)
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        if self.write(buf)? == buf.len() {
            Ok(())
        } else {
            Err(Error::WRITE_ALL_EOF)
        }
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

/// The bytes of `cursor`'s data from its position on, empty past the end.
fn remaining<T: AsRef<[u8]>>(cursor: &Cursor<T>) -> &[u8] {
    let data = cursor.get_ref().as_ref();
    let start = cursor.position().min(data.len() as u64) as usize;
    &data[start..]
}

impl<T: AsRef<[u8]>> Read for Cursor<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let n = Read::read(&mut remaining(self), buf)?;
        self.set_position(self.position() + n as u64);
        Ok(n)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        let result = remaining(self).read_exact(buf);
        match result {
            Ok(()) => self.set_position(self.position() + buf.len() as u64),
            Err(_) => self.set_position(self.get_ref().as_ref().len() as u64),
        }
        result
    }
}

impl<T: AsRef<[u8]>> BufRead for Cursor<T> {
    fn fill_buf(&mut self) -> Result<&[u8]> {
        Ok(remaining(self))
    }

    fn consume(&mut self, amount: usize) {
        self.set_position(self.position() + amount as u64);
    }
}

impl<T: AsRef<[u8]>> Seek for Cursor<T> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        let (base, offset) = match pos {
            SeekFrom::Start(n) => {
                self.set_position(n);
                return Ok(n);
            }
            SeekFrom::End(n) => (self.get_ref().as_ref().len() as u64, n),
            SeekFrom::Current(n) => (self.position(), n),
        };
        match base.checked_add_signed(offset) {
            Some(n) => {
                self.set_position(n);
                Ok(n)
            }
            None => Err(Error::message(
                ErrorKind::InvalidInput,
                "invalid seek to a negative or overflowing position",
            )),
        }
    }
}

/// Writes `buf` into `slice` at `*pos`, as far as `slice` reaches.
fn slice_write(pos: &mut u64, slice: &mut [u8], buf: &[u8]) -> Result<usize> {
    let start = (*pos).min(slice.len() as u64) as usize;
    let n = (&mut slice[start..]).write(buf)?;
    *pos += n as u64;
    Ok(n)
}

/// Writes `buf` into `vec` at `*pos`, zero-filling any gap and growing the vector.
fn vec_write(pos: &mut u64, vec: &mut Vec<u8>, buf: &[u8]) -> Result<usize> {
    let start = usize::try_from(*pos).map_err(|_| {
        Error::message(
            ErrorKind::InvalidInput,
            "cursor position exceeds maximum possible vector length",
        )
    })?;
    if vec.len() < start {
        vec.resize(start, 0);
    }
    let overlap = (vec.len() - start).min(buf.len());
    vec[start..start + overlap].copy_from_slice(&buf[..overlap]);
    vec.extend_from_slice(&buf[overlap..]);
    *pos = (start + buf.len()) as u64;
    Ok(buf.len())
}

/// Implements `Write` for a cursor through `slice_write` or `vec_write`.
macro_rules! cursor_write {
    ($([$($gen:tt)*] $ty:ty => $via:ident;)*) => {$(
        impl<$($gen)*> Write for Cursor<$ty> {
            fn write(&mut self, buf: &[u8]) -> Result<usize> {
                let mut pos = self.position();
                let n = $via(&mut pos, self.get_mut(), buf)?;
                self.set_position(pos);
                Ok(n)
            }

            fn flush(&mut self) -> Result<()> {
                Ok(())
            }
        }
    )*};
}

cursor_write! {
    [] Vec<u8> => vec_write;
    [] &mut Vec<u8> => vec_write;
    [] &mut [u8] => slice_write;
    [] Box<[u8]> => slice_write;
    [const N: usize] [u8; N] => slice_write;
}

// ---------------------------------------------------------------- adapters

/// The bytes of a reader, from `Read::bytes`. Ends after yielding a cancellation error.
#[derive(Debug)]
pub struct Bytes<R> {
    inner: R,
    done: bool,
}

impl<R: Read> Iterator for Bytes<R> {
    type Item = Result<u8>;

    fn next(&mut self) -> Option<Result<u8>> {
        if self.done {
            return None;
        }
        let mut byte = 0;
        loop {
            return match self.inner.read(std::slice::from_mut(&mut byte)) {
                Ok(0) => None,
                Ok(_) => Some(Ok(byte)),
                Err(e) if e.is_interrupted() => continue,
                Err(e) => {
                    self.done = e.is_cancelled();
                    Some(Err(e))
                }
            };
        }
    }
}

/// Two readers one after the other, from `Read::chain`.
#[derive(Debug)]
pub struct Chain<T, U> {
    first: T,
    second: U,
    done_first: bool,
}

impl<T, U> Chain<T, U> {
    pub fn into_inner(self) -> (T, U) {
        (self.first, self.second)
    }

    pub fn get_ref(&self) -> (&T, &U) {
        (&self.first, &self.second)
    }

    pub fn get_mut(&mut self) -> (&mut T, &mut U) {
        (&mut self.first, &mut self.second)
    }
}

impl<T: Read, U: Read> Read for Chain<T, U> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if !self.done_first {
            match self.first.read(buf)? {
                0 if !buf.is_empty() => self.done_first = true,
                n => return Ok(n),
            }
        }
        self.second.read(buf)
    }
}

impl<T: BufRead, U: BufRead> BufRead for Chain<T, U> {
    fn fill_buf(&mut self) -> Result<&[u8]> {
        if !self.done_first {
            match self.first.fill_buf()? {
                buf if buf.is_empty() => self.done_first = true,
                buf => return Ok(buf),
            }
        }
        self.second.fill_buf()
    }

    fn consume(&mut self, amount: usize) {
        if !self.done_first {
            self.first.consume(amount)
        } else {
            self.second.consume(amount)
        }
    }
}

/// A reader limited to `limit` bytes, from `Read::take`.
#[derive(Debug)]
pub struct Take<T> {
    inner: T,
    limit: u64,
}

impl<T> Take<T> {
    pub fn limit(&self) -> u64 {
        self.limit
    }

    pub fn set_limit(&mut self, limit: u64) {
        self.limit = limit;
    }

    pub fn into_inner(self) -> T {
        self.inner
    }

    pub fn get_ref(&self) -> &T {
        &self.inner
    }

    pub fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

impl<T: Read> Read for Take<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if self.limit == 0 {
            return Ok(0);
        }
        let max = (buf.len() as u64).min(self.limit) as usize;
        let n = self.inner.read(&mut buf[..max])?;
        assert!(n as u64 <= self.limit, "number of read bytes exceeds limit");
        self.limit -= n as u64;
        Ok(n)
    }
}

impl<T: BufRead> BufRead for Take<T> {
    fn fill_buf(&mut self) -> Result<&[u8]> {
        if self.limit == 0 {
            return Ok(&[]);
        }
        let buf = self.inner.fill_buf()?;
        let n = (buf.len() as u64).min(self.limit) as usize;
        Ok(&buf[..n])
    }

    fn consume(&mut self, amount: usize) {
        let amount = (amount as u64).min(self.limit) as usize;
        self.limit -= amount as u64;
        self.inner.consume(amount);
    }
}

/// The parts of a reader between delimiters, without them, from `BufRead::split`.
/// Ends after yielding a cancellation error; other errors do not end it.
#[derive(Debug)]
pub struct Split<B> {
    inner: B,
    delim: u8,
    done: bool,
}

impl<B: BufRead> Iterator for Split<B> {
    type Item = Result<Vec<u8>>;

    fn next(&mut self) -> Option<Result<Vec<u8>>> {
        if self.done {
            return None;
        }
        let mut buf = Vec::new();
        match self.inner.read_until(self.delim, &mut buf) {
            Ok(0) => None,
            Ok(_) => {
                if buf.last() == Some(&self.delim) {
                    buf.pop();
                }
                Some(Ok(buf))
            }
            Err(e) => {
                self.done = e.is_cancelled();
                Some(Err(e))
            }
        }
    }
}

/// The lines of a reader, without `\n` or `\r\n`, from `BufRead::lines`.
/// Ends after yielding a cancellation error; other errors do not end it.
#[derive(Debug)]
pub struct Lines<B> {
    inner: B,
    done: bool,
}

impl<B: BufRead> Iterator for Lines<B> {
    type Item = Result<String>;

    #[track_caller]
    fn next(&mut self) -> Option<Result<String>> {
        if self.done {
            return None;
        }
        let mut buf = String::new();
        match self.inner.read_line(&mut buf) {
            Ok(0) => None,
            Ok(_) => {
                if buf.ends_with('\n') {
                    buf.pop();
                    if buf.ends_with('\r') {
                        buf.pop();
                    }
                }
                Some(Ok(buf))
            }
            Err(e) => {
                self.done = e.is_cancelled();
                Some(Err(e))
            }
        }
    }
}

// ---------------------------------------------------------------- buffered I/O

/// Adds a buffer to a reader, so small reads and `BufRead`'s methods do not each call it.
pub struct BufReader<R: ?Sized> {
    buf: Box<[u8]>,
    pos: usize,
    filled: usize,
    inner: R,
}

impl<R: Read> BufReader<R> {
    pub fn new(inner: R) -> BufReader<R> {
        BufReader::with_capacity(DEFAULT_BUF_SIZE, inner)
    }

    pub fn with_capacity(capacity: usize, inner: R) -> BufReader<R> {
        BufReader {
            buf: vec![0; capacity].into_boxed_slice(),
            pos: 0,
            filled: 0,
            inner,
        }
    }
}

impl<R> BufReader<R> {
    /// The reader; buffered data is lost.
    pub fn into_inner(self) -> R {
        self.inner
    }
}

impl<R: ?Sized> BufReader<R> {
    pub fn get_ref(&self) -> &R {
        &self.inner
    }

    pub fn get_mut(&mut self) -> &mut R {
        &mut self.inner
    }

    /// The buffered data not yet read.
    pub fn buffer(&self) -> &[u8] {
        &self.buf[self.pos..self.filled]
    }

    pub fn capacity(&self) -> usize {
        self.buf.len()
    }

    fn discard_buffer(&mut self) {
        self.pos = 0;
        self.filled = 0;
    }
}

impl<R: ?Sized + Read> Read for BufReader<R> {
    fn read(&mut self, out: &mut [u8]) -> Result<usize> {
        // A read at least as large as the buffer bypasses it once it is empty.
        if self.pos == self.filled && out.len() >= self.capacity() {
            self.discard_buffer();
            return self.inner.read(out);
        }
        let n = self.fill_buf()?.read(out)?;
        self.consume(n);
        Ok(n)
    }
}

impl<R: ?Sized + Read> BufRead for BufReader<R> {
    fn fill_buf(&mut self) -> Result<&[u8]> {
        if self.pos >= self.filled {
            self.discard_buffer();
            self.filled = self.inner.read(&mut self.buf)?;
        }
        Ok(&self.buf[self.pos..self.filled])
    }

    fn consume(&mut self, amount: usize) {
        self.pos = (self.pos + amount).min(self.filled);
    }
}

impl<R: ?Sized + Seek> BufReader<R> {
    /// Seeks relative to the caller's position, keeping the buffer for moves within it.
    pub fn seek_relative(&mut self, offset: i64) -> Result<()> {
        if let Some(pos) = (self.pos as u64).checked_add_signed(offset) {
            if pos <= self.filled as u64 {
                self.pos = pos as usize;
                return Ok(());
            }
        }
        self.seek(SeekFrom::Current(offset)).map(drop)
    }
}

/// `seek` discards the buffer; `SeekFrom::Current` counts from what the caller has read.
impl<R: ?Sized + Seek> Seek for BufReader<R> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        let result = if let SeekFrom::Current(n) = pos {
            let remainder = (self.filled - self.pos) as i64;
            match n.checked_sub(remainder) {
                Some(offset) => self.inner.seek(SeekFrom::Current(offset))?,
                None => {
                    self.inner.seek(SeekFrom::Current(-remainder))?;
                    self.discard_buffer();
                    self.inner.seek(SeekFrom::Current(n))?
                }
            }
        } else {
            self.inner.seek(pos)?
        };
        self.discard_buffer();
        Ok(result)
    }

    fn stream_position(&mut self) -> Result<u64> {
        let remainder = (self.filled - self.pos) as u64;
        self.inner.stream_position().map(|pos| {
            pos.checked_sub(remainder).expect(
                "overflow when subtracting remaining buffer size from inner stream position",
            )
        })
    }

    fn seek_relative(&mut self, offset: i64) -> Result<()> {
        self.seek_relative(offset)
    }
}

impl<R: ?Sized + fmt::Debug> fmt::Debug for BufReader<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BufReader")
            .field("reader", &&self.inner)
            .field(
                "buffer",
                &format_args!("{}/{}", self.filled - self.pos, self.capacity()),
            )
            .finish()
    }
}

/// Adds a buffer to a sized writer, so small writes do not each call it.
/// Per TASK-6, Drop discards pending bytes and warns with their count; call `flush` or
/// `into_inner` to write them.
pub struct BufWriter<W: Write> {
    buf: Vec<u8>,
    /// Set while the inner writer runs; stays true if it panics.
    panicked: bool,
    /// `None` only once `into_inner` has taken it.
    inner: Option<W>,
}

impl<W: Write> BufWriter<W> {
    pub fn new(inner: W) -> BufWriter<W> {
        BufWriter::with_capacity(DEFAULT_BUF_SIZE, inner)
    }

    pub fn with_capacity(capacity: usize, inner: W) -> BufWriter<W> {
        BufWriter {
            buf: Vec::with_capacity(capacity),
            panicked: false,
            inner: Some(inner),
        }
    }

    /// Writes what is buffered and returns the writer; on a failure, the error holds this
    /// `BufWriter` with what is still buffered.
    pub fn into_inner(mut self) -> std::result::Result<W, IntoInnerError<BufWriter<W>>> {
        match self.flush_buf() {
            Err(e) => Err(IntoInnerError(self, e)),
            Ok(()) => Ok(self.inner.take().expect("BufWriter holds its writer")),
        }
    }

    pub fn get_ref(&self) -> &W {
        self.inner.as_ref().expect("BufWriter holds its writer")
    }

    pub fn get_mut(&mut self) -> &mut W {
        self.inner.as_mut().expect("BufWriter holds its writer")
    }

    /// The data buffered and not yet written.
    pub fn buffer(&self) -> &[u8] {
        &self.buf
    }

    pub fn capacity(&self) -> usize {
        self.buf.capacity()
    }

    fn spare_capacity(&self) -> usize {
        self.buf.capacity() - self.buf.len()
    }

    /// Writes the whole buffer; on an error or unwind, keeps only unacknowledged bytes.
    fn flush_buf(&mut self) -> Result<()> {
        struct BufGuard<'a> {
            buf: &'a mut Vec<u8>,
            written: usize,
        }
        impl Drop for BufGuard<'_> {
            fn drop(&mut self) {
                self.buf.drain(..self.written);
            }
        }
        let mut guard = BufGuard {
            buf: &mut self.buf,
            written: 0,
        };
        while guard.written < guard.buf.len() {
            self.panicked = true;
            let r = self
                .inner
                .as_mut()
                .expect("BufWriter holds its writer")
                .write(&guard.buf[guard.written..]);
            self.panicked = false;
            match r {
                Ok(0) => {
                    return Err(Error::message(
                        ErrorKind::WriteZero,
                        "failed to write the buffered data",
                    ));
                }
                Ok(n) => guard.written += n,
                Err(e) if e.is_interrupted() => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    /// Writes `buf` through `write` when it does not fit the buffer, flushing first when the
    /// buffer cannot take it; otherwise buffers it.
    fn write_cold<T>(
        &mut self,
        buf: &[u8],
        write: impl FnOnce(&mut W, &[u8]) -> Result<T>,
        buffered: T,
    ) -> Result<T> {
        if buf.len() > self.spare_capacity() {
            self.flush_buf()?;
        }
        if buf.len() >= self.buf.capacity() {
            self.panicked = true;
            let r = write(self.get_mut(), buf);
            self.panicked = false;
            r
        } else {
            self.buf.extend_from_slice(buf);
            Ok(buffered)
        }
    }
}

impl<W: Write> Write for BufWriter<W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        if buf.len() < self.spare_capacity() {
            self.buf.extend_from_slice(buf);
            Ok(buf.len())
        } else {
            self.write_cold(buf, |w, buf| w.write(buf), buf.len())
        }
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        if buf.len() < self.spare_capacity() {
            self.buf.extend_from_slice(buf);
            Ok(())
        } else {
            self.write_cold(buf, |w, buf| w.write_all(buf), ())
        }
    }

    fn flush(&mut self) -> Result<()> {
        self.flush_buf()?;
        self.get_mut().flush()
    }
}

/// Seeking writes what is buffered first.
impl<W: Write + Seek> Seek for BufWriter<W> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        self.flush_buf()?;
        self.get_mut().seek(pos)
    }
}

impl<W: Write> Drop for BufWriter<W> {
    fn drop(&mut self) {
        if !self.buf.is_empty() {
            crate::__sink::log(crate::log::Level::Warn, format_args!("BufWriter dropped with {} unflushed bytes", self.buf.len()));
        }
    }
}

impl<W: Write + fmt::Debug> fmt::Debug for BufWriter<W> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BufWriter")
            .field(
                "writer",
                &self.inner.as_ref().expect("BufWriter holds its writer"),
            )
            .field(
                "buffer",
                &format_args!("{}/{}", self.buf.len(), self.buf.capacity()),
            )
            .finish()
    }
}

/// The error of `BufWriter::into_inner`: the error, and the `BufWriter` with what is still
/// buffered.
#[derive(Debug)]
pub struct IntoInnerError<W>(W, Error);

impl<W> IntoInnerError<W> {
    pub fn error(&self) -> &Error {
        &self.1
    }

    pub fn into_error(self) -> Error {
        self.1
    }

    pub fn into_inner(self) -> W {
        self.0
    }

    pub fn into_parts(self) -> (Error, W) {
        (self.1, self.0)
    }
}

impl<W> From<IntoInnerError<W>> for Error {
    fn from(e: IntoInnerError<W>) -> Error {
        e.1
    }
}

impl<W> fmt::Display for IntoInnerError<W> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.1, f)
    }
}

impl<W: Send + fmt::Debug> error::Error for IntoInnerError<W> {}

// Shared with File, which checks cancellation even for empty buffers.
pub(crate) fn read_exact<R: Read + ?Sized>(reader: &mut R, mut buf: &mut [u8]) -> Result<()> {
    while !buf.is_empty() {
        match reader.read(buf) {
            Ok(0) => return Err(Error::READ_EXACT_EOF),
            Ok(n) => buf = &mut buf[n..],
            Err(e) if e.is_interrupted() => {}
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

// Shared with File, which checks cancellation even for empty buffers.
pub(crate) fn write_all<W: Write + ?Sized>(writer: &mut W, mut buf: &[u8]) -> Result<()> {
    while !buf.is_empty() {
        match writer.write(buf) {
            Ok(0) => return Err(Error::WRITE_ALL_EOF),
            Ok(n) => buf = &buf[n..],
            Err(e) if e.is_interrupted() => {}
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

// Shared with File, which checks cancellation even for empty buffers.
pub(crate) fn write_fmt<W: Write + ?Sized>(writer: &mut W, args: fmt::Arguments<'_>) -> Result<()> {
    struct Adapter<'a, T: ?Sized> {
        inner: &'a mut T,
        error: Result<()>,
    }
    impl<T: Write + ?Sized> fmt::Write for Adapter<'_, T> {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            self.inner.write_all(s.as_bytes()).map_err(|e| {
                self.error = Err(e);
                fmt::Error
            })
        }
    }
    let mut output = Adapter {
        inner: writer,
        error: Ok(()),
    };
    match fmt::write(&mut output, args) {
        Ok(()) => Ok(()),
        Err(_) if output.error.is_err() => output.error,
        Err(_) => panic!(
            "a formatting trait implementation returned an error when the underlying stream did not"
        ),
    }
}
