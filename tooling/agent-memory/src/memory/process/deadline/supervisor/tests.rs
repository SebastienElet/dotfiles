use super::*;
use crate::memory::process::deadline::cleanup::SystemGroupController;
use crate::memory::process::deadline::readers::{ReaderPipe, ReaderSpawner, ReaderState};
use rustix::process::{
    Pid, Signal, WaitOptions, kill_process_group, test_kill_process_group, waitpid,
};
use std::io;
use std::os::unix::process::CommandExt;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

struct CountingSpawner(AtomicUsize);

impl CommandSpawner for CountingSpawner {
    fn spawn(&self, _command: &mut Command) -> io::Result<Child> {
        self.0.fetch_add(1, Ordering::AcqRel);
        Err(io::Error::other("unexpected_spawn"))
    }
}

struct RecordingSpawner {
    pid: Arc<Mutex<Option<Pid>>>,
}

impl RecordingSpawner {
    fn new() -> Self {
        Self {
            pid: Arc::new(Mutex::new(None)),
        }
    }

    fn pid(&self) -> Pid {
        self.pid.lock().unwrap().unwrap()
    }

    fn group_guard(&self) -> GroupGuard {
        GroupGuard(Arc::clone(&self.pid))
    }
}

impl CommandSpawner for RecordingSpawner {
    fn spawn(&self, command: &mut Command) -> io::Result<Child> {
        let child = command.spawn()?;
        *self.pid.lock().unwrap() = Pid::from_raw(child.id() as i32);
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

struct FailingReaderSpawner {
    fail_on: usize,
    attempts: AtomicUsize,
}

impl ReaderSpawner for FailingReaderSpawner {
    fn spawn(
        &self,
        reader: ReaderPipe,
        state: Arc<ReaderState>,
        cancelled: Arc<std::sync::atomic::AtomicBool>,
    ) -> io::Result<JoinHandle<io::Result<Vec<u8>>>> {
        let attempt = self.attempts.fetch_add(1, Ordering::AcqRel) + 1;
        if attempt == self.fail_on {
            return Err(io::Error::other("reader_spawn_unavailable"));
        }
        SystemReaderSpawner.spawn(reader, state, cancelled)
    }
}

struct FailingGroupController;

impl GroupController for FailingGroupController {
    fn kill_group(&self, _group: Pid) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "kill_group_unavailable",
        ))
    }

    fn group_closed(&self, group: Pid) -> io::Result<bool> {
        SystemGroupController.group_closed(group)
    }
}

struct TransientProbeFailureController(AtomicUsize);

impl GroupController for TransientProbeFailureController {
    fn kill_group(&self, group: Pid) -> io::Result<()> {
        SystemGroupController.kill_group(group)
    }

    fn group_closed(&self, group: Pid) -> io::Result<bool> {
        if self.0.fetch_add(1, Ordering::AcqRel) == 0 {
            return Err(group_probe_error());
        }
        SystemGroupController.group_closed(group)
    }
}

struct PersistentProbeFailureController;

impl GroupController for PersistentProbeFailureController {
    fn kill_group(&self, group: Pid) -> io::Result<()> {
        SystemGroupController.kill_group(group)
    }

    fn group_closed(&self, _group: Pid) -> io::Result<bool> {
        Err(group_probe_error())
    }
}

fn group_probe_error() -> io::Error {
    io::Error::new(io::ErrorKind::PermissionDenied, "group_probe_unavailable")
}

#[test]
fn skips_an_injected_spawner_after_observing_an_expired_work_cutoff() {
    let spawner = CountingSpawner(AtomicUsize::new(0));
    let mut command = Command::new("sh");

    let error = run_command(
        &mut command,
        ProcessBudget::new(Instant::now() - Duration::from_millis(1)),
        &spawner,
        &SystemGroupController,
    )
    .unwrap_err();

    assert_eq!(error.kind(), io::ErrorKind::TimedOut);
    assert_eq!(spawner.0.load(Ordering::Acquire), 0);
}

