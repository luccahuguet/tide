//! Capabilities and synchronization handles. Constructors are crate-private,
//! so only `__rt::root()` can create them.

pub mod fs {
    use std::io;
    use std::path::{Component, Path, PathBuf};

    /// File-system authority, optionally confined below one directory.
    /// Spike: confinement is lexical (no `..`, no absolute paths); symlinks
    /// are not handled.
    #[derive(Clone, Debug)]
    pub struct Fs {
        base: Option<PathBuf>,
    }

    impl Fs {
        pub(crate) fn root() -> Fs {
            Fs { base: None }
        }

        fn resolve(&self, p: &Path) -> io::Result<PathBuf> {
            let Some(base) = &self.base else { return Ok(p.to_path_buf()) };
            if p.components().all(|c| matches!(c, Component::Normal(_) | Component::CurDir)) {
                Ok(base.join(p))
            } else {
                Err(io::Error::new(io::ErrorKind::PermissionDenied, format!("{} escapes {}", p.display(), base.display())))
            }
        }

        pub fn read(&self, p: impl AsRef<Path>) -> io::Result<Vec<u8>> {
            std::fs::read(self.resolve(p.as_ref())?)
        }

        pub fn read_to_string(&self, p: impl AsRef<Path>) -> io::Result<String> {
            std::fs::read_to_string(self.resolve(p.as_ref())?)
        }

        pub fn write(&self, p: impl AsRef<Path>, data: impl AsRef<[u8]>) -> io::Result<()> {
            std::fs::write(self.resolve(p.as_ref())?, data)
        }

        pub fn create_dir_all(&self, p: impl AsRef<Path>) -> io::Result<()> {
            std::fs::create_dir_all(self.resolve(p.as_ref())?)
        }

        /// Attenuation: an `Fs` confined below `p`, which must be a directory.
        pub fn dir(&self, p: impl AsRef<Path>) -> io::Result<Fs> {
            let d = self.resolve(p.as_ref())?;
            if !std::fs::metadata(&d)?.is_dir() {
                return Err(io::Error::new(io::ErrorKind::NotADirectory, d.display().to_string()));
            }
            Ok(Fs { base: Some(d) })
        }
    }
}

pub mod time {
    pub use std::time::Duration;

    /// Monotonic time authority.
    #[derive(Clone, Debug)]
    pub struct Clock(pub(crate) ());

    impl Clock {
        pub fn now(&self) -> Instant {
            Instant(std::time::Instant::now())
        }

        pub fn sleep(&self, d: Duration) {
            std::thread::sleep(d)
        }
    }

    /// Tide's `Instant`: a distinct type with no `now()` or `elapsed()`, so
    /// reading the time needs a `Clock`.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Instant(std::time::Instant);

    impl Instant {
        pub fn duration_since(&self, earlier: Instant) -> Duration {
            self.0.duration_since(earlier.0)
        }
    }

    impl std::ops::Add<Duration> for Instant {
        type Output = Instant;
        fn add(self, d: Duration) -> Instant {
            Instant(self.0 + d)
        }
    }

    impl std::ops::Sub for Instant {
        type Output = Duration;
        fn sub(self, o: Instant) -> Duration {
            self.0 - o.0
        }
    }
}

pub mod task {
    /// Authority to run tasks and create synchronization handles.
    #[derive(Clone, Debug)]
    pub struct Tasks(pub(crate) ());

    impl Tasks {
        /// Structured tasks over `std::thread::scope`: every child finishes
        /// before `scope` returns, and children may borrow from `'env`.
        pub fn scope<'env, F, T>(&'env self, f: F) -> T
        where
            F: for<'scope> FnOnce(Scope<'scope, 'env>) -> T,
        {
            std::thread::scope(|s| f(Scope { inner: s }))
        }
    }

    /// Passed by value, as in Tide (`Copy + Send + Sync`).
    #[derive(Clone, Copy)]
    pub struct Scope<'scope, 'env: 'scope> {
        inner: &'scope std::thread::Scope<'scope, 'env>,
    }

    impl<'scope, 'env> Scope<'scope, 'env> {
        pub fn spawn<T, F>(self, f: F) -> Task<'scope, T>
        where
            F: FnOnce() -> T + Send + 'scope,
            T: Send + 'scope,
        {
            Task { handle: self.inner.spawn(f) }
        }
    }

    pub struct Task<'scope, T> {
        handle: std::thread::ScopedJoinHandle<'scope, T>,
    }

    impl<T> Task<'_, T> {
        /// Waits for the child; a child's panic is re-raised here.
        pub fn join(self) -> T {
            match self.handle.join() {
                Ok(v) => v,
                Err(p) => std::panic::resume_unwind(p),
            }
        }
    }
}

pub mod sync {
    use crate::marker::Pure;
    use crate::task::Tasks;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::{Arc, Mutex, MutexGuard};

    pub use std::sync::mpsc::{Receiver, RecvError, SendError, Sender, TryRecvError};

    /// Creating a channel takes `&Tasks` (reference 7.3).
    pub fn channel<T: Send>(_tasks: &Tasks) -> (Sender<T>, Receiver<T>) {
        std::sync::mpsc::channel()
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum LockError {
        Aliased,
        Poisoned,
    }

    impl std::fmt::Display for LockError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(match self {
                LockError::Aliased => "the same cell was passed twice",
                LockError::Poisoned => "a panic poisoned the cell",
            })
        }
    }

    impl std::error::Error for LockError {}

    static NEXT_ID: AtomicU64 = AtomicU64::new(0);

    struct Cell<T> {
        id: u64,
        m: Mutex<T>,
    }

    /// Replaces `Arc<Mutex<T>>`; `clone()` names the same cell.
    pub struct Shared<T> {
        cell: Arc<Cell<T>>,
    }

    impl<T> Clone for Shared<T> {
        fn clone(&self) -> Self {
            Shared { cell: Arc::clone(&self.cell) }
        }
    }

    fn lock<T>(c: &Cell<T>) -> Result<MutexGuard<'_, T>, LockError> {
        c.m.lock().map_err(|_| LockError::Poisoned)
    }

    impl<T: Send + Pure> Shared<T> {
        pub fn new(v: T, _tasks: &Tasks) -> Shared<T> {
            let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
            Shared { cell: Arc::new(Cell { id, m: Mutex::new(v) }) }
        }

        pub fn with<R>(&self, f: impl FnOnce(&mut T) -> R + Pure) -> Result<R, LockError> {
            Ok(f(&mut *lock(&self.cell)?))
        }

        /// Locks both cells in global id order, so it cannot deadlock.
        pub fn with2<U: Send + Pure, R>(
            a: &Shared<T>,
            b: &Shared<U>,
            f: impl FnOnce(&mut T, &mut U) -> R + Pure,
        ) -> Result<R, LockError> {
            let (ia, ib) = (a.cell.id, b.cell.id);
            if ia == ib {
                return Err(LockError::Aliased);
            }
            let (mut ga, mut gb);
            if ia < ib {
                ga = lock(&a.cell)?;
                gb = lock(&b.cell)?;
            } else {
                gb = lock(&b.cell)?;
                ga = lock(&a.cell)?;
            }
            Ok(f(&mut ga, &mut gb))
        }

        pub fn same(a: &Shared<T>, b: &Shared<T>) -> bool {
            a.cell.id == b.cell.id
        }
    }
}
