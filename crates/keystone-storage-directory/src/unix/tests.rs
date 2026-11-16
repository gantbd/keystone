use super::*;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::sync::{
    Mutex, MutexGuard,
    atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);
// A concurrently running test can fork between opening a store and execing its
// helper. The child briefly inherits every process file descriptor, including
// another test's owner lock, even though close-on-exec then closes it. Serialize
// fixtures so immediate-release assertions cannot race that fork/exec window.
static FIXTURES: Mutex<()> = Mutex::new(());

struct Fixture {
    path: PathBuf,
    _serial: MutexGuard<'static, ()>,
}
impl Fixture {
    fn new() -> Self {
        let serial = FIXTURES.lock().unwrap_or_else(|poison| poison.into_inner());
        let path = std::env::temp_dir().join(format!(
            "keystone-directory-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
        Self {
            path,
            _serial: serial,
        }
    }
    fn path(&self) -> PathBuf {
        self.path.join("store")
    }
    fn create(&self) -> DirectoryCheckpointStore {
        DirectoryCheckpointStore::create_new(self.path(), LimitsV1::default()).unwrap()
    }
    fn open(&self) -> DirectoryCheckpointStore {
        DirectoryCheckpointStore::open_existing(self.path(), LimitsV1::default()).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.path).unwrap();
    }
}

fn committed(outcome: CommitOutcome, expected: u64) {
    assert!(matches!(outcome, CommitOutcome::Committed(CommitReceipt {
        revision: Revision(actual), strength: AcknowledgementStrength::Durable, ..
    }) if actual == expected));
}

#[test]
fn explicit_creation_cas_and_restart_preserve_exact_bytes() {
    let fixture = Fixture::new();
    assert!(DirectoryCheckpointStore::open_existing(fixture.path(), LimitsV1::default()).is_err());
    let mut store = fixture.create();
    assert_eq!(store.load("key").unwrap(), None);
    assert_eq!(
        DirectoryCheckpointStore::create_new(fixture.path(), LimitsV1::default()).unwrap_err(),
        DirectoryErrorV1::AlreadyExists
    );
    committed(
        store.compare_exchange("key", None, &[0, 255, 42]).unwrap(),
        0,
    );
    assert!(matches!(
        store.compare_exchange("key", None, b"wrong"),
        Ok(CommitOutcome::Refused(
            StoreRefusal::RevisionConflict { .. }
        ))
    ));
    committed(
        store
            .compare_exchange("key", Some(Revision(0)), b"winner")
            .unwrap(),
        1,
    );
    let signals = store.signals();
    assert_eq!(
        (
            signals.offers,
            signals.commits,
            signals.refusals,
            signals.retained_bytes
        ),
        (3, 2, 1, 6)
    );
    assert!(!format!("{store:?}").contains("winner"));
    drop(store);
    let reopened = fixture.open();
    let saved = reopened.load("key").unwrap().unwrap();
    assert_eq!(saved.revision, Revision(1));
    assert_eq!(saved.bytes, b"winner");
    assert_eq!(saved.strength, AcknowledgementStrength::Durable);
}

#[test]
fn initialization_failure_is_not_silently_reset() {
    let fixture = Fixture::new();
    drop(fixture.create());
    assert_eq!(
        DirectoryCheckpointStore::open_existing(fixture.path(), LimitsV1::default()).unwrap_err(),
        DirectoryErrorV1::Uninitialized
    );
    assert_eq!(
        DirectoryCheckpointStore::create_new(fixture.path(), LimitsV1::default()).unwrap_err(),
        DirectoryErrorV1::AlreadyExists
    );
}

#[test]
fn refusal_is_identity_and_key_is_not_a_path() {
    let fixture = Fixture::new();
    let mut store = DirectoryCheckpointStore::create_new(
        fixture.path(),
        LimitsV1 {
            max_key_bytes: 8,
            max_value_bytes: 8,
        },
    )
    .unwrap();
    assert_eq!(
        store.compare_exchange("", None, b"x").unwrap(),
        CommitOutcome::Refused(StoreRefusal::InvalidKey)
    );
    assert_eq!(
        store.compare_exchange("123456789", None, b"x").unwrap(),
        CommitOutcome::Refused(StoreRefusal::KeyLimit)
    );
    assert_eq!(
        store.compare_exchange("a", None, b"123456789").unwrap(),
        CommitOutcome::Refused(StoreRefusal::ValueLimit)
    );
    committed(store.compare_exchange("../x", None, b"value").unwrap(), 0);
    assert!(!fixture.path.join("x").exists());
    assert_eq!(
        store
            .compare_exchange("other", Some(Revision(0)), b"x")
            .unwrap(),
        CommitOutcome::Refused(StoreRefusal::InvalidKey)
    );
    assert_eq!(store.load("../x").unwrap().unwrap().bytes, b"value");
}

#[test]
fn failures_on_both_sides_of_publication_fence_and_recover_the_whole_winner() {
    for fault in [Fault::BeforeRename, Fault::AfterRename, Fault::AfterSync] {
        let fixture = Fixture::new();
        let mut store = fixture.create();
        committed(store.compare_exchange("key", None, b"old").unwrap(), 0);
        store.fault = Some(fault);
        let result = store.compare_exchange("key", Some(Revision(0)), b"new");
        if fault == Fault::BeforeRename {
            assert!(result.is_err());
        } else {
            assert_eq!(result.unwrap(), CommitOutcome::Indeterminate);
        }
        assert_eq!(store.load("key"), Err(DirectoryErrorV1::Fenced));
        assert!(store.signals().fenced);
        drop(store);
        let reopened = fixture.open();
        let saved = reopened.load("key").unwrap().unwrap();
        if fault == Fault::BeforeRename {
            assert_eq!(
                (saved.revision, saved.bytes.as_slice()),
                (Revision(0), b"old".as_slice())
            );
        } else {
            assert_eq!(
                (saved.revision, saved.bytes.as_slice()),
                (Revision(1), b"new".as_slice())
            );
        }
        assert!(!fixture.path().join(PENDING).exists());
    }
}

#[test]
fn aliases_permissions_external_changes_and_missing_head_are_refused() {
    for change in 0..5 {
        let fixture = Fixture::new();
        let mut store = fixture.create();
        committed(store.compare_exchange("key", None, b"old").unwrap(), 0);
        let head = fixture.path().join(HEAD);
        match change {
            0 => fs::remove_file(&head).unwrap(),
            1 => fs::hard_link(&head, fixture.path.join("alias")).unwrap(),
            2 => fs::set_permissions(&head, fs::Permissions::from_mode(0o644)).unwrap(),
            3 => {
                fs::remove_file(&head).unwrap();
                symlink(".owner.lock", &head).unwrap();
            }
            4 => fs::write(&head, b"external replacement").unwrap(),
            _ => unreachable!(),
        }
        assert!(store.load("key").is_err());
        assert_eq!(store.load("key"), Err(DirectoryErrorV1::Fenced));
    }
    let fixture = Fixture::new();
    let mut store = fixture.create();
    committed(store.compare_exchange("key", None, b"old").unwrap(), 0);
    fs::rename(fixture.path(), fixture.path.join("moved")).unwrap();
    assert!(store.load("key").is_err());
}

#[test]
fn recovery_rejects_unknown_files_over_budget_corruption_and_symlink_root() {
    let fixture = Fixture::new();
    let mut store = fixture.create();
    committed(store.compare_exchange("key", None, b"value").unwrap(), 0);
    drop(store);
    symlink(fixture.path(), fixture.path.join("alias")).unwrap();
    assert!(
        DirectoryCheckpointStore::open_existing(
            fixture.path.join("alias").join("."),
            LimitsV1::default()
        )
        .is_err()
    );
    assert!(
        DirectoryCheckpointStore::open_existing(
            format!("{}/", fixture.path.join("alias").display()),
            LimitsV1::default()
        )
        .is_err()
    );
    assert!(
        DirectoryCheckpointStore::open_existing(fixture.path.join("alias"), LimitsV1::default())
            .is_err()
    );
    assert_eq!(
        DirectoryCheckpointStore::open_existing(
            fixture.path(),
            LimitsV1 {
                max_key_bytes: 3,
                max_value_bytes: 1
            }
        )
        .unwrap_err(),
        DirectoryErrorV1::Capacity
    );
    fs::write(fixture.path().join("surprise"), b"x").unwrap();
    assert_eq!(
        DirectoryCheckpointStore::open_existing(fixture.path(), LimitsV1::default()).unwrap_err(),
        DirectoryErrorV1::UnsafeResource
    );
    fs::remove_file(fixture.path().join("surprise")).unwrap();
    fs::write(fixture.path().join(HEAD), b"corrupt").unwrap();
    assert_eq!(
        DirectoryCheckpointStore::open_existing(fixture.path(), LimitsV1::default()).unwrap_err(),
        DirectoryErrorV1::Corrupt
    );
}

#[test]
fn codec_is_bounded_and_rejects_every_truncation_and_trailing_data() {
    let limits = LimitsV1 {
        max_key_bytes: 3,
        max_value_bytes: 3,
    };
    let record = encode("key", Revision(4), &[0, 255, 1]).unwrap();
    assert_eq!(record.len(), HEADER + 6);
    for size in 0..record.len() {
        assert!(decode(&record[..size], limits).is_err());
    }
    let (key, cp) = decode(&record, limits).unwrap();
    assert_eq!(
        (key.as_str(), cp.revision, cp.bytes.as_slice()),
        ("key", Revision(4), [0, 255, 1].as_slice())
    );
    let mut trailing = record.clone();
    trailing.push(0);
    assert!(decode(&trailing, limits).is_err());
    let mut version = record.clone();
    version[7] = 2;
    assert!(decode(&version, limits).is_err());
    let mut huge = record;
    huge[20..28].copy_from_slice(&u64::MAX.to_be_bytes());
    assert!(decode(&huge, limits).is_err());
    assert_eq!(
        validate_limits(LimitsV1 {
            max_key_bytes: 1,
            max_value_bytes: usize::MAX
        }),
        Err(DirectoryErrorV1::InvalidLimits)
    );
}

#[test]
fn counters_saturate_without_wrap() {
    let mut count = u64::MAX;
    let mut saturated = false;
    increment(&mut count, &mut saturated);
    assert_eq!(count, u64::MAX);
    assert!(saturated);
    let mut duration = Duration::MAX;
    saturated = false;
    elapsed(&mut duration, Duration::from_nanos(1), &mut saturated);
    assert_eq!(duration, Duration::MAX);
    assert!(saturated);
}

// Also executed as a separate process. exit() deliberately skips Rust Drop.
#[test]
fn process_helper() {
    let Some(path) = std::env::var_os("KEYSTONE_TEST_STORE") else {
        return;
    };
    let mode = std::env::var("KEYSTONE_TEST_MODE").unwrap();
    if mode == "busy" {
        assert_eq!(
            DirectoryCheckpointStore::open_existing(path, LimitsV1::default()).unwrap_err(),
            DirectoryErrorV1::Busy
        );
        std::process::exit(0);
    }
    let mut store = DirectoryCheckpointStore::open_existing(path, LimitsV1::default()).unwrap();
    if mode == "stage" {
        store.fault = Some(Fault::BeforeRename);
    }
    if mode == "publish" {
        store.fault = Some(Fault::AfterRename);
    }
    let result = store.compare_exchange("key", Some(Revision(0)), b"child");
    match mode.as_str() {
        "stage" => assert!(result.is_err()),
        "publish" => assert_eq!(result.unwrap(), CommitOutcome::Indeterminate),
        "commit" => committed(result.unwrap(), 1),
        _ => panic!("unknown helper mode"),
    }
    std::process::exit(0);
}

fn child(path: &Path, mode: &str) {
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "unix::tests::process_helper", "--nocapture"])
        .env("KEYSTONE_TEST_STORE", path)
        .env("KEYSTONE_TEST_MODE", mode)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "child failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn process_lock_excludes_another_owner_and_crash_releases_without_drop() {
    for mode in ["stage", "publish", "commit"] {
        let fixture = Fixture::new();
        let mut store = fixture.create();
        committed(store.compare_exchange("key", None, b"parent").unwrap(), 0);
        child(&fixture.path(), "busy");
        drop(store);
        child(&fixture.path(), mode);
        let reopened = fixture.open();
        let saved = reopened.load("key").unwrap().unwrap();
        assert_eq!(
            saved.bytes,
            if mode == "stage" {
                b"parent".as_slice()
            } else {
                b"child".as_slice()
            }
        );
    }
}
