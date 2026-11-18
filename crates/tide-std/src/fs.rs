//! File-system authority and handles (reference §6.3).
//!
//! `dir` pins a directory descriptor; absolute paths and resolutions escaping it return
//! `PermissionDenied`, except magic links inside the base fail with ELOOP's raw kind
//! (provisional); `write_synced` gives `PermissionDenied` on the same path.
//! Real files are Linux-only in v0; tide-std does not build on other targets. Confinement
//! requires Linux 5.6+ (`openat2`); unavailable kernels fail closed with `Unsupported`.
//! Relative root paths use the process's starting directory.
//! Listings are snapshots sorted by filename bytes. Metadata exposes type and length only;
//! directory length is zero on both backends. Simulated calls take at most 1 ms and apply
//! whole; links, crashes and faults are not modeled. Attenuation/data accessors do not schedule.
//! Error kinds are the contract; simulated errors carry no OS code and may differ in text.
//! `write_synced` follows up to 40 final symlinks separately from parent lookups; unlike
//! `write`, this is not Linux's total 40-link pathname budget.

use crate::io::{self, Read, Seek, SeekFrom, Write};
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};

#[path = "fs/real.rs"]
mod real;
#[path = "fs/sim.rs"]
mod sim;

/// Access permitted through an `Fs`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Read,
    ReadWrite,
}

/// File-system authority, attenuated with `dir` or `read_only`.
#[derive(Clone)]
pub struct Fs(FsBackend, Mode);
#[derive(Clone)]
enum FsBackend {
    Real(real::Fs),
    Sim(sim::Fs),
}

/// Options for `Fs::open_with`; this data cannot open a file by itself.
#[derive(Clone, Debug, Default)]
pub struct OpenOptions {
    read: bool,
    write: bool,
    append: bool,
    truncate: bool,
    create: bool,
    create_new: bool,
}

impl OpenOptions {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn read(&mut self, value: bool) -> &mut Self {
        self.read = value;
        self
    }
    pub fn write(&mut self, value: bool) -> &mut Self {
        self.write = value;
        self
    }
    pub fn append(&mut self, value: bool) -> &mut Self {
        self.append = value;
        self
    }
    pub fn truncate(&mut self, value: bool) -> &mut Self {
        self.truncate = value;
        self
    }
    pub fn create(&mut self, value: bool) -> &mut Self {
        self.create = value;
        self
    }
    pub fn create_new(&mut self, value: bool) -> &mut Self {
        self.create_new = value;
        self
    }

    fn writes(&self) -> bool {
        self.write || self.append || self.truncate || self.create || self.create_new
    }

    fn validate(&self) -> io::Result<()> {
        if !(self.read || self.write || self.append)
            || (!self.write && !self.append && (self.truncate || self.create || self.create_new))
            || (self.append && self.truncate && !self.create_new)
        {
            return Err(io::ErrorKind::InvalidInput.into());
        }
        Ok(())
    }
}

// Real multi-call operations also check between their syscalls.
pub(super) fn checkpoint() -> io::Result<()> {
    crate::task::outside_simulation();
    if crate::task::observe_cancellation() {
        Err(io::ErrorKind::Cancelled.into())
    } else {
        Ok(())
    }
}

fn check_path(p: &Path, confined: bool) -> io::Result<()> {
    if confined && p.is_absolute() {
        return Err(io::ErrorKind::PermissionDenied.into());
    }
    if p.as_os_str().as_encoded_bytes().contains(&0) {
        return Err(io::ErrorKind::InvalidInput.into());
    }
    Ok(())
}
fn normal_name(p: &Path) -> io::Result<()> {
    let bytes = p.as_os_str().as_encoded_bytes();
    if bytes.is_empty() {
        return Err(io::ErrorKind::NotFound.into());
    }
    let name = trim_final_slashes(bytes)
        .rsplit(|&b| b == b'/')
        .next()
        .unwrap();
    if name.is_empty() || name == b"." || name == b".." {
        return Err(io::ErrorKind::InvalidInput.into());
    }
    Ok(())
}