#[test]
fn reaps_a_live_child_when_the_first_reader_creation_fails() {
    assert_reader_failure_reaps(1);
}

#[test]
fn reaps_a_live_child_when_the_second_reader_creation_fails() {
    assert_reader_failure_reaps(2);
}

#[test]
fn returns_promptly_when_group_kill_fails_but_the_leader_is_live() {
    let spawner = RecordingSpawner::new();
    let _group_guard = spawner.group_guard();
    let mut command = sleep_command();
    let started = Instant::now();

    let error = run_command(
        &mut command,
        ProcessBudget::new(Instant::now() + Duration::from_millis(120)),
        &spawner,
        &FailingGroupController,
    )
    .unwrap_err();

    assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
    assert!(started.elapsed() < Duration::from_millis(250));
    assert_child_reaped(spawner.pid());
}

#[test]
fn returns_success_after_a_transient_group_probe_error_and_verified_closure() {
    let spawner = RecordingSpawner::new();
    let _group_guard = spawner.group_guard();
    let controller = TransientProbeFailureController(AtomicUsize::new(0));
    let mut command = successful_command();

    let output = run_command(
        &mut command,
        ProcessBudget::new(Instant::now() + Duration::from_millis(500)),
        &spawner,
        &controller,
    )
    .unwrap();

    assert!(output.success());
    assert_eq!(controller.0.load(Ordering::Acquire), 2);
}

#[test]
fn returns_a_persistent_group_probe_error_at_the_cleanup_deadline() {
    let spawner = RecordingSpawner::new();
    let _group_guard = spawner.group_guard();
    let mut command = successful_command();
    let started = Instant::now();
    let cleanup_deadline = started + Duration::from_millis(120);

    let error = run_command(
        &mut command,
        ProcessBudget::new(cleanup_deadline),
        &spawner,
        &PersistentProbeFailureController,
    )
    .unwrap_err();

    assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
    assert_eq!(error.to_string(), "group_probe_unavailable");
    assert!((Duration::from_millis(120)..Duration::from_millis(250)).contains(&started.elapsed()));
}

fn assert_reader_failure_reaps(fail_on: usize) {
    let spawner = RecordingSpawner::new();
    let _group_guard = spawner.group_guard();
    let readers = FailingReaderSpawner {
        fail_on,
        attempts: AtomicUsize::new(0),
    };
    let mut command = sleep_command();

    let error = run_command_with_readers(
        &mut command,
        ProcessBudget::new(Instant::now() + Duration::from_millis(500)),
        &spawner,
        &readers,
        &SystemGroupController,
    )
    .unwrap_err();

    assert_eq!(error.kind(), io::ErrorKind::Other);
    assert_eq!(readers.attempts.load(Ordering::Acquire), fail_on);
    assert_child_reaped(spawner.pid());
}

fn assert_child_reaped(pid: Pid) {
    assert!(matches!(
        waitpid(Some(pid), WaitOptions::NOHANG),
        Err(rustix::io::Errno::CHILD)
    ));
}

fn sleep_command() -> Command {
    let mut command = Command::new("sleep");
    command
        .arg("5")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0);
    command
}

fn successful_command() -> Command {
    let mut command = Command::new("sh");
    command
        .args(["-c", "printf ready"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0);
    command
}

#[test]
fn leaves_no_unbounded_wait_when_a_pipe_holding_descendant_outlives_a_failed_group_kill() {
    let spawner = RecordingSpawner::new();
    let _group_guard = spawner.group_guard();
    let mut command = Command::new("sh");
    command
        .args(["-c", "sleep 5 &"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0);
    let started = Instant::now();

    let error = run_command(
        &mut command,
        ProcessBudget::new(Instant::now() + Duration::from_millis(120)),
        &spawner,
        &FailingGroupController,
    )
    .unwrap_err();

    assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
    assert!(started.elapsed() < Duration::from_millis(250));
    assert_child_reaped(spawner.pid());
    assert!(test_kill_process_group(spawner.pid()).is_ok());
}
