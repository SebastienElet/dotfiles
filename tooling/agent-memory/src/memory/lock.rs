use super::MemoryError;
use super::path::ManagedPath;
use rustix::fs::FlockOperation;
use std::fs::File;
use std::time::{Duration, Instant};

const LOCK_TIMEOUT: Duration = Duration::from_secs(2);
const LOCK_POLL_INTERVAL: Duration = Duration::from_millis(10);

#[derive(Debug)]
pub(crate) struct GlobalLock {
    _directory: File,
    file: File,
}

impl GlobalLock {
    pub(crate) fn acquire(root: &ManagedPath, path: &ManagedPath) -> Result<Self, MemoryError> {
        let started = Instant::now();
        let directory = root.open_root_directory().map_err(|_| lock_unavailable())?;
        acquire_exclusive(&directory, started)?;
        let file = path
            .open_existing_read_write()
            .map_err(|_| lock_unavailable())?;
        path.repair_private_file_mode(&file)
            .map_err(|_| lock_unavailable())?;
        acquire_exclusive(&file, started)?;
        Ok(Self {
            _directory: directory,
            file,
        })
    }

    pub(crate) fn ensure_anchored(&self, path: &ManagedPath) -> Result<(), MemoryError> {
        path.ensure_same_file(&self.file)
            .map_err(|_| lock_unavailable())
    }
}

fn acquire_exclusive(file: &File, started: Instant) -> Result<(), MemoryError> {
    loop {
        match rustix::fs::flock(file, FlockOperation::NonBlockingLockExclusive) {
            Ok(()) => return Ok(()),
            Err(error)
                if error == rustix::io::Errno::AGAIN || error == rustix::io::Errno::WOULDBLOCK =>
            {
                let elapsed = started.elapsed();
                if elapsed >= LOCK_TIMEOUT {
                    return Err(MemoryError::unavailable("store_lock_timeout", "store"));
                }
                std::thread::sleep(LOCK_POLL_INTERVAL.min(LOCK_TIMEOUT - elapsed));
            }
            Err(_) => return Err(lock_unavailable()),
        }
    }
}

const fn lock_unavailable() -> MemoryError {
    MemoryError::unavailable("store_lock_unavailable", "store")
}