fn trim_final_slashes(bytes: &[u8]) -> &[u8] {
    let end = bytes.iter().rposition(|&b| b != b'/').map_or(0, |i| i + 1);
    &bytes[..end]
}

// Preserve the raw suffix: Path::file_name discards trailing slashes and `.`.
fn split(p: &Path) -> io::Result<(PathBuf, OsString)> {
    let bytes = p.as_os_str().as_bytes();
    if bytes.is_empty() {
        return Err(io::ErrorKind::NotFound.into());
    }
    let end = trim_final_slashes(bytes).len();
    if end == 0 {
        return Ok((p.to_owned(), OsString::from(".")));
    }
    let start = bytes[..end]
        .iter()
        .rposition(|&b| b == b'/')
        .map_or(0, |i| i + 1);
    let parent = if start == 0 {
        Path::new(".")
    } else {
        Path::new(OsStr::from_bytes(&bytes[..start]))
    };
    Ok((
        parent.to_owned(),
        OsString::from_vec(bytes[start..].to_vec()),
    ))
}
impl Fs {
    pub(crate) fn root() -> Self {
        Self(FsBackend::Real(real::Fs::default()), Mode::ReadWrite)
    }
    pub(crate) fn simulated(tasks: &crate::task::Tasks) -> Self {
        Self(FsBackend::Sim(sim::Fs::root(tasks)), Mode::ReadWrite)
    }

    #[track_caller]
    fn run<T>(
        &self,
        name: &'static str,
        p: &Path,
        write: bool,
        f: impl FnOnce() -> io::Result<T>,
    ) -> io::Result<T> {
        let apply = || {
            if write && self.1 == Mode::Read {
                return Err(io::ErrorKind::PermissionDenied.into());
            }
            self.check_path(p)?;
            f()
        };
        match &self.0 {
            FsBackend::Real(_) => {
                checkpoint()?;
                apply()
            }
            FsBackend::Sim(fs) => {
                fs.tasks
                    .file_op(name, p, false, std::panic::Location::caller(), apply)
            }
        }
    }
    fn check_path(&self, p: &Path) -> io::Result<()> {
        let confined = match &self.0 {
            FsBackend::Real(fs) => fs.is_confined(),
            FsBackend::Sim(fs) => fs.is_confined(),
        };
        check_path(p, confined)
    }
    /// Attenuation is data-only: it does not perform I/O or schedule.
    pub fn read_only(&self) -> Self {
        Self(self.0.clone(), Mode::Read)
    }

