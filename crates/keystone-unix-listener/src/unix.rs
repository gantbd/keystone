use std::{
    fs, io,
    os::unix::{
        fs::{FileTypeExt, MetadataExt, PermissionsExt},
        net::{SocketAddr, UnixListener, UnixStream},
    },
    path::{Path, PathBuf},
};

/// One listener and the exact filesystem identity it published.
#[derive(Debug)]
pub struct OwnedUnixListener {
    listener: UnixListener,
    path: PathBuf,
    device: u64,
    inode: u64,
}

impl OwnedUnixListener {
    pub fn bind_new(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        if fs::symlink_metadata(&path).is_ok() {
            return Err(io::Error::new(
                io::ErrorKind::AddrInUse,
                "socket path already exists",
            ));
        }
        let listener = UnixListener::bind(&path)?;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
        let metadata = fs::symlink_metadata(&path)?;
        if !metadata.file_type().is_socket() {
            return Err(io::Error::other("bound path is not a socket"));
        }
        Ok(Self {
            listener,
            path,
            device: metadata.dev(),
            inode: metadata.ino(),
        })
    }

    pub fn accept(&self) -> io::Result<(UnixStream, SocketAddr)> {
        self.listener.accept()
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for OwnedUnixListener {
    fn drop(&mut self) {
        if let Ok(metadata) = fs::symlink_metadata(&self.path)
            && metadata.file_type().is_socket()
            && metadata.dev() == self.device
            && metadata.ino() == self.inode
        {
            let _ = fs::remove_file(&self.path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        os::unix::fs::PermissionsExt,
        sync::atomic::{AtomicU64, Ordering},
    };

    static NEXT: AtomicU64 = AtomicU64::new(0);

    struct Scratch(PathBuf);

    impl Scratch {
        fn new() -> Self {
            let serial = NEXT.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "keystone-unix-listener-{}-{serial}",
                std::process::id(),
            ));
            fs::create_dir(&path).unwrap();
            fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
            Self(path)
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn publishes_private_socket_and_removes_only_its_identity() {
        let scratch = Scratch::new();
        let path = scratch.0.join("service.sock");
        let listener = OwnedUnixListener::bind_new(&path).unwrap();
        let metadata = fs::symlink_metadata(&path).unwrap();
        assert!(metadata.file_type().is_socket());
        assert_eq!(metadata.permissions().mode() & 0o777, 0o600);
        assert_eq!(listener.path(), path);
        drop(listener);
        assert!(!path.exists());

        let listener = OwnedUnixListener::bind_new(&path).unwrap();
        fs::remove_file(&path).unwrap();
        fs::write(&path, b"replacement").unwrap();
        drop(listener);
        assert_eq!(fs::read(&path).unwrap(), b"replacement");
    }

    #[test]
    fn refuses_every_existing_leaf_without_replacing_it() {
        let scratch = Scratch::new();
        let path = scratch.0.join("occupied");
        fs::write(&path, b"retained").unwrap();
        let error = OwnedUnixListener::bind_new(&path).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::AddrInUse);
        assert_eq!(fs::read(&path).unwrap(), b"retained");
    }
}
