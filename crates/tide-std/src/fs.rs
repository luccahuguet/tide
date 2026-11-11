//! File-system authority and handles (reference §6.3).
//!
//! `dir` pins a directory descriptor; absolute paths and resolutions escaping it return
//! `PermissionDenied`, except magic links inside the base fail with ELOOP's raw kind
//! (provisional); `write_synced` gives `PermissionDenied` on the same path.
//! Real files are Linux-only in v0; tide-std does not build on other targets. Confinement
//! requires Linux 5.6+ (`openat2`); unavailable kernels fail closed with `Unsupported`.
//! Relative root paths use the process's starting directory.
//! Listings are snapshots sorted by filename bytes. Metadata exposes type and length only.
//! Error kinds are the contract; a future simulated backend need not reproduce OS codes or text.
//! `write_synced` follows up to 40 final symlinks separately from parent lookups; unlike
//! `write`, this is not Linux's total 40-link pathname budget.

use crate::io::{self, Read, Seek, SeekFrom, Write};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

#[path = "fs/real.rs"]
mod real;

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

// Every file operation enters here or through `File::close`. Each `Fs` is real until
// fx-byk.2, so its I/O would run outside a simulation's schedule.
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
    let end = bytes.iter().rposition(|&b| b != b'/').map_or(0, |i| i + 1);
    let name = bytes[..end].rsplit(|&b| b == b'/').next().unwrap();
    if name.is_empty() || name == b"." || name == b".." {
        return Err(io::ErrorKind::InvalidInput.into());
    }
    Ok(())
}

impl Fs {
    pub(crate) fn root() -> Self {
        Self(FsBackend::Real(real::Fs::default()), Mode::ReadWrite)
    }

    fn check(&self, write: bool) -> io::Result<()> {
        checkpoint()?;
        if write && self.1 == Mode::Read {
            return Err(io::ErrorKind::PermissionDenied.into());
        }
        Ok(())
    }

    fn check_path(&self, p: &Path) -> io::Result<()> {
        match &self.0 {
            FsBackend::Real(fs) => check_path(p, fs.is_confined()),
        }
    }

    #[track_caller]
    pub fn dir(&self, p: impl AsRef<Path>, mode: Mode) -> io::Result<Self> {
        self.check(mode == Mode::ReadWrite)?;
        let p = p.as_ref();
        self.check_path(p)?;
        match &self.0 {
            FsBackend::Real(fs) => Ok(Self(FsBackend::Real(fs.dir(p)?), mode)),
        }
    }

    pub fn read_only(&self) -> Self {
        Self(self.0.clone(), Mode::Read)
    }

    #[track_caller]
    pub fn open(&self, p: impl AsRef<Path>) -> io::Result<File> {
        self.open_with(p, OpenOptions::new().read(true))
    }

    #[track_caller]
    pub fn create(&self, p: impl AsRef<Path>) -> io::Result<File> {
        self.open_with(
            p,
            OpenOptions::new().write(true).create(true).truncate(true),
        )
    }

    #[track_caller]
    pub fn open_with(&self, p: impl AsRef<Path>, opts: &OpenOptions) -> io::Result<File> {
        self.check(opts.writes())?;
        let p = p.as_ref();
        self.check_path(p)?;
        opts.validate()?;
        match &self.0 {
            FsBackend::Real(fs) => Ok(File {
                backend: FileBackend::Real(fs.open(p, opts)?),
                read: opts.read,
                write: opts.write || opts.append,
            }),
        }
    }

    #[track_caller]
    pub fn read(&self, p: impl AsRef<Path>) -> io::Result<Vec<u8>> {
        let mut file = self.open(p)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        Ok(bytes)
    }

    #[track_caller]
    pub fn read_to_string(&self, p: impl AsRef<Path>) -> io::Result<String> {
        let mut file = self.open(p)?;
        let mut text = String::new();
        file.read_to_string(&mut text)?;
        Ok(text)
    }

    #[track_caller]
    pub fn write(&self, p: impl AsRef<Path>, data: impl AsRef<[u8]>) -> io::Result<()> {
        self.create(p)?.write_all(data.as_ref())
    }

    #[track_caller]
    pub fn write_synced(&self, p: impl AsRef<Path>, data: impl AsRef<[u8]>) -> io::Result<()> {
        self.check(true)?;
        let p = p.as_ref();
        self.check_path(p)?;
        match &self.0 {
            FsBackend::Real(fs) => fs.write_synced(p, data.as_ref()),
        }
    }

    #[track_caller]
    pub fn metadata(&self, p: impl AsRef<Path>) -> io::Result<Metadata> {
        self.check(false)?;
        let p = p.as_ref();
        self.check_path(p)?;
        match &self.0 {
            FsBackend::Real(fs) => fs.metadata(p),
        }
    }

    #[track_caller]
    pub fn exists(&self, p: impl AsRef<Path>) -> io::Result<bool> {
        match self.metadata(p) {
            Ok(_) => Ok(true),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(e),
        }
    }

    #[track_caller]
    pub fn read_dir(&self, p: impl AsRef<Path>) -> io::Result<ReadDir> {
        self.check(false)?;
        let p = p.as_ref();
        self.check_path(p)?;
        match &self.0 {
            FsBackend::Real(fs) => Ok(ReadDir {
                backend: ReadDirBackend::Real(fs.read_dir(p)?.into_iter()),
                finished: false,
            }),
        }
    }