    #[track_caller]
    pub fn dir(&self, p: impl AsRef<Path>, mode: Mode) -> io::Result<Self> {
        let p = p.as_ref();
        self.run("fs.dir", p, mode == Mode::ReadWrite, || {
            Ok(Self(
                match &self.0 {
                    FsBackend::Real(fs) => FsBackend::Real(fs.dir(p)?),
                    FsBackend::Sim(fs) => FsBackend::Sim(fs.dir(p)?),
                },
                mode,
            ))
        })
    }
    #[track_caller]
    pub fn open(&self, p: impl AsRef<Path>) -> io::Result<File> {
        self.open_options("fs.open", p.as_ref(), OpenOptions::new().read(true))
    }
    #[track_caller]
    pub fn create(&self, p: impl AsRef<Path>) -> io::Result<File> {
        self.open_options(
            "fs.create",
            p.as_ref(),
            OpenOptions::new().write(true).create(true).truncate(true),
        )
    }
    #[track_caller]
    pub fn open_with(&self, p: impl AsRef<Path>, opts: &OpenOptions) -> io::Result<File> {
        self.open_options("fs.open_with", p.as_ref(), opts)
    }
    #[track_caller]
    fn open_options(&self, name: &'static str, p: &Path, opts: &OpenOptions) -> io::Result<File> {
        self.run(name, p, opts.writes(), || {
            opts.validate()?;
            Ok(File {
                backend: match &self.0 {
                    FsBackend::Real(fs) => FileBackend::Real(fs.open(p, opts)?),
                    FsBackend::Sim(fs) => FileBackend::Sim(fs.open(p, opts)?),
                },
                read: opts.read,
                write: opts.write || opts.append,
            })
        })
    }
    #[track_caller]
    pub fn read(&self, p: impl AsRef<Path>) -> io::Result<Vec<u8>> {
        let p = p.as_ref();
        self.run("fs.read", p, false, || self.read_bytes(p))
    }
    fn read_bytes(&self, p: &Path) -> io::Result<Vec<u8>> {
        match &self.0 {
            FsBackend::Real(_) => {
                let mut bytes = Vec::new();
                self.open(p)?.read_to_end(&mut bytes)?;
                Ok(bytes)
            }
            FsBackend::Sim(fs) => fs.read(p),
        }
    }
    #[track_caller]
    pub fn read_to_string(&self, p: impl AsRef<Path>) -> io::Result<String> {
        let p = p.as_ref();
        self.run("fs.read_to_string", p, false, || {
            String::from_utf8(self.read_bytes(p)?).map_err(|_| io::ErrorKind::InvalidData.into())
        })
    }
    #[track_caller]
    pub fn write(&self, p: impl AsRef<Path>, data: impl AsRef<[u8]>) -> io::Result<()> {
        let p = p.as_ref();
        self.run("fs.write", p, true, || match &self.0 {
            FsBackend::Real(_) => self.create(p)?.write_all(data.as_ref()),
            FsBackend::Sim(fs) => fs.write(p, data.as_ref()),
        })
    }
    #[track_caller]
    pub fn write_synced(&self, p: impl AsRef<Path>, data: impl AsRef<[u8]>) -> io::Result<()> {
        let p = p.as_ref();
        self.run("fs.write_synced", p, true, || match &self.0 {
            FsBackend::Real(fs) => fs.write_synced(p, data.as_ref()),
            FsBackend::Sim(fs) => fs.write_synced(p, data.as_ref()),
        })
    }
    #[track_caller]
    pub fn metadata(&self, p: impl AsRef<Path>) -> io::Result<Metadata> {
        let p = p.as_ref();
        self.run("fs.metadata", p, false, || match &self.0 {
            FsBackend::Real(fs) => fs.metadata(p),
            FsBackend::Sim(fs) => fs.metadata(p),
        })
    }
    #[track_caller]
    pub fn remove_file(&self, p: impl AsRef<Path>) -> io::Result<()> {
        let p = p.as_ref();
        self.run("fs.remove_file", p, true, || {
            normal_name(p)?;
            match &self.0 {
                FsBackend::Real(fs) => fs.remove(p, false),
                FsBackend::Sim(fs) => fs.remove(p, false),
            }
        })
    }
    #[track_caller]
    pub fn remove_dir(&self, p: impl AsRef<Path>) -> io::Result<()> {
        let p = p.as_ref();
        self.run("fs.remove_dir", p, true, || {
            normal_name(p)?;
            match &self.0 {
                FsBackend::Real(fs) => fs.remove(p, true),
                FsBackend::Sim(fs) => fs.remove(p, true),
            }
        })
    }
    #[track_caller]
    pub fn remove_dir_all(&self, p: impl AsRef<Path>) -> io::Result<()> {
        let p = p.as_ref();
        self.run("fs.remove_dir_all", p, true, || match &self.0 {
            FsBackend::Real(fs) => fs.remove_dir_all(p),
            FsBackend::Sim(fs) => fs.remove_dir_all(p),
        })
    }
    #[track_caller]
    pub fn create_dir_all(&self, p: impl AsRef<Path>) -> io::Result<()> {
        let p = p.as_ref();
        self.run("fs.create_dir_all", p, true, || match &self.0 {
            FsBackend::Real(fs) => fs.create_dir_all(p),
            FsBackend::Sim(fs) => fs.create_dir_all(p),
        })
    }
    #[track_caller]
    pub fn sync_dir(&self, p: impl AsRef<Path>) -> io::Result<()> {
        let p = p.as_ref();
        self.run("fs.sync_dir", p, false, || match &self.0 {
            FsBackend::Real(fs) => fs.sync_dir(p),
            FsBackend::Sim(fs) => fs.sync_dir(p),
        })
    }
    #[track_caller]
    pub fn exists(&self, p: impl AsRef<Path>) -> io::Result<bool> {
        let p = p.as_ref();
        self.run("fs.exists", p, false, || {
            let result = match &self.0 {
                FsBackend::Real(fs) => fs.metadata(p),
                FsBackend::Sim(fs) => fs.metadata(p),
            };
            match result {
                Ok(_) => Ok(true),
                Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
                Err(e) => Err(e),
            }
        })
    }
    #[track_caller]
    pub fn rename(&self, from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<()> {
        let (from, to) = (from.as_ref(), to.as_ref());
        self.run("fs.rename", from, true, || {
            self.check_path(to)?;
            normal_name(from)?;
            normal_name(to)?;
            match &self.0 {
                FsBackend::Real(fs) => fs.rename(from, to),
                FsBackend::Sim(fs) => fs.rename(from, to),
            }
        })
    }
    #[track_caller]
    pub fn read_dir(&self, p: impl AsRef<Path>) -> io::Result<ReadDir> {
        let p = p.as_ref();
        self.run("fs.read_dir", p, false, || {
            Ok(ReadDir {
                backend: match &self.0 {
                    FsBackend::Real(fs) => ReadDirBackend::Real(fs.read_dir(p)?.into_iter()),
                    FsBackend::Sim(fs) => ReadDirBackend::Sim {
                        entries: fs.read_dir(p)?.into_iter(),
                        fs: fs.clone(),
                        path: p.to_owned(),
                    },
                },
                finished: false,
            })
        })
    }
    #[track_caller]
    pub(crate) fn world_file(&self, p: &Path, data: &[u8]) {
        let FsBackend::Sim(fs) = &self.0 else {
            unreachable!()
        };
        fs.world_file(p, data);
    }
}

/// An open file. Access flags are checked before reaching its backend.
pub struct File {
    backend: FileBackend,
    read: bool,
    write: bool,
}
enum FileBackend {
    Real(std::fs::File),
    Sim(sim::File),
}
impl std::fmt::Debug for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("File")
            .field("read", &self.read)
            .field("write", &self.write)
            .finish()
    }
}
impl File {
    /// The descriptor of a real file, for `Stdio::from`; `None` for a simulated one.
    pub(crate) fn into_fd(self) -> Option<std::os::fd::OwnedFd> {
        match self.backend {
            FileBackend::Real(file) => Some(file.into()),
            FileBackend::Sim(_) => None,
        }
    }
    // Sinks are not checkpoints and cannot observe cancellation or report a write failure.
    pub(crate) fn write_sink(&self, bytes: &[u8]) {
        if let FileBackend::Real(file) = &self.backend {
            crate::task::outside_simulation();
            if self.write { let _ = std::io::Write::write_all(&mut &*file, bytes); }
        }
    }

