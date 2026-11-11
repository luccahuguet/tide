// Inode identity, names, data and open-handle retention have one owner: Store.
use super::{DirEntry, DirEntryBackend, Kind, Metadata, OpenOptions};
use crate::io::{self, ErrorKind as K, SeekFrom, Write};
use crate::task::Tasks;
use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Mutex};

type InodeId = u64;
const ROOT: InodeId = 0;
struct Inode {
    refs: usize,
    data: Data,
}
enum Data {
    File(Vec<u8>),
    Dir {
        entries: BTreeMap<OsString, InodeId>,
        parent: InodeId,
    },
}
struct Store {
    inodes: BTreeMap<InodeId, Inode>,
    next: InodeId,
}
impl Store {
    fn new() -> Self {
        Self {
            inodes: BTreeMap::from([(
                ROOT,
                Inode {
                    refs: 2, // One for the root's permanent name, one for the base handle.
                    data: Data::Dir {
                        entries: BTreeMap::new(),
                        parent: ROOT,
                    },
                },
            )]),
            next: 1,
        }
    }
    fn entries(&self, id: InodeId) -> io::Result<&BTreeMap<OsString, InodeId>> {
        match &self.inodes[&id].data {
            Data::Dir { entries, .. } => Ok(entries),
            Data::File(_) => Err(K::NotADirectory.into()),
        }
    }
    fn entries_mut(&mut self, id: InodeId) -> &mut BTreeMap<OsString, InodeId> {
        match &mut self.inodes.get_mut(&id).unwrap().data {
            Data::Dir { entries, .. } => entries,
            Data::File(_) => unreachable!(),
        }
    }
    fn allocate(&mut self, parent: InodeId, name: OsString, directory: bool) -> InodeId {
        let id = self.next;
        self.next += 1;
        self.inodes.insert(
            id,
            Inode {
                refs: 1,
                data: if directory {
                    Data::Dir {
                        entries: BTreeMap::new(),
                        parent,
                    }
                } else {
                    Data::File(Vec::new())
                },
            },
        );
        self.entries_mut(parent).insert(name, id);
        id
    }
    fn release(&mut self, id: InodeId) {
        let inode = self.inodes.get_mut(&id).unwrap();
        inode.refs -= 1;
        if inode.refs == 0 {
            let inode = self.inodes.remove(&id).unwrap();
            if let Data::Dir { entries, .. } = inode.data {
                for (_, child) in entries {
                    self.release(child);
                }
            }
        }
    }
    fn unlink(&mut self, parent: InodeId, name: &OsStr) {
        let id = self.entries_mut(parent).remove(name).unwrap();
        // An unlinked open directory has no navigable parent (Linux ENOENT for `..`).
        if let Data::Dir { parent, .. } = &mut self.inodes.get_mut(&id).unwrap().data {
            *parent = id;
        }
        self.release(id);
    }
    fn unlinked(&self, id: InodeId) -> bool {
        id != ROOT && matches!(self.inodes[&id].data, Data::Dir { parent, .. } if parent == id)
    }
    fn resolve(&self, base: InodeId, confined: bool, p: &Path) -> io::Result<InodeId> {
        check_path_len(p)?;
        if p.as_os_str().is_empty() {
            return Err(K::NotFound.into());
        }
        let mut id = base;
        for component in p.components() {
            // Even `file/.` and `file/..` must look through a directory.
            self.entries(id)?;
            match component {
                Component::RootDir => id = ROOT,
                Component::CurDir => {}
                Component::ParentDir => {
                    if confined && id == base {
                        return Err(K::PermissionDenied.into());
                    }
                    let Data::Dir { parent, .. } = self.inodes[&id].data else {
                        unreachable!()
                    };
                    if self.unlinked(id) {
                        return Err(K::NotFound.into());
                    }
                    id = parent;
                }
                Component::Normal(name) => {
                    if name.as_bytes().len() > 255 {
                        return Err(K::InvalidFilename.into());
                    }
                    id = *self
                        .entries(id)?
                        .get(name)
                        .ok_or(io::Error::from(K::NotFound))?;
                }
                Component::Prefix(_) => return Err(K::InvalidInput.into()),
            }
        }
        // `components` discards both trailing slashes and trailing `.`.
        if needs_directory(p) {
            self.entries(id)?;
        }
        Ok(id)
    }
    fn parent(&self, base: InodeId, confined: bool, p: &Path) -> io::Result<(InodeId, OsString)> {
        let (parent, name) = super::split(p)?;
        let parent = self.resolve(base, confined, &parent)?;
        self.entries(parent)?;
        // Resolve the parent before classifying the final name, as Linux mkdir does.
        super::normal_name(p)?;
        check_path_len(Path::new(&name))?;
        if self.unlinked(parent) {
            return Err(K::NotFound.into());
        }
        let name = OsStr::from_bytes(super::trim_final_slashes(name.as_bytes()));
        if name.as_bytes().len() > 255 {
            return Err(K::InvalidFilename.into());
        }
        Ok((parent, name.to_owned()))
    }
    fn metadata(&self, id: InodeId) -> Metadata {
        let (kind, len) = match &self.inodes[&id].data {
            Data::File(bytes) => (Kind::File, bytes.len() as u64),
            Data::Dir { .. } => (Kind::Dir, 0),
        };
        Metadata::new(kind, len)
    }
    fn mkdir_all(&mut self, base: InodeId, confined: bool, p: &Path) -> io::Result<()> {
        let mut prefix = PathBuf::new();
        for component in p.components() {
            prefix.push(component);
            match self
                .resolve(base, confined, &prefix)
                .and_then(|id| self.entries(id).map(|_| id))
            {
                Ok(_) => continue,
                Err(e) if e.kind() == K::NotFound && matches!(component, Component::Normal(_)) => {}
                Err(e) if e.kind() == K::NotADirectory => {
                    let (parent, name) = self.parent(base, confined, p)?;
                    return if self.entries(parent)?.contains_key(&name) {
                        Err(K::AlreadyExists.into())
                    } else {
                        Err(e)
                    };
                }
                Err(e) => return Err(e),
            }
            let (parent, name) = self.parent(base, confined, &prefix)?;
            self.allocate(parent, name, true);
        }
        Ok(())
    }
    fn open(
        &mut self,
        base: InodeId,
        confined: bool,
        p: &Path,
        opts: &OpenOptions,
    ) -> io::Result<InodeId> {
        check_path_len(p)?;
        // Linux rejects creation with a trailing slash after resolving the parent.
        if (opts.create || opts.create_new) && p.as_os_str().as_bytes().ends_with(b"/") {
            if super::normal_name(p).is_ok() {
                self.parent(base, confined, p)?;
            } else {
                self.resolve(base, confined, p)?;
                // A raw final `.`/`..` exists; a slash-only root is still IsADirectory.
                if opts.create_new && p.as_os_str().as_bytes().iter().any(|&b| b != b'/') {
                    return Err(K::AlreadyExists.into());
                }
            }
            return Err(K::IsADirectory.into());
        }
        let id = match self.resolve(base, confined, p) {
            Ok(_) if opts.create_new => return Err(K::AlreadyExists.into()),
            Ok(id) => id,
            Err(e) if e.kind() == K::NotFound && (opts.create || opts.create_new) => {
                if super::normal_name(p).is_err() {
                    return Err(e);
                }
                let (parent, name) = self.parent(base, confined, p)?;
                self.allocate(parent, name, false)
            }
            Err(e) => return Err(e),
        };
        match &mut self.inodes.get_mut(&id).unwrap().data {
            Data::Dir { .. } if opts.write || opts.append => return Err(K::IsADirectory.into()),
            Data::File(bytes) if opts.truncate && !opts.create_new => bytes.clear(),
            _ => {}
        }
        Ok(id)
    }
    fn write(&mut self, base: InodeId, confined: bool, p: &Path, data: &[u8]) -> io::Result<()> {
        let id = self.open(
            base,
            confined,
            p,
            OpenOptions::new().write(true).create(true).truncate(true),
        )?;
        let Data::File(bytes) = &mut self.inodes.get_mut(&id).unwrap().data else {
            unreachable!()
        };
        bytes.extend_from_slice(data);
        Ok(())
    }
    fn empty(&mut self, id: InodeId) {
        let names: Vec<_> = self.entries(id).unwrap().keys().cloned().collect();
        for name in names {
            let child = self.entries(id).unwrap()[&name];
            if self.metadata(child).is_dir() {
                self.empty(child);
            }
            self.unlink(id, &name);
        }
    }
}