    #[track_caller]
    pub fn rename(&self, from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<()> {
        self.check(true)?;
        let (from, to) = (from.as_ref(), to.as_ref());
        self.check_path(from)?;
        self.check_path(to)?;
        normal_name(from)?;
        normal_name(to)?;
        match &self.0 {
            FsBackend::Real(fs) => fs.rename(from, to),
        }
    }

    #[track_caller]
    pub fn remove_file(&self, p: impl AsRef<Path>) -> io::Result<()> {
        self.check(true)?;
        let p = p.as_ref();
        self.check_path(p)?;
        normal_name(p)?;
        match &self.0 {
            FsBackend::Real(fs) => fs.remove(p, false),
        }
    }

    #[track_caller]
    pub fn remove_dir(&self, p: impl AsRef<Path>) -> io::Result<()> {
        self.check(true)?;
        let p = p.as_ref();
        self.check_path(p)?;
        normal_name(p)?;
        match &self.0 {
            FsBackend::Real(fs) => fs.remove(p, true),
        }
    }

    #[track_caller]
    pub fn remove_dir_all(&self, p: impl AsRef<Path>) -> io::Result<()> {
        self.check(true)?;
        let p = p.as_ref();
        self.check_path(p)?;
        match &self.0 {
            FsBackend::Real(fs) => fs.remove_dir_all(p),
        }
    }

    #[track_caller]
    pub fn create_dir_all(&self, p: impl AsRef<Path>) -> io::Result<()> {
        self.check(true)?;
        let p = p.as_ref();
        self.check_path(p)?;
        match &self.0 {
            FsBackend::Real(fs) => fs.create_dir_all(p),
        }
    }

    #[track_caller]
    pub fn sync_dir(&self, p: impl AsRef<Path>) -> io::Result<()> {
        self.check(false)?;
        let p = p.as_ref();
        self.check_path(p)?;
        match &self.0 {
            FsBackend::Real(fs) => fs.sync_dir(p),
        }
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
    fn check(&self, allowed: bool) -> io::Result<()> {
        checkpoint()?;
        if !allowed {
            return Err(io::ErrorKind::PermissionDenied.into());
        }
        Ok(())
    }

    #[track_caller]
    pub fn sync(&self) -> io::Result<()> {
        self.sync_all()
    }

    #[track_caller]
    pub fn sync_all(&self) -> io::Result<()> {
        self.check(true)?;
        match &self.backend {
            FileBackend::Real(file) => real::sync_all(file),
        }
    }

    #[track_caller]
    pub fn sync_data(&self) -> io::Result<()> {
        self.check(true)?;
        match &self.backend {
            FileBackend::Real(file) => real::sync_data(file),
        }
    }

    #[track_caller]
    pub fn set_len(&self, n: u64) -> io::Result<()> {
        self.check(self.write)?;
        match &self.backend {
            FileBackend::Real(file) => real::set_len(file, n),
        }
    }

    #[track_caller]
    pub fn metadata(&self) -> io::Result<Metadata> {
        self.check(true)?;
        match &self.backend {
            FileBackend::Real(file) => real::file_metadata(file),
        }
    }

    /// Always closes, even in a cancelled scope. A failed close must never be retried.
    #[track_caller]
    pub fn close(self) -> io::Result<()> {
        crate::task::outside_simulation();
        match self.backend {
            FileBackend::Real(file) => real::close(file),
        }
    }
}

impl Read for &File {
    #[track_caller]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.check(self.read)?;
        match &self.backend {
            FileBackend::Real(file) => {
                std::io::Read::read(&mut &*file, buf).map_err(real::io_error)
            }
        }
    }
    #[track_caller]
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.check(self.read)?;
        io::read_exact(self, buf)
    }
}
impl Write for &File {
    #[track_caller]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.check(self.write)?;
        match &self.backend {
            FileBackend::Real(file) => {
                std::io::Write::write(&mut &*file, buf).map_err(real::io_error)
            }
        }
    }
    #[track_caller]
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.check(self.write)?;
        io::write_all(self, buf)
    }
    #[track_caller]
    fn write_fmt(&mut self, args: std::fmt::Arguments<'_>) -> io::Result<()> {
        self.check(self.write)?;
        io::write_fmt(self, args)
    }
    #[track_caller]
    fn flush(&mut self) -> io::Result<()> {
        self.check(true)
    }
}
impl Seek for &File {
    #[track_caller]
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.check(true)?;
        match &self.backend {
            FileBackend::Real(file) => {
                std::io::Seek::seek(&mut &*file, pos).map_err(real::io_error)
            }
        }
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

/// Snapshot of a directory's names, sorted by filename bytes.
#[derive(Debug)]
pub struct ReadDir {
    backend: ReadDirBackend,
    finished: bool,
}
#[derive(Debug)]
enum ReadDirBackend {
    Real(std::vec::IntoIter<DirEntry>),
}
impl Iterator for ReadDir {
    type Item = io::Result<DirEntry>;
    #[track_caller]
    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        if let Err(e) = checkpoint() {
            self.finished = true;
            return Some(Err(e));
        }
        let entry = match &mut self.backend {
            ReadDirBackend::Real(entries) => entries.next(),
        };
        if entry.is_none() {
            self.finished = true;
        }
        entry.map(Ok)
    }
}

/// An entry keeps its listing's directory descriptor, including after a rename.
#[derive(Debug)]
pub struct DirEntry {
    backend: DirEntryBackend,
    path: PathBuf,
    name: OsString,
}
#[derive(Debug)]
enum DirEntryBackend {
    Real(real::Entry),
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
        checkpoint()?;
        match &self.backend {
            DirEntryBackend::Real(entry) => entry.metadata(&self.name),
        }
    }
    #[track_caller]
    pub fn file_type(&self) -> io::Result<FileType> {
        Ok(self.metadata()?.file_type())
    }
}

/// Type and length of a file-system object; no authority is retained.
#[derive(Clone, Debug)]
pub struct Metadata {
    file_type: FileType,
    len: u64,
}
impl Metadata {
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
