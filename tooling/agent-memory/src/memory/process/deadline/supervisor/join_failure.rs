use super::*;
use crate::memory::process::deadline::cleanup::SystemGroupController;
use crate::memory::process::deadline::readers::{ReaderPipe, ReaderSpawner, ReaderState};
use rustix::process::{Pid, Signal, kill_process_group, test_kill_process};
use std::io;
use std::os::unix::process::CommandExt;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

struct RecordingSpawner(Arc<AtomicI32>);

impl CommandSpawner for RecordingSpawner {
    fn spawn(&self, command: &mut Command) -> io::Result<Child> {
        let child = command.spawn()?;
        self.0.store(
            i32::try_from(child.id()).map_err(io::Error::other)?,
            Ordering::Release,
        );
        Ok(child)
    }
}

struct GroupGuard(Arc<AtomicI32>);

impl Drop for GroupGuard {
    fn drop(&mut self) {
        let group = Pid::from_raw(self.0.load(Ordering::Acquire));
        if let Some(group) = group {
            let _ = kill_process_group(group, Signal::KILL);
        }
    }
}

struct JoinErrorReaderSpawner;

impl ReaderSpawner for JoinErrorReaderSpawner {
    fn spawn(
        &self,
        _reader: ReaderPipe,
        _state: Arc<ReaderState>,
        _cancelled: Arc<std::sync::atomic::AtomicBool>,
    ) -> io::Result<JoinHandle<io::Result<Vec<u8>>>> {
        std::thread::Builder::new().spawn(|| Err(io::Error::other("reader_join_error")))
    }
}

#[test]
fn closes_redirected_descendants_before_returning_a_joined_reader_error()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let state = fixture.path().join("descendant-pid");
    let group = Arc::new(AtomicI32::new(0));
    let _guard = GroupGuard(Arc::clone(&group));
    let spawner = RecordingSpawner(group);
    let mut command = Command::new("sh");
    command
        .args([
            "-c",
            "sleep 5 </dev/null >/dev/null 2>&1 & printf '%s' \"$!\" > \"$1\"",
            "sh",
        ])
        .arg(&state)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0);

    let error = run_command_with_readers(
        &mut command,
        ProcessBudget::new(Instant::now() + Duration::from_secs(1)),
        &spawner,
        &JoinErrorReaderSpawner,
        &SystemGroupController,
    )
    .err()
    .ok_or("expected operation failure")?;
    let pid = std::fs::read_to_string(state)?.trim().parse::<i32>()?;
    let pid = Pid::from_raw(pid).ok_or("missing fixture value")?;

    assert_eq!(error.kind(), io::ErrorKind::Other);
    assert!(test_kill_process(pid).is_err());
    Ok(())
}
