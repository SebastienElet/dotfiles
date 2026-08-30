use super::*;
use crate::memory::process::deadline::cleanup::SystemGroupController;
use crate::memory::process::deadline::readers::{ReaderPipe, ReaderSpawner, ReaderState};
use rustix::process::{Pid, Signal, kill_process_group, test_kill_process};
use std::io;
use std::os::unix::process::CommandExt;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

struct RecordingSpawner(Arc<Mutex<Option<Pid>>>);

impl CommandSpawner for RecordingSpawner {
    fn spawn(&self, command: &mut Command) -> io::Result<Child> {
        let child = command.spawn()?;
        *self.0.lock().unwrap() = Pid::from_raw(child.id() as i32);
        Ok(child)
    }
}

struct GroupGuard(Arc<Mutex<Option<Pid>>>);

impl Drop for GroupGuard {
    fn drop(&mut self) {
        if let Some(group) = *self.0.lock().unwrap() {
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
fn closes_redirected_descendants_before_returning_a_joined_reader_error() {
    let fixture = tempfile::tempdir().unwrap();
    let state = fixture.path().join("descendant-pid");
    let group = Arc::new(Mutex::new(None));
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
        Deadlines::new(Instant::now() + Duration::from_secs(1)),
        &spawner,
        &JoinErrorReaderSpawner,
        &SystemGroupController,
    )
    .unwrap_err();
    let pid = std::fs::read_to_string(state)
        .unwrap()
        .trim()
        .parse::<i32>()
        .unwrap();
    let pid = Pid::from_raw(pid).unwrap();

    assert_eq!(error.kind(), io::ErrorKind::Other);
    assert!(test_kill_process(pid).is_err());
}
