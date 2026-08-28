use super::MemoryError;
use super::path::ManagedPath;
use rustix::fs::FlockOperation;
use std::fs::File;
use std::time::{Duration, Instant};

const LOCK_TIMEOUT: Duration = Duration::from_secs(2);
const LOCK_POLL_INTERVAL: Duration = Duration::from_millis(10);

#[derive(Debug)]
pub(crate) struct GlobalLock {
    _file: File,
}

impl GlobalLock {
    pub(crate) fn acquire(path: &ManagedPath) -> Result<Self, MemoryError> {
        let file = path
            .open_existing_read_write()
            .map_err(|_| lock_unavailable())?;
        path.repair_private_file_mode(&file)
            .map_err(|_| lock_unavailable())?;
        let started = Instant::now();
        loop {
            match rustix::fs::flock(&file, FlockOperation::NonBlockingLockExclusive) {
                Ok(()) => return Ok(Self { _file: file }),
                Err(error)
                    if error == rustix::io::Errno::AGAIN
                        || error == rustix::io::Errno::WOULDBLOCK =>
                {
                    let elapsed = started.elapsed();
                    if elapsed >= LOCK_TIMEOUT {
                        return Err(MemoryError::new("store_lock_timeout", "store"));
                    }
                    std::thread::sleep(LOCK_POLL_INTERVAL.min(LOCK_TIMEOUT - elapsed));
                }
                Err(_) => return Err(lock_unavailable()),
            }
        }
    }
}

const fn lock_unavailable() -> MemoryError {
    MemoryError::new("store_lock_unavailable", "store")
}