    #[track_caller]
    fn run<T>(
        &self,
        name: &'static str,
        allowed: bool,
        f: impl FnOnce() -> io::Result<T>,
    ) -> io::Result<T> {
        let apply = || {
            if !allowed {
                return Err(io::ErrorKind::PermissionDenied.into());
            }
            f()
        };
        match &self.backend {
            FileBackend::Real(_) => {
                checkpoint()?;
                apply()
            }
            FileBackend::Sim(file) => file.tasks.file_op(
                name,
                &file.path,
                false,
                std::panic::Location::caller(),
                apply,
            ),
        }
    }
    #[track_caller]
    pub fn sync(&self) -> io::Result<()> {
        self.run("file.sync", true, || match &self.backend {
            FileBackend::Real(file) => real::sync_all(file),
            FileBackend::Sim(_) => Ok(()),
        })
    }
    #[track_caller]
    pub fn sync_all(&self) -> io::Result<()> {
        self.run("file.sync_all", true, || match &self.backend {
            FileBackend::Real(file) => real::sync_all(file),
            FileBackend::Sim(_) => Ok(()),
        })
    }
    #[track_caller]
    pub fn sync_data(&self) -> io::Result<()> {
        self.run("file.sync_data", true, || match &self.backend {
            FileBackend::Real(file) => real::sync_data(file),
            FileBackend::Sim(_) => Ok(()),
        })
    }
    #[track_caller]
    pub fn set_len(&self, n: u64) -> io::Result<()> {
        self.run("file.set_len", self.write, || match &self.backend {
            FileBackend::Real(file) => real::set_len(file, n),
            FileBackend::Sim(file) => file.set_len(n),
        })
    }
    #[track_caller]
    pub fn metadata(&self) -> io::Result<Metadata> {
        self.run("file.metadata", true, || match &self.backend {
            FileBackend::Real(file) => real::file_metadata(file),
            FileBackend::Sim(file) => Ok(file.metadata()),
        })
    }
    /// Always closes, even in a cancelled scope. A failed close must never be retried.
    #[track_caller]
    pub fn close(self) -> io::Result<()> {
        match self.backend {
            FileBackend::Real(file) => {
                crate::task::outside_simulation();
                real::close(file)
            }
            FileBackend::Sim(file) => {
                let tasks = file.tasks.clone();
                let path = file.path.clone();
                tasks.file_op(
                    "file.close",
                    &path,
                    true,
                    std::panic::Location::caller(),
                    || {
                        drop(file);
                        Ok(())
                    },
                )
            }
        }
    }
}
impl Read for &File {
    #[track_caller]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.run("file.read", self.read, || match &self.backend {
            FileBackend::Real(file) => {
                std::io::Read::read(&mut &*file, buf).map_err(real::io_error)
            }
            FileBackend::Sim(file) => file.cursor(false, |cursor| cursor.read(buf)),
        })
    }
    #[track_caller]
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.run("file.read_exact", self.read, || match &self.backend {
            FileBackend::Real(file) => real::Reader(file).read_exact(buf),
            FileBackend::Sim(_) if buf.is_empty() => Ok(()),
            FileBackend::Sim(file) => file.cursor(false, |cursor| io::read_exact(cursor, buf)),
        })
    }
    #[track_caller]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.run("file.read_to_end", self.read, || match &self.backend {
            FileBackend::Real(file) => real::Reader(file).read_to_end(buf),
            FileBackend::Sim(file) => file.cursor(false, |cursor| cursor.read_to_end(buf)),
        })
    }
    #[track_caller]
    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        self.run("file.read_to_string", self.read, || match &self.backend {
            FileBackend::Real(file) => real::Reader(file).read_to_string(buf),
            FileBackend::Sim(file) => file.cursor(false, |cursor| cursor.read_to_string(buf)),
        })
    }
}
impl Write for &File {
    #[track_caller]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.run("file.write", self.write, || match &self.backend {
            FileBackend::Real(file) => {
                std::io::Write::write(&mut &*file, buf).map_err(real::io_error)
            }
            FileBackend::Sim(file) => file.write(buf),
        })
    }
    #[track_caller]
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.run("file.write_all", self.write, || match &self.backend {
            FileBackend::Real(_) => io::write_all(&mut &**self, buf),
            FileBackend::Sim(file) => file.write(buf).map(|_| ()),
        })
    }
    #[track_caller]
    fn write_fmt(&mut self, args: std::fmt::Arguments<'_>) -> io::Result<()> {
        // Format outside the store lock: user Display implementations may themselves do I/O.
        self.run("file.write_fmt", self.write, || match &self.backend {
            FileBackend::Real(_) => io::write_fmt(&mut &**self, args),
            FileBackend::Sim(file) => {
                let mut bytes = Vec::new();
                bytes.write_fmt(args)?;
                file.write(&bytes).map(|_| ())
            }
        })
    }
    #[track_caller]
    fn flush(&mut self) -> io::Result<()> {
        self.run("file.flush", true, || Ok(()))
    }
}
impl Seek for &File {
    #[track_caller]
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.run("file.seek", true, || match &self.backend {
            FileBackend::Real(file) => {
                std::io::Seek::seek(&mut &*file, pos).map_err(real::io_error)
            }
            FileBackend::Sim(file) => file.seek(pos),
        })
    }
}
impl Read for File {
    #[track_caller]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        Read::read(&mut &*self, buf)
    }
    #[track_caller]
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        Read::read_exact(&mut &*self, buf)
    }
    #[track_caller]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        Read::read_to_end(&mut &*self, buf)
    }
    #[track_caller]
    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        Read::read_to_string(&mut &*self, buf)
    }
}
impl Write for File {
    #[track_caller]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Write::write(&mut &*self, buf)
    }
    #[track_caller]
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        Write::write_all(&mut &*self, buf)
    }
    #[track_caller]
    fn write_fmt(&mut self, args: std::fmt::Arguments<'_>) -> io::Result<()> {
        Write::write_fmt(&mut &*self, args)
    }
    #[track_caller]
    fn flush(&mut self) -> io::Result<()> {
        Write::flush(&mut &*self)
    }
}
impl Seek for File {
    #[track_caller]
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        Seek::seek(&mut &*self, pos)
    }
}