// Linux PATH_MAX includes the terminating NUL; check each syscall-shaped argument.
fn check_path_len(p: &Path) -> io::Result<()> {
    if p.as_os_str().as_bytes().len() >= 4096 {
        return Err(K::InvalidFilename.into());
    }
    Ok(())
}

fn needs_directory(p: &Path) -> bool {
    let bytes = p.as_os_str().as_bytes();
    bytes.ends_with(b"/") || bytes.ends_with(b"/.")
}

// Counts external handles in the same table as directory names; drops never do I/O.
struct Handle {
    store: Arc<Mutex<Store>>,
    id: InodeId,
}
impl Handle {
    fn retain(store: &mut Store, source: &Self, id: InodeId) -> Self {
        store.inodes.get_mut(&id).unwrap().refs += 1;
        Self {
            store: source.store.clone(),
            id,
        }
    }
}
impl Clone for Handle {
    fn clone(&self) -> Self {
        Self::retain(&mut self.store.lock().unwrap(), self, self.id)
    }
}
impl Drop for Handle {
    fn drop(&mut self) {
        self.store.lock().unwrap().release(self.id);
    }
}
impl std::fmt::Debug for Handle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Inode").field(&self.id).finish()
    }
}
#[derive(Clone)]
pub(super) struct Fs {
    base: Handle,
    confined: bool,
    pub(super) tasks: Tasks,
}
impl std::fmt::Debug for Fs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimFs").field("base", &self.base).finish()
    }
}
impl Fs {
    pub(super) fn root(tasks: &Tasks) -> Self {
        Self {
            base: Handle {
                store: Arc::new(Mutex::new(Store::new())),
                id: ROOT,
            },
            confined: false,
            tasks: tasks.clone(),
        }
    }
    pub(super) fn is_confined(&self) -> bool {
        self.confined
    }
    pub(super) fn dir(&self, p: &Path) -> io::Result<Self> {
        let mut store = self.base.store.lock().unwrap();
        let id = store.resolve(self.base.id, self.confined, p)?;
        store.entries(id)?;
        Ok(Self {
            base: Handle::retain(&mut store, &self.base, id),
            confined: true,
            tasks: self.tasks.clone(),
        })
    }
    pub(super) fn open(&self, p: &Path, opts: &OpenOptions) -> io::Result<File> {
        let mut store = self.base.store.lock().unwrap();
        let id = store.open(self.base.id, self.confined, p, opts)?;
        Ok(File {
            inode: Handle::retain(&mut store, &self.base, id),
            position: Mutex::new(0),
            append: opts.append,
            tasks: self.tasks.clone(),
            path: p.to_owned(),
        })
    }
    pub(super) fn read(&self, p: &Path) -> io::Result<Vec<u8>> {
        let store = self.base.store.lock().unwrap();
        let id = store.resolve(self.base.id, self.confined, p)?;
        match &store.inodes[&id].data {
            Data::File(bytes) => Ok(bytes.clone()),
            Data::Dir { .. } => Err(K::IsADirectory.into()),
        }
    }
    pub(super) fn write(&self, p: &Path, data: &[u8]) -> io::Result<()> {
        self.base
            .store
            .lock()
            .unwrap()
            .write(self.base.id, self.confined, p, data)
    }
    pub(super) fn write_synced(&self, p: &Path, data: &[u8]) -> io::Result<()> {
        let mut store = self.base.store.lock().unwrap();
        let (parent, name) = super::split(p)?;
        let parent = store.resolve(self.base.id, self.confined, &parent)?;
        store.entries(parent)?;
        // Real write_synced opens the final target beneath this newly pinned parent.
        store.write(parent, self.confined, Path::new(&name), data)
    }
    pub(super) fn metadata(&self, p: &Path) -> io::Result<Metadata> {
        let store = self.base.store.lock().unwrap();
        Ok(store.metadata(store.resolve(self.base.id, self.confined, p)?))
    }
    pub(super) fn sync_dir(&self, p: &Path) -> io::Result<()> {
        let store = self.base.store.lock().unwrap();
        store.entries(store.resolve(self.base.id, self.confined, p)?)?;
        Ok(())
    }
    pub(super) fn create_dir_all(&self, p: &Path) -> io::Result<()> {
        self.base
            .store
            .lock()
            .unwrap()
            .mkdir_all(self.base.id, self.confined, p)
    }
    pub(super) fn remove(&self, p: &Path, directory: bool) -> io::Result<()> {
        let mut store = self.base.store.lock().unwrap();
        let (parent, name) = store.parent(self.base.id, self.confined, p)?;
        let id = *store
            .entries(parent)?
            .get(&name)
            .ok_or(io::Error::from(K::NotFound))?;
        let is_dir = store.metadata(id).is_dir();
        if needs_directory(p) && !is_dir {
            return Err(K::NotADirectory.into());
        }
        if directory {
            if !store.entries(id)?.is_empty() {
                return Err(K::DirectoryNotEmpty.into());
            }
        } else if is_dir {
            return Err(K::IsADirectory.into());
        }
        store.unlink(parent, &name);
        Ok(())
    }
    pub(super) fn remove_dir_all(&self, p: &Path) -> io::Result<()> {
        let mut store = self.base.store.lock().unwrap();
        if super::normal_name(p).is_err() && !p.as_os_str().is_empty() {
            let id = store.resolve(self.base.id, self.confined, p)?;
            if store.unlinked(id) {
                return Err(K::NotFound.into());
            }
            store.entries(id)?;
            store.empty(id);
            return Err(K::InvalidInput.into());
        }
        // Real recursive removal strips final slashes before its target syscalls.
        let bare = Path::new(OsStr::from_bytes(super::trim_final_slashes(
            p.as_os_str().as_bytes(),
        )));
        let (parent, name) = store.parent(self.base.id, self.confined, bare)?;
        let id = *store
            .entries(parent)?
            .get(&name)
            .ok_or(io::Error::from(K::NotFound))?;
        let directory = store.metadata(id).is_dir();
        if needs_directory(p) && !directory {
            return Err(K::NotADirectory.into());
        }
        if directory {
            store.empty(id);
        }
        store.unlink(parent, &name);
        Ok(())
    }
    pub(super) fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        let mut store = self.base.store.lock().unwrap();
        let (src, name) = store.parent(self.base.id, self.confined, from)?;
        let (dst, target) = store.parent(self.base.id, self.confined, to)?;
        let id = *store
            .entries(src)?
            .get(&name)
            .ok_or(io::Error::from(K::NotFound))?;
        let directory = store.metadata(id).is_dir();
        if !directory && (needs_directory(from) || needs_directory(to)) {
            return Err(K::NotADirectory.into());
        }
        if directory {
            let mut ancestor = dst;
            loop {
                if ancestor == id {
                    return Err(K::InvalidInput.into());
                }
                let Data::Dir { parent, .. } = store.inodes[&ancestor].data else {
                    unreachable!()
                };
                if parent == ancestor {
                    break;
                }
                ancestor = parent;
            }
        }
        if let Some(&existing) = store.entries(dst)?.get(&target) {
            if existing == id {
                return Ok(());
            }
            let target_dir = store.metadata(existing).is_dir();
            if directory && !target_dir {
                return Err(K::NotADirectory.into());
            }
            if !directory && target_dir {
                return Err(K::IsADirectory.into());
            }
            if target_dir && !store.entries(existing)?.is_empty() {
                return Err(K::DirectoryNotEmpty.into());
            }
            store.unlink(dst, &target);
        }
        store.entries_mut(src).remove(&name);
        store.entries_mut(dst).insert(target, id);
        if let Data::Dir { parent, .. } = &mut store.inodes.get_mut(&id).unwrap().data {
            *parent = dst;
        }
        Ok(())
    }
    pub(super) fn read_dir(&self, p: &Path) -> io::Result<Vec<DirEntry>> {
        let directory = self.dir(p)?;
        let names: Vec<_> = {
            let store = directory.base.store.lock().unwrap();
            if store.unlinked(directory.base.id) {
                return Err(K::NotFound.into());
            }
            store.entries(directory.base.id)?.keys().cloned().collect()
        };
        Ok(names
            .into_iter()
            .map(|name| DirEntry {
                backend: DirEntryBackend::Sim(directory.clone()),
                path: p.join(&name),
                name,
            })
            .collect())
    }
    #[track_caller]
    pub(crate) fn world_file(&self, p: &Path, data: &[u8]) {
        self.tasks.check_simulation();
        let result = (|| {
            super::check_path(p, false)?;
            // Setup rejects above-root parents instead of silently clamping them like root Fs.
            let mut depth = 0usize;
            for part in p.components() {
                match part {
                    Component::Normal(_) => depth += 1,
                    Component::ParentDir => {
                        depth = depth
                            .checked_sub(1)
                            .ok_or(io::Error::from(K::PermissionDenied))?
                    }
                    _ => {}
                }
            }
            let (parent, _) = super::split(p)?;
            self.create_dir_all(&parent)?;
            self.write(p, data)
        })();
        let outcome = match &result {
            Ok(_) => "ok".into(),
            Err(e) => format!("{:?}", e.kind()),
        };
        self.tasks
            .world_file(p, std::panic::Location::caller(), &outcome);
        result.unwrap_or_else(|e| panic!("sim.world.file({p:?}): {e}"));
    }
}

