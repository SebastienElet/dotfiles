use super::budget::Deadlines;
use super::cleanup::{GroupController, close_and_reap, process_deadline};
use super::readers::{ReaderSpawner, Readers, SystemReaderSpawner, set_nonblocking};
use crate::memory::ProcessOutput;
use std::io;
use std::process::{Child, Command};
use std::thread;
use std::time::Duration;

const POLL_INTERVAL: Duration = Duration::from_millis(5);

pub(super) trait CommandSpawner {
    fn spawn(&self, command: &mut Command) -> io::Result<Child>;
}

pub(super) struct SystemCommandSpawner;

impl CommandSpawner for SystemCommandSpawner {
    fn spawn(&self, command: &mut Command) -> io::Result<Child> {
        command.spawn()
    }
}

pub(super) fn run_command(
    command: &mut Command,
    deadlines: Deadlines,
    spawner: &dyn CommandSpawner,
    controller: &dyn GroupController,
) -> io::Result<ProcessOutput> {
    run_command_with_readers(
        command,
        deadlines,
        spawner,
        &SystemReaderSpawner,
        controller,
    )
}

fn run_command_with_readers(
    command: &mut Command,
    deadlines: Deadlines,
    spawner: &dyn CommandSpawner,
    reader_spawner: &dyn ReaderSpawner,
    controller: &dyn GroupController,
) -> io::Result<ProcessOutput> {
    if deadlines.work_expired() {
        return Err(process_deadline());
    }
    let mut child = spawner.spawn(command)?;
    let mut readers = Readers::new();
    if deadlines.work_expired() {
        return fail_after_cleanup(
            &mut child,
            &mut readers,
            deadlines,
            controller,
            process_deadline(),
        );
    }
    let (stdout, stderr) = take_pipes(&mut child, &mut readers, deadlines, controller)?;
    if let Err(error) = set_nonblocking(&stdout).and_then(|()| set_nonblocking(&stderr)) {
        return fail_after_cleanup(&mut child, &mut readers, deadlines, controller, error);
    }
    if let Err(error) = readers.start_stdout(stdout, reader_spawner) {
        return fail_after_cleanup(&mut child, &mut readers, deadlines, controller, error);
    }
    if let Err(error) = readers.start_stderr(stderr, reader_spawner) {
        return fail_after_cleanup(&mut child, &mut readers, deadlines, controller, error);
    }
    wait_for_process(&mut child, &mut readers, deadlines, controller)
}

fn take_pipes(
    child: &mut Child,
    readers: &mut Readers,
    deadlines: Deadlines,
    controller: &dyn GroupController,
) -> io::Result<(std::process::ChildStdout, std::process::ChildStderr)> {
    match (child.stdout.take(), child.stderr.take()) {
        (Some(stdout), Some(stderr)) => Ok((stdout, stderr)),
        _ => fail_after_cleanup(child, readers, deadlines, controller, pipe_unavailable()),
    }
}

fn wait_for_process(
    child: &mut Child,
    readers: &mut Readers,
    deadlines: Deadlines,
    controller: &dyn GroupController,
) -> io::Result<ProcessOutput> {
    let mut status = None;
    loop {
        if status.is_none() {
            match child.try_wait() {
                Ok(Some(completed)) => status = Some(completed),
                Ok(None) => {}
                Err(error) => {
                    return fail_after_cleanup(child, readers, deadlines, controller, error);
                }
            }
        }
        if let Some(error) = readers.failure() {
            return fail_after_cleanup(child, readers, deadlines, controller, error);
        }
        if let Some(status) = status
            && readers.finished()
        {
            return complete_after_cleanup(child, readers, deadlines, controller, status);
        }
        if deadlines.work_expired() {
            return fail_after_cleanup(child, readers, deadlines, controller, process_deadline());
        }
        thread::sleep(POLL_INTERVAL.min(deadlines.remaining_work()));
    }
}

fn complete_after_cleanup(
    child: &mut Child,
    readers: &mut Readers,
    deadlines: Deadlines,
    controller: &dyn GroupController,
    status: std::process::ExitStatus,
) -> io::Result<ProcessOutput> {
    close_and_reap(child, readers, deadlines.hard(), controller)?;
    let (stdout, stderr) = readers.join_outputs()?;
    Ok(ProcessOutput::new(
        status.success(),
        status.code(),
        stdout,
        stderr,
    ))
}

fn fail_after_cleanup<T>(
    child: &mut Child,
    readers: &mut Readers,
    deadlines: Deadlines,
    controller: &dyn GroupController,
    error: io::Error,
) -> io::Result<T> {
    let cleanup = close_and_reap(child, readers, deadlines.hard(), controller);
    let joined = readers.join_finished();
    match cleanup {
        Ok(()) => match joined {
            Ok(()) => Err(error),
            Err(join_error) => Err(join_error),
        },
        Err(cleanup_error) => Err(cleanup_error),
    }
}

fn pipe_unavailable() -> io::Error {
    io::Error::other("process_pipe_unavailable")
}

#[cfg(test)]
mod join_failure;
#[cfg(test)]
mod tests;