/// Snapshot of directory names, sorted by filename bytes.
#[derive(Debug)]
pub struct ReadDir {
    backend: ReadDirBackend,
    finished: bool,
}
#[derive(Debug)]
enum ReadDirBackend {
    Real(std::vec::IntoIter<DirEntry>),
    Sim {
        entries: std::vec::IntoIter<DirEntry>,
        fs: sim::Fs,
        path: PathBuf,
    },
}
impl Iterator for ReadDir {
    type Item = io::Result<DirEntry>;
    #[track_caller]
    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        let result = match &mut self.backend {
            ReadDirBackend::Real(entries) => checkpoint().map(|()| entries.next()),
            ReadDirBackend::Sim { entries, fs, path } => fs.tasks.file_op(
                "read_dir.next",
                path,
                false,
                std::panic::Location::caller(),
                || Ok(entries.next()),
            ),
        };
        match result {
            Ok(None) => {
                self.finished = true;
                None
            }
            Ok(Some(entry)) => Some(Ok(entry)),
            Err(e) => {
                self.finished = true;
                Some(Err(e))
            }
        }
    }
}
/// An entry keeps its listing's directory, including after a rename.
#[derive(Debug)]
pub struct DirEntry {
    backend: DirEntryBackend,
    path: PathBuf,
    name: OsString,
}
#[derive(Debug)]
enum DirEntryBackend {
    Real(real::Entry),
    Sim(sim::Fs),
}
impl DirEntry {
    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }
    pub fn file_name(&self) -> OsString {
        self.name.clone()
    }
    #[track_caller]
    pub fn metadata(&self) -> io::Result<Metadata> {
        self.stat("dir_entry.metadata")
    }
    #[track_caller]
    pub fn file_type(&self) -> io::Result<FileType> {
        Ok(self.stat("dir_entry.file_type")?.file_type())
    }
    #[track_caller]
    fn stat(&self, name: &'static str) -> io::Result<Metadata> {
        match &self.backend {
            DirEntryBackend::Real(entry) => {
                checkpoint()?;
                entry.metadata(&self.name)
            }
            DirEntryBackend::Sim(fs) => fs.tasks.file_op(
                name,
                &self.path,
                false,
                std::panic::Location::caller(),
                || fs.metadata(Path::new(&self.name)),
            ),
        }
    }
}

/// Type and length of a file-system object; no authority is retained.
#[derive(Clone, Debug)]
pub struct Metadata {
    file_type: FileType,
    len: u64,
}
impl Metadata {
    fn new(kind: Kind, len: u64) -> Self {
        Self {
            file_type: FileType(kind),
            len: if kind == Kind::Dir { 0 } else { len },
        }
    }
    pub fn file_type(&self) -> FileType {
        self.file_type
    }
    pub fn is_dir(&self) -> bool {
        self.file_type.is_dir()
    }
    pub fn is_file(&self) -> bool {
        self.file_type.is_file()
    }
    pub fn is_symlink(&self) -> bool {
        self.file_type.is_symlink()
    }
    /// File length in bytes; directories return zero on both backends.
    pub fn len(&self) -> u64 {
        self.len
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Kind {
    File,
    Dir,
    Symlink,
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FileType(Kind);
impl FileType {
    pub fn is_dir(&self) -> bool {
        self.0 == Kind::Dir
    }
    pub fn is_file(&self) -> bool {
        self.0 == Kind::File
    }
    pub fn is_symlink(&self) -> bool {
        self.0 == Kind::Symlink
    }
}