pub(super) struct File {
    inode: Handle,
    position: Mutex<u64>,
    append: bool,
    pub(super) tasks: Tasks,
    pub(super) path: PathBuf,
}
impl File {
    pub(super) fn metadata(&self) -> Metadata {
        self.inode.store.lock().unwrap().metadata(self.inode.id)
    }
    pub(super) fn set_len(&self, n: u64) -> io::Result<()> {
        if n > i64::MAX as u64 {
            return Err(K::InvalidInput.into());
        }
        let mut store = self.inode.store.lock().unwrap();
        let Data::File(bytes) = &mut store.inodes.get_mut(&self.inode.id).unwrap().data else {
            return Err(K::InvalidInput.into());
        };
        resize(
            bytes,
            usize::try_from(n).map_err(|_| io::Error::from(K::InvalidInput))?,
        )
    }
    pub(super) fn cursor<T>(
        &self,
        writing: bool,
        f: impl FnOnce(&mut io::Cursor<&mut Vec<u8>>) -> io::Result<T>,
    ) -> io::Result<T> {
        let mut store = self.inode.store.lock().unwrap();
        let Data::File(bytes) = &mut store.inodes.get_mut(&self.inode.id).unwrap().data else {
            return Err(K::IsADirectory.into());
        };
        let mut position = self.position.lock().unwrap();
        if writing && self.append {
            *position = bytes.len() as u64;
        }
        let mut cursor = io::Cursor::new(bytes);
        cursor.set_position(*position);
        let result = f(&mut cursor);
        *position = cursor.position();
        result
    }
    pub(super) fn write(&self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.is_empty() {
            return Ok(0);
        }
        self.cursor(true, |cursor| {
            let end = cursor
                .position()
                .checked_add(bytes.len() as u64)
                .filter(|&n| n <= i64::MAX as u64)
                .ok_or(io::Error::from(K::InvalidInput))?;
            let end = usize::try_from(end).map_err(|_| io::Error::from(K::InvalidInput))?;
            if end > cursor.get_ref().len() {
                resize(cursor.get_mut(), end)?;
            }
            cursor.write(bytes)
        })
    }
    pub(super) fn seek(&self, pos: SeekFrom) -> io::Result<u64> {
        let metadata = self.metadata();
        let mut position = self.position.lock().unwrap();
        // shortcut: directory end cookies use i64::MAX; model host-specific cookies if needed.
        let end = if metadata.is_dir() {
            i64::MAX as u64
        } else {
            metadata.len()
        };
        let (base, offset) = match pos {
            SeekFrom::Start(n) => (n, 0),
            SeekFrom::Current(n) => (*position, n),
            SeekFrom::End(n) => (end, n),
        };
        let next = base
            .checked_add_signed(offset)
            .filter(|&n| n <= i64::MAX as u64)
            .ok_or(io::Error::from(K::InvalidInput))?;
        *position = next;
        Ok(next)
    }
}
fn resize(bytes: &mut Vec<u8>, n: usize) -> io::Result<()> {
    if n > bytes.len() {
        bytes.try_reserve(n - bytes.len())?;
    }
    bytes.resize(n, 0);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::{Read, Seek};
    use std::panic::{AssertUnwindSafe, catch_unwind};
    use std::time::Duration;

    #[test]
    fn every_io_call_has_one_draw_and_data_accessors_have_none() {
        let sim = crate::Root::sim(23);
        sim.world.file("/d/file", "abc");
        macro_rules! op {
            ($call:expr) => {{
                let before = sim.elapsed();
                let events = sim
                    .trace()
                    .lines()
                    .filter(|l| l.contains("\"kind\":\"fs\""))
                    .count();
                let result = $call;
                let trace = sim.trace();
                assert_eq!(
                    trace
                        .lines()
                        .filter(|l| l.contains("\"kind\":\"fs\""))
                        .count(),
                    events + 1,
                    "{}",
                    stringify!($call)
                );
                assert!(
                    sim.elapsed() >= before && sim.elapsed() <= before + Duration::from_millis(1),
                    "{}",
                    stringify!($call)
                );
                result
            }};
        }
        let fs = op!(sim.fs.dir("d", super::super::Mode::ReadWrite)).unwrap();
        let before = sim.trace();
        let ro = fs.read_only();
        assert_eq!(sim.trace(), before);
        op!(fs.read("file")).unwrap();
        op!(fs.read_to_string("file")).unwrap();
        op!(fs.write("file", "abc")).unwrap();
        op!(fs.write_synced("file", "abc")).unwrap();
        op!(fs.exists("absent")).unwrap();
        op!(fs.metadata("file")).unwrap();
        op!(fs.sync_dir(".")).unwrap();
        op!(fs.create_dir_all("a/b")).unwrap();
        assert_eq!(
            op!(fs.write_synced("a/b/..", "bad")).unwrap_err().kind(),
            K::PermissionDenied
        );
        op!(fs.remove_dir("a/b")).unwrap();
        op!(fs.remove_dir_all("a")).unwrap();
        op!(fs.rename("file", "renamed")).unwrap();
        let mut list = op!(fs.read_dir(".")).unwrap();
        let entry = op!(list.next()).unwrap().unwrap();
        let trace = sim.trace();
        entry.path();
        entry.file_name();
        assert_eq!(sim.trace(), trace);
        op!(entry.metadata()).unwrap();
        op!(entry.file_type()).unwrap();
        assert!(op!(list.next()).is_none());
        let trace = sim.trace();
        assert!(list.next().is_none());
        assert_eq!(sim.trace(), trace);
        op!(fs.open("renamed")).unwrap();
        op!(fs.create("other")).unwrap();
        let mut file =
            op!(fs.open_with("renamed", OpenOptions::new().read(true).write(true))).unwrap();
        op!(file.read(&mut [0; 1])).unwrap();
        op!(file.read_exact(&mut [])).unwrap();
        op!((&file).read_to_end(&mut Vec::new())).unwrap();
        op!((&file).read_to_string(&mut String::new())).unwrap();
        op!(file.seek(SeekFrom::Start(0))).unwrap();
        op!(file.rewind()).unwrap();
        op!((&file).stream_position()).unwrap();
        op!(file.seek_relative(1)).unwrap();
        op!(file.write(b"x")).unwrap();
        op!((&file).write_all(b"y")).unwrap();
        op!(file.write_fmt(format_args!("z"))).unwrap();
        op!(file.flush()).unwrap();
        op!(file.set_len(1)).unwrap();
        op!(file.metadata()).unwrap();
        op!(file.sync()).unwrap();
        op!(file.sync_all()).unwrap();
        op!(file.sync_data()).unwrap();
        op!(file.close()).unwrap();
        assert_eq!(
            op!(ro.write("bad", "bad")).unwrap_err().kind(),
            K::PermissionDenied
        );
        assert_eq!(
            op!(fs.read("renamed/.")).unwrap_err().kind(),
            K::NotADirectory
        );
        op!(fs.remove_file("renamed")).unwrap();
    }

    #[test]
    fn setup_errors_and_wrong_identity_do_not_touch_host_or_poison_the_store() {
        let (fs, file, mut listing, entry) = {
            let sim = crate::Root::sim(0);
            sim.world.file("a/b", "data");
            for path in ["", "a", "a/b/c", "../outside", "nul\0name", "/a/.."] {
                assert!(
                    catch_unwind(AssertUnwindSafe(|| sim.world.file(path, "bad"))).is_err(),
                    "{path:?}"
                );
            }
            assert_eq!(sim.elapsed(), Duration::ZERO);
            assert_eq!(sim.fs.read("Cargo.toml").unwrap_err().kind(), K::NotFound);
            assert_eq!(sim.fs.read("a/b").unwrap(), b"data");
            let mut listing = sim.fs.read_dir("a").unwrap();
            let entry = listing.next().unwrap().unwrap();
            let file = sim.fs.open("a/b").unwrap();
            (sim.fs.clone(), file, listing, entry)
        };
        let _different = crate::Root::sim(0);
        assert!(catch_unwind(AssertUnwindSafe(|| fs.read("a/b"))).is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| file.metadata())).is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| listing.next())).is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| entry.metadata())).is_err());
    }

    #[test]
    fn removed_files_live_until_the_last_handle_drops() {
        let sim = crate::Root::sim(0);
        sim.world.file("value", "kept");
        let file = sim.fs.open("value").unwrap();
        let super::super::FileBackend::Sim(inner) = &file.backend else {
            unreachable!()
        };
        let store = inner.inode.store.clone();
        sim.fs.remove_file("value").unwrap();
        assert_eq!(store.lock().unwrap().inodes.len(), 2);
        let mut bytes = Vec::new();
        (&file).read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes, b"kept");
        file.close().unwrap();
        assert_eq!(store.lock().unwrap().inodes.len(), 1);
    }

    #[test]
    fn a_precancelled_call_does_not_consume_a_latency_draw() {
        fn next_latency(cancel: bool) -> Duration {
            let sim = crate::Root::sim(0);
            if cancel {
                sim.tasks
                    .scope::<_, crate::task::TaskError, _>(|scope| {
                        scope.cancel();
                        assert_eq!(sim.fs.read("absent").unwrap_err().kind(), K::Cancelled);
                        assert_eq!(sim.elapsed(), Duration::ZERO);
                        Ok(())
                    })
                    .unwrap();
            }
            sim.fs.write("file", "data").unwrap();
            sim.elapsed()
        }
        assert_eq!(next_latency(true), next_latency(false));
    }
}
