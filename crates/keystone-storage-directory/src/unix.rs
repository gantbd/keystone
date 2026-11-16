use keystone_storage::{
    AcknowledgementStrength, Checkpoint, CheckpointStore, CommitOutcome, CommitReceipt, Revision,
    StoreRefusal,
};
use std::cell::{Cell, RefCell};
use std::fs::{self, File, Metadata, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const HEAD: &str = "checkpoint.bin";
const LOCK: &str = ".owner.lock";
const PENDING: &str = ".pending";
const MAGIC: &[u8; 8] = b"KSDIRCP\x01";
const HEADER: usize = 28;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LimitsV1 {
    pub max_key_bytes: usize,
    pub max_value_bytes: usize,
}

impl Default for LimitsV1 {
    fn default() -> Self {
        Self {
            max_key_bytes: 1024,
            max_value_bytes: 8 * 1024 * 1024,
        }
    }
}

/// Error classes contain neither paths, keys nor stored content.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DirectoryErrorV1 {
    InvalidLimits,
    AlreadyExists,
    Busy,
    Uninitialized,
    UnsafeResource,
    LostResource,
    Corrupt,
    Capacity,
    Fenced,
    Io(std::io::ErrorKind),
}

type Result<T> = std::result::Result<T, DirectoryErrorV1>;
fn io(error: std::io::Error) -> DirectoryErrorV1 {
    DirectoryErrorV1::Io(error.kind())
}

/// Fixed-cardinality, content-free observations; never execution authority.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SignalsV1 {
    pub loads: u64,
    pub offers: u64,
    pub commits: u64,
    pub refusals: u64,
    pub errors: u64,
    pub indeterminate: u64,
    pub load_time: Duration,
    pub commit_time: Duration,
    pub retained_bytes: usize,
    pub fenced: bool,
    pub saturated: bool,
}

