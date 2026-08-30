use super::readers::Readers;
use rustix::process::{Pid, Signal, kill_process_group, test_kill_process_group};
use std::io;
use std::process::Child;
use std::thread;
use std::time::{Duration, Instant};

const POLL_INTERVAL: Duration = Duration::from_millis(5);

pub(super) trait GroupController {
    fn kill_group(&self, group: Pid) -> io::Result<()>;
    fn group_closed(&self, group: Pid) -> io::Result<bool>;
}

pub(super) struct SystemGroupController;

impl GroupController for SystemGroupController {
    fn kill_group(&self, group: Pid) -> io::Result<()> {
        match kill_process_group(group, Signal::KILL) {
            Ok(()) | Err(rustix::io::Errno::SRCH) => Ok(()),
            Err(error) => Err(io::Error::from_raw_os_error(error.raw_os_error())),
        }
    }

    fn group_closed(&self, group: Pid) -> io::Result<bool> {
        match test_kill_process_group(group) {
            Ok(()) => Ok(false),
            Err(rustix::io::Errno::SRCH) => Ok(true),
            Err(error) => Err(io::Error::from_raw_os_error(error.raw_os_error())),
        }
    }
}

pub(super) fn close_and_reap(
    child: &mut Child,
    readers: &mut Readers,
    deadline: Instant,
    controller: &dyn GroupController,
) -> io::Result<()> {
    readers.cancel();
    let group = Pid::from_raw(child.id() as i32).ok_or_else(group_unavailable)?;
    let mut error = controller.kill_group(group).err();
    if error.is_some() {
        let _ = child.kill();
    }
    wait_for_closure(child, readers, group, deadline, controller, &mut error)
}

fn wait_for_closure(
    child: &mut Child,
    readers: &Readers,
    group: Pid,
    deadline: Instant,
    controller: &dyn GroupController,
    error: &mut Option<io::Error>,
) -> io::Result<()> {
    loop {
        let reaped = poll_child(child, error);
        let closed = poll_group(controller, group, error);
        if reaped && closed && readers.finished() {
            return error.take().map_or(Ok(()), Err);
        }
        if Instant::now() >= deadline {
            return Err(error.take().unwrap_or_else(process_deadline));
        }
        thread::sleep(POLL_INTERVAL.min(deadline.saturating_duration_since(Instant::now())));
    }
}

fn poll_child(child: &mut Child, error: &mut Option<io::Error>) -> bool {
    match child.try_wait() {
        Ok(Some(_)) => true,
        Ok(None) => false,
        Err(observed) => {
            remember(error, observed);
            false
        }
    }
}

fn poll_group(controller: &dyn GroupController, group: Pid, error: &mut Option<io::Error>) -> bool {
    match controller.group_closed(group) {
        Ok(closed) => closed,
        Err(observed) => {
            remember(error, observed);
            false
        }
    }
}

fn remember(error: &mut Option<io::Error>, observed: io::Error) {
    if error.is_none() {
        *error = Some(observed);
    }
}

pub(super) fn process_deadline() -> io::Error {
    io::Error::new(io::ErrorKind::TimedOut, "process_deadline")
}

fn group_unavailable() -> io::Error {
    io::Error::other("process_group_unavailable")
}