fn increment(value: &mut u64, saturated: &mut bool) {
    match value.checked_add(1) {
        Some(next) => *value = next,
        None => *saturated = true,
    }
}
fn elapsed(value: &mut Duration, delta: Duration, saturated: &mut bool) {
    match value.checked_add(delta) {
        Some(next) => *value = next,
        None => {
            *value = Duration::MAX;
            *saturated = true;
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
struct Stamp {
    dev: u64,
    ino: u64,
    len: u64,
    modified: (i64, i64),
    changed: (i64, i64),
}
impl Stamp {
    fn of(meta: &Metadata) -> Self {
        Self {
            dev: meta.dev(),
            ino: meta.ino(),
            len: meta.len(),
            modified: (meta.mtime(), meta.mtime_nsec()),
            changed: (meta.ctime(), meta.ctime_nsec()),
        }
    }
    fn same_object(self, other: Self) -> bool {
        self.dev == other.dev && self.ino == other.ino
    }
}

pub struct DirectoryCheckpointStore {
    path: PathBuf,
    directory: File,
    lock: File,
    root: Stamp,
    lock_stamp: Stamp,
    head: Option<Stamp>,
    key: Option<String>,
    checkpoint: Option<Checkpoint>,
    limits: LimitsV1,
    fenced: Cell<bool>,
    signals: RefCell<SignalsV1>,
    #[cfg(test)]
    fault: Option<Fault>,
}

impl std::fmt::Debug for DirectoryCheckpointStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DirectoryCheckpointStore")
            .field("signals", &self.signals())
            .finish_non_exhaustive()
    }
}

impl DirectoryCheckpointStore {
    pub fn create_new(path: impl AsRef<Path>, limits: LimitsV1) -> Result<Self> {
        validate_limits(limits)?;
        let path = path.as_ref();
        fs::DirBuilder::new()
            .mode(0o700)
            .create(path)
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::AlreadyExists {
                    DirectoryErrorV1::AlreadyExists
                } else {
                    io(e)
                }
            })?;
        let directory = open_directory(path)?;
        let lock = open_private(&path.join(LOCK), true)?;
        lock.try_lock().map_err(|_| DirectoryErrorV1::Busy)?;
        lock.sync_all().map_err(io)?;
        directory.sync_all().map_err(io)?;
        let absolute = fs::canonicalize(path).map_err(io)?;
        // The parent must already exist; persist the newly created leaf entry.
        open_directory(absolute.parent().ok_or(DirectoryErrorV1::UnsafeResource)?)?
            .sync_all()
            .map_err(io)?;
        Self::assemble(absolute, directory, lock, limits)
    }

    pub fn open_existing(path: impl AsRef<Path>, limits: LimitsV1) -> Result<Self> {
        validate_limits(limits)?;
        // Remove trailing separators/dot components before O_NOFOLLOW so a leaf
        // symlink cannot be disguised as `alias/` or `alias/.`.
        let leaf: PathBuf = path.as_ref().components().collect();
        let path = leaf.as_path();
        let directory = open_directory(path)?;
        let lock = open_private(&path.join(LOCK), false)?;
        lock.try_lock().map_err(|_| DirectoryErrorV1::Busy)?;
        let mut store =
            Self::assemble(fs::canonicalize(path).map_err(io)?, directory, lock, limits)?;
        store.check_directory_entries()?;
        let head = store.path.join(HEAD);
        if !head.try_exists().map_err(io)? {
            return Err(DirectoryErrorV1::Uninitialized);
        }
        let mut file = open_private(&head, false)?;
        let meta = file.metadata().map_err(io)?;
        let maximum = record_limit(limits)?;
        if meta.len() > maximum as u64 {
            return Err(DirectoryErrorV1::Capacity);
        }
        let mut record = Vec::new();
        record
            .try_reserve_exact(meta.len() as usize)
            .map_err(|_| DirectoryErrorV1::Capacity)?;
        (&mut file)
            .take(maximum as u64 + 1)
            .read_to_end(&mut record)
            .map_err(io)?;
        if record.len() > maximum {
            return Err(DirectoryErrorV1::Capacity);
        }
        let (key, checkpoint) = decode(&record, limits)?;
        if Stamp::of(&file.metadata().map_err(io)?) != Stamp::of(&meta) {
            return Err(DirectoryErrorV1::LostResource);
        }
        synchronize_file(&file)?;
        // Only this adapter's validated orphan stage may be discarded.
        match fs::symlink_metadata(store.path.join(PENDING)) {
            Ok(meta) => {
                validate_private(&meta)?;
                if meta.len() > maximum as u64 {
                    return Err(DirectoryErrorV1::Capacity);
                }
                fs::remove_file(store.path.join(PENDING)).map_err(io)?;
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(io(e)),
        }
        store.directory.sync_all().map_err(io)?;
        store.head = Some(Stamp::of(&file.metadata().map_err(io)?));
        store.signals.get_mut().retained_bytes = checkpoint.bytes.len();
        store.key = Some(key);
        store.checkpoint = Some(checkpoint);
        store.verify()?;
        Ok(store)
    }

    fn assemble(path: PathBuf, directory: File, lock: File, limits: LimitsV1) -> Result<Self> {
        let root_meta = directory.metadata().map_err(io)?;
        validate_root(&root_meta)?;
        let lock_meta = lock.metadata().map_err(io)?;
        validate_private(&lock_meta)?;
        let store = Self {
            path,
            root: Stamp::of(&root_meta),
            lock_stamp: Stamp::of(&lock_meta),
            directory,
            lock,
            limits,
            head: None,
            key: None,
            checkpoint: None,
            fenced: Cell::new(false),
            signals: RefCell::new(SignalsV1::default()),
            #[cfg(test)]
            fault: None,
        };
        store.verify_identity()?;
        Ok(store)
    }

    pub fn signals(&self) -> SignalsV1 {
        let mut signals = *self.signals.borrow();
        signals.fenced = self.fenced.get();
        signals
    }

    fn verify_identity(&self) -> Result<()> {
        let root = fs::symlink_metadata(&self.path).map_err(io)?;
        validate_root(&root)?;
        let lock = fs::symlink_metadata(self.path.join(LOCK)).map_err(io)?;
        validate_private(&lock)?;
        if !self.root.same_object(Stamp::of(&root))
            || !self.lock_stamp.same_object(Stamp::of(&lock))
            || !self
                .lock_stamp
                .same_object(Stamp::of(&self.lock.metadata().map_err(io)?))
        {
            return Err(DirectoryErrorV1::LostResource);
        }
        Ok(())
    }

    fn check_directory_entries(&self) -> Result<()> {
        for (count, entry) in fs::read_dir(&self.path).map_err(io)?.enumerate() {
            if count >= 3 {
                return Err(DirectoryErrorV1::UnsafeResource);
            }
            let name = entry.map_err(io)?.file_name();
            if name != HEAD && name != LOCK && name != PENDING {
                return Err(DirectoryErrorV1::UnsafeResource);
            }
        }
        Ok(())
    }

    fn verify(&self) -> Result<()> {
        if self.fenced.get() {
            return Err(DirectoryErrorV1::Fenced);
        }
        let result = (|| {
            self.verify_identity()?;
            self.check_directory_entries()?;
            match fs::symlink_metadata(self.path.join(HEAD)) {
                Ok(meta) => {
                    validate_private(&meta)?;
                    if self.head != Some(Stamp::of(&meta)) {
                        return Err(DirectoryErrorV1::LostResource);
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound && self.head.is_none() => {}
                Err(_) => return Err(DirectoryErrorV1::LostResource),
            }
            Ok(())
        })();
        if result.is_err() {
            self.fenced.set(true);
        }
        result
    }

    fn commit(
        &mut self,
        key: &str,
        expected: Option<Revision>,
        bytes: &[u8],
    ) -> Result<CommitOutcome> {
        self.verify()?;
        let refusal = if key.is_empty() || self.key.as_deref().is_some_and(|bound| bound != key) {
            Some(StoreRefusal::InvalidKey)
        } else if key.len() > self.limits.max_key_bytes {
            Some(StoreRefusal::KeyLimit)
        } else if bytes.len() > self.limits.max_value_bytes {
            Some(StoreRefusal::ValueLimit)
        } else {
            None
        };
        if let Some(refusal) = refusal {
            return Ok(CommitOutcome::Refused(refusal));
        }
        let actual = self.checkpoint.as_ref().map(|cp| cp.revision);
        if expected != actual {
            return Ok(CommitOutcome::Refused(StoreRefusal::RevisionConflict {
                expected,
                actual,
            }));
        }
        let revision = match actual {
            None => Revision(0),
            Some(Revision(value)) => match value.checked_add(1) {
                Some(next) => Revision(next),
                None => return Ok(CommitOutcome::Refused(StoreRefusal::RevisionExhausted)),
            },
        };
        // Allocate the future retained state before any filesystem effect.
        let retained_key = bounded_string(key)?;
        let retained_bytes = bounded_vec(bytes)?;
        let record = encode(key, revision, bytes)?;
        let stage = self.path.join(PENDING);
        let mut file = open_private(&stage, true)?;
        file.write_all(&record).map_err(io)?;
        synchronize_file(&file)?;
        #[cfg(test)]
        if self.fault == Some(Fault::BeforeRename) {
            return Err(DirectoryErrorV1::Io(std::io::ErrorKind::Interrupted));
        }
        self.verify()?;
        // From the publication attempt onward no error means "not committed".
        if fs::rename(&stage, self.path.join(HEAD)).is_err() {
            return Ok(CommitOutcome::Indeterminate);
        }
        #[cfg(test)]
        if self.fault == Some(Fault::AfterRename) {
            return Ok(CommitOutcome::Indeterminate);
        }
        if self.directory.sync_all().is_err() {
            return Ok(CommitOutcome::Indeterminate);
        }
        #[cfg(test)]
        if self.fault == Some(Fault::AfterSync) {
            return Ok(CommitOutcome::Indeterminate);
        }
        let meta = match file.metadata() {
            Ok(meta) => meta,
            Err(_) => return Ok(CommitOutcome::Indeterminate),
        };
        self.head = Some(Stamp::of(&meta));
        self.key = Some(retained_key);
        self.checkpoint = Some(Checkpoint {
            revision,
            bytes: retained_bytes,
            strength: AcknowledgementStrength::Durable,
        });
        self.signals.get_mut().retained_bytes = bytes.len();
        Ok(CommitOutcome::Committed(CommitReceipt {
            revision,
            mechanism: "local-directory-v1",
            strength: AcknowledgementStrength::Durable,
        }))
    }
}

impl CheckpointStore for DirectoryCheckpointStore {
    type Error = DirectoryErrorV1;
    fn load(&self, key: &str) -> Result<Option<Checkpoint>> {
        let started = Instant::now();
        let result = (|| {
            self.verify()?;
            if key.is_empty()
                || key.len() > self.limits.max_key_bytes
                || self.key.as_deref().is_some_and(|bound| bound != key)
            {
                return Err(DirectoryErrorV1::UnsafeResource);
            }
            self.checkpoint
                .as_ref()
                .map(|cp| {
                    Ok(Checkpoint {
                        revision: cp.revision,
                        strength: cp.strength,
                        bytes: bounded_vec(&cp.bytes)?,
                    })
                })
                .transpose()
        })();
        let SignalsV1 {
            loads,
            errors,
            load_time,
            saturated,
            ..
        } = &mut *self.signals.borrow_mut();
        increment(loads, saturated);
        elapsed(load_time, started.elapsed(), saturated);
        if result.is_err() {
            increment(errors, saturated);
        }
        result
    }
    fn compare_exchange(
        &mut self,
        key: &str,
        expected: Option<Revision>,
        bytes: &[u8],
    ) -> Result<CommitOutcome> {
        let started = Instant::now();
        let result = self.commit(key, expected, bytes);
        let SignalsV1 {
            offers,
            commits,
            refusals,
            errors,
            indeterminate,
            commit_time,
            saturated,
            ..
        } = self.signals.get_mut();
        increment(offers, saturated);
        elapsed(commit_time, started.elapsed(), saturated);
        match &result {
            Ok(CommitOutcome::Committed(_)) => increment(commits, saturated),
            Ok(CommitOutcome::Refused(_)) => increment(refusals, saturated),
            Ok(CommitOutcome::Indeterminate) => {
                increment(indeterminate, saturated);
                self.fenced.set(true);
            }
            Err(_) => {
                increment(errors, saturated);
                self.fenced.set(true);
            }
        }
        result
    }
}

fn validate_limits(limits: LimitsV1) -> Result<()> {
    if limits.max_key_bytes == 0
        || limits.max_value_bytes == 0
        || u32::try_from(limits.max_key_bytes).is_err()
    {
        return Err(DirectoryErrorV1::InvalidLimits);
    }
    record_limit(limits)?;
    Ok(())
}
fn record_limit(limits: LimitsV1) -> Result<usize> {
    HEADER
        .checked_add(limits.max_key_bytes)
        .and_then(|n| n.checked_add(limits.max_value_bytes))
        .filter(|n| *n < isize::MAX as usize)
        .ok_or(DirectoryErrorV1::InvalidLimits)
}
fn validate_root(meta: &Metadata) -> Result<()> {
    if !meta.is_dir()
        || meta.uid() != rustix::process::geteuid().as_raw()
        || meta.mode() & 0o7777 != 0o700
    {
        return Err(DirectoryErrorV1::UnsafeResource);
    }
    Ok(())
}
fn validate_private(meta: &Metadata) -> Result<()> {
    if !meta.is_file()
        || meta.nlink() != 1
        || meta.uid() != rustix::process::geteuid().as_raw()
        || meta.mode() & 0o7777 != 0o600
    {
        return Err(DirectoryErrorV1::UnsafeResource);
    }
    Ok(())
}
fn open_directory(path: &Path) -> Result<File> {
    OpenOptions::new()
        .read(true)
        .custom_flags((rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::DIRECTORY).bits() as i32)
        .open(path)
        .map_err(io)
}
fn open_private(path: &Path, create: bool) -> Result<File> {
    let file = OpenOptions::new()
        .read(true)
        .write(create)
        .create_new(create)
        .mode(0o600)
        .custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32)
        .open(path)
        .map_err(io)?;
    validate_private(&file.metadata().map_err(io)?)?;
    Ok(file)
}
fn synchronize_file(file: &File) -> Result<()> {
    file.sync_all().map_err(io)?;
    #[cfg(target_vendor = "apple")]
    rustix::fs::fcntl_fullfsync(file).map_err(|error| io(error.into()))?;
    Ok(())
}
fn bounded_vec(bytes: &[u8]) -> Result<Vec<u8>> {
    let mut value = Vec::new();
    value
        .try_reserve_exact(bytes.len())
        .map_err(|_| DirectoryErrorV1::Capacity)?;
    value.extend_from_slice(bytes);
    Ok(value)
}
fn bounded_string(text: &str) -> Result<String> {
    String::from_utf8(bounded_vec(text.as_bytes())?).map_err(|_| DirectoryErrorV1::Corrupt)
}
fn encode(key: &str, revision: Revision, bytes: &[u8]) -> Result<Vec<u8>> {
    let size = HEADER
        .checked_add(key.len())
        .and_then(|v| v.checked_add(bytes.len()))
        .ok_or(DirectoryErrorV1::Capacity)?;
    let mut result = Vec::new();
    result
        .try_reserve_exact(size)
        .map_err(|_| DirectoryErrorV1::Capacity)?;
    result.extend_from_slice(MAGIC);
    result.extend_from_slice(&revision.0.to_be_bytes());
    result.extend_from_slice(&(key.len() as u32).to_be_bytes());
    result.extend_from_slice(&(bytes.len() as u64).to_be_bytes());
    result.extend_from_slice(key.as_bytes());
    result.extend_from_slice(bytes);
    Ok(result)
}
fn decode(record: &[u8], limits: LimitsV1) -> Result<(String, Checkpoint)> {
    if record.len() < HEADER || &record[..8] != MAGIC {
        return Err(DirectoryErrorV1::Corrupt);
    }
    let revision = Revision(u64::from_be_bytes(
        record[8..16]
            .try_into()
            .map_err(|_| DirectoryErrorV1::Corrupt)?,
    ));
    let key_len = u32::from_be_bytes(
        record[16..20]
            .try_into()
            .map_err(|_| DirectoryErrorV1::Corrupt)?,
    ) as usize;
    let value_len = usize::try_from(u64::from_be_bytes(
        record[20..28]
            .try_into()
            .map_err(|_| DirectoryErrorV1::Corrupt)?,
    ))
    .map_err(|_| DirectoryErrorV1::Capacity)?;
    if key_len == 0 || key_len > limits.max_key_bytes || value_len > limits.max_value_bytes {
        return Err(DirectoryErrorV1::Capacity);
    }
    let key_end = HEADER
        .checked_add(key_len)
        .ok_or(DirectoryErrorV1::Corrupt)?;
    if key_end.checked_add(value_len) != Some(record.len()) {
        return Err(DirectoryErrorV1::Corrupt);
    }
    let key =
        std::str::from_utf8(&record[HEADER..key_end]).map_err(|_| DirectoryErrorV1::Corrupt)?;
    Ok((
        bounded_string(key)?,
        Checkpoint {
            revision,
            bytes: bounded_vec(&record[key_end..])?,
            strength: AcknowledgementStrength::Durable,
        },
    ))
}

#[cfg(test)]
#[derive(Clone, Copy, Eq, PartialEq)]
enum Fault {
    BeforeRename,
    AfterRename,
    AfterSync,
}
#[cfg(test)]
mod tests;
