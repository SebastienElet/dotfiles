use serde::de::DeserializeOwned;
use std::error::Error;
use std::fmt;
use std::io::{self, Read};
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

const COMMAND_TIMEOUT: Duration = Duration::from_secs(120);
const MAX_COMMAND_OUTPUT_BYTES: usize = 8 * 1024 * 1024;
const COMMAND_POLL_INTERVAL: Duration = Duration::from_millis(5);

mod posix {
    use std::ffi::c_int;
    use std::io;

    const SIGKILL: c_int = 9;
    const ESRCH: c_int = 3;

    unsafe extern "C" {
        #[link_name = "kill"]
        fn kill_process(process: c_int, signal: c_int) -> c_int;
    }

    pub fn kill_process_group(process_group: u32) -> io::Result<()> {
        let process_group = c_int::try_from(process_group)
            .map_err(|_| io::Error::other("process group ID does not fit in a POSIX pid_t"))?;
        if process_group <= 1 {
            return Err(io::Error::other(
                "refusing to signal an unsafe process group ID",
            ));
        }

        // SAFETY: kill accepts scalar integers, and the validated negative PID targets one group.
        if unsafe { kill_process(-process_group, SIGKILL) } == 0 {
            return Ok(());
        }

        let error = io::Error::last_os_error();
        if error.raw_os_error() == Some(ESRCH) {
            Ok(())
        } else {
            Err(error)
        }
    }
}

#[derive(Debug)]
pub struct CommandError {
    program: String,
    args: Vec<String>,
    kind: CommandErrorKind,
}

#[derive(Debug)]
enum CommandErrorKind {
    Spawn(io::Error),
    PipeUnavailable(&'static str),
    PipeRead {
        pipe: &'static str,
        error: io::Error,
    },
    ReaderSpawn {
        pipe: &'static str,
        error: io::Error,
    },
    PipeReaderPanicked(&'static str),
    ProcessGroupKill(io::Error),
    Wait(io::Error),
    Timeout {
        timeout: Duration,
    },
    OutputTooLarge {
        limit: usize,
    },
    Nonzero {
        status: ExitStatus,
        stderr: String,
    },
    Utf8(std::string::FromUtf8Error),
    Json(serde_json::Error),
}

impl CommandError {
    fn new(program: &str, args: &[String], kind: CommandErrorKind) -> Self {
        Self {
            program: program.to_owned(),
            args: args.to_vec(),
            kind,
        }
    }

    fn command(&self) -> String {
        std::iter::once(&self.program)
            .chain(&self.args)
            .map(|part| format!("{part:?}"))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl fmt::Display for CommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let command = self.command();
        match &self.kind {
            CommandErrorKind::Spawn(error) => {
                write!(formatter, "failed to spawn command {command}: {error}")
            }
            CommandErrorKind::PipeUnavailable(pipe) => {
                write!(formatter, "command {command} has no piped {pipe}")
            }
            CommandErrorKind::PipeRead { pipe, error } => {
                write!(
                    formatter,
                    "failed to read {pipe} from command {command}: {error}"
                )
            }
            CommandErrorKind::ReaderSpawn { pipe, error } => {
                write!(
                    formatter,
                    "failed to start {pipe} reader for command {command}: {error}"
                )
            }
            CommandErrorKind::PipeReaderPanicked(pipe) => {
                write!(formatter, "{pipe} reader for command {command} panicked")
            }
            CommandErrorKind::ProcessGroupKill(error) => {
                write!(
                    formatter,
                    "failed to terminate command group {command}: {error}"
                )
            }
            CommandErrorKind::Wait(error) => {
                write!(formatter, "failed to wait for command {command}: {error}")
            }
            CommandErrorKind::Timeout { timeout } => {
                write!(formatter, "command {command} timed out after {timeout:?}")
            }
            CommandErrorKind::OutputTooLarge { limit } => {
                write!(formatter, "command {command} output exceeded {limit} bytes")
            }
            CommandErrorKind::Nonzero { status, stderr } => {
                if stderr.is_empty() {
                    write!(formatter, "command {command} failed with {status}")
                } else {
                    write!(
                        formatter,
                        "command {command} failed with {status}: {stderr}"
                    )
                }
            }
            CommandErrorKind::Utf8(error) => {
                write!(
                    formatter,
                    "command {command} produced stdout that is not valid UTF-8: {error}"
                )
            }
            CommandErrorKind::Json(error) => {
                write!(
                    formatter,
                    "command {command} produced invalid JSON on stdout: {error}"
                )
            }
        }
    }
}

impl Error for CommandError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.kind {
            CommandErrorKind::Spawn(error) => Some(error),
            CommandErrorKind::PipeRead { error, .. } => Some(error),
            CommandErrorKind::ReaderSpawn { error, .. } => Some(error),
            CommandErrorKind::ProcessGroupKill(error) => Some(error),
            CommandErrorKind::Wait(error) => Some(error),
            CommandErrorKind::Utf8(error) => Some(error),
            CommandErrorKind::Json(error) => Some(error),
            CommandErrorKind::PipeUnavailable(_)
            | CommandErrorKind::PipeReaderPanicked(_)
            | CommandErrorKind::Timeout { .. }
            | CommandErrorKind::OutputTooLarge { .. }
            | CommandErrorKind::Nonzero { .. } => None,
        }
    }
}

pub fn run(program: &str, args: &[String]) -> Result<String, CommandError> {
    run_with_limits(program, args, COMMAND_TIMEOUT, MAX_COMMAND_OUTPUT_BYTES)
}

fn run_with_limits(
    program: &str,
    args: &[String],
    timeout: Duration,
    max_output_bytes: usize,
) -> Result<String, CommandError> {
    let limits = ExecutionLimits {
        timeout,
        max_output_bytes,
    };
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        // Provider CLIs may leave descendants holding inherited output pipes open.
        .process_group(0)
        .spawn()
        .map_err(|error| CommandError::new(program, args, CommandErrorKind::Spawn(error)))?;
    let process_group = child.id();

    let stdout = child.stdout.take();
    let stdout = take_pipe(&mut child, process_group, stdout, "stdout", program, args)?;
    let stderr = child.stderr.take();
    let stderr = take_pipe(&mut child, process_group, stderr, "stderr", program, args)?;
    let output_too_large = AtomicBool::new(false);
    let reader_failed = AtomicBool::new(false);
    let readers_finished = AtomicUsize::new(0);

    let execution = thread::scope(|scope| {
        let stdout_reader = match thread::Builder::new()
            .name("daily-routine-stdout".to_owned())
            .spawn_scoped(scope, || {
                read_bounded(
                    stdout,
                    limits.max_output_bytes,
                    &output_too_large,
                    &reader_failed,
                    &readers_finished,
                )
            }) {
            Ok(reader) => reader,
            Err(error) => {
                let cleanup_outcome =
                    stop_child(&mut child, process_group, ChildOutcome::ReaderFailed);
                return ScopedExecution::ReaderSpawnFailed {
                    pipe: "stdout",
                    error,
                    cleanup_outcome,
                };
            }
        };
        let stderr_reader = match thread::Builder::new()
            .name("daily-routine-stderr".to_owned())
            .spawn_scoped(scope, || {
                read_bounded(
                    stderr,
                    limits.max_output_bytes,
                    &output_too_large,
                    &reader_failed,
                    &readers_finished,
                )
            }) {
            Ok(reader) => reader,
            Err(error) => {
                let cleanup_outcome =
                    stop_child(&mut child, process_group, ChildOutcome::ReaderFailed);
                let _ = stdout_reader.join();
                return ScopedExecution::ReaderSpawnFailed {
                    pipe: "stderr",
                    error,
                    cleanup_outcome,
                };
            }
        };
        let child_outcome = wait_for_child(
            &mut child,
            process_group,
            limits.timeout,
            &output_too_large,
            &reader_failed,
            &readers_finished,
        );

        ScopedExecution::Complete {
            child_outcome,
            stdout_result: stdout_reader.join(),
            stderr_result: stderr_reader.join(),
        }
    });

    let output = match execution {
        ScopedExecution::Complete {
            child_outcome,
            stdout_result,
            stderr_result,
        } => finish_execution(
            child_outcome,
            stdout_result,
            stderr_result,
            output_too_large.load(Ordering::SeqCst),
            limits,
            program,
            args,
        )?,
        ScopedExecution::ReaderSpawnFailed {
            pipe,
            error,
            cleanup_outcome,
        } => {
            let kind = match cleanup_outcome {
                ChildOutcome::WaitFailed(error) => CommandErrorKind::Wait(error),
                ChildOutcome::ProcessGroupKillFailed(error) => {
                    CommandErrorKind::ProcessGroupKill(error)
                }
                _ => CommandErrorKind::ReaderSpawn { pipe, error },
            };
            return Err(CommandError::new(program, args, kind));
        }
    };

    if !output.status.success() {
        return Err(CommandError::new(
            program,
            args,
            CommandErrorKind::Nonzero {
                status: output.status,
                stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            },
        ));
    }

    String::from_utf8(output.stdout)
        .map_err(|error| CommandError::new(program, args, CommandErrorKind::Utf8(error)))
}

fn take_pipe<T>(
    child: &mut Child,
    process_group: u32,
    pipe: Option<T>,
    pipe_name: &'static str,
    program: &str,
    args: &[String],
) -> Result<T, CommandError> {
    match pipe {
        Some(pipe) => Ok(pipe),
        None => {
            let kind = match stop_child(child, process_group, ChildOutcome::ReaderFailed) {
                ChildOutcome::WaitFailed(error) => CommandErrorKind::Wait(error),
                ChildOutcome::ProcessGroupKillFailed(error) => {
                    CommandErrorKind::ProcessGroupKill(error)
                }
                _ => CommandErrorKind::PipeUnavailable(pipe_name),
            };
            Err(CommandError::new(program, args, kind))
        }
    }
}

fn read_bounded(
    pipe: impl Read,
    max_output_bytes: usize,
    output_too_large: &AtomicBool,
    reader_failed: &AtomicBool,
    readers_finished: &AtomicUsize,
) -> io::Result<Vec<u8>> {
    let read_limit = u64::try_from(max_output_bytes)
        .unwrap_or(u64::MAX)
        .saturating_add(1);
    let mut output = Vec::new();
    let result = pipe.take(read_limit).read_to_end(&mut output);

    let result = match result {
        Ok(_) => {
            if output.len() > max_output_bytes {
                output_too_large.store(true, Ordering::SeqCst);
            }
            Ok(output)
        }
        Err(error) => {
            reader_failed.store(true, Ordering::SeqCst);
            Err(error)
        }
    };
    readers_finished.fetch_add(1, Ordering::SeqCst);
    result
}

type PipeReadResult = thread::Result<io::Result<Vec<u8>>>;

enum ScopedExecution {
    Complete {
        child_outcome: ChildOutcome,
        stdout_result: PipeReadResult,
        stderr_result: PipeReadResult,
    },
    ReaderSpawnFailed {
        pipe: &'static str,
        error: io::Error,
        cleanup_outcome: ChildOutcome,
    },
}

#[derive(Debug)]
struct CommandOutput {
    status: ExitStatus,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

#[derive(Clone, Copy)]
struct ExecutionLimits {
    timeout: Duration,
    max_output_bytes: usize,
}

enum CompletedChildOutcome {
    Exited(ExitStatus),
    TimedOut,
    OutputTooLarge,
    ReaderFailed,
}

fn finish_execution(
    child_outcome: ChildOutcome,
    stdout_result: PipeReadResult,
    stderr_result: PipeReadResult,
    output_too_large: bool,
    limits: ExecutionLimits,
    program: &str,
    args: &[String],
) -> Result<CommandOutput, CommandError> {
    let child_outcome = match child_outcome {
        ChildOutcome::WaitFailed(error) => {
            return Err(CommandError::new(
                program,
                args,
                CommandErrorKind::Wait(error),
            ));
        }
        ChildOutcome::ProcessGroupKillFailed(error) => {
            return Err(CommandError::new(
                program,
                args,
                CommandErrorKind::ProcessGroupKill(error),
            ));
        }
        ChildOutcome::Exited(status) => CompletedChildOutcome::Exited(status),
        ChildOutcome::TimedOut => CompletedChildOutcome::TimedOut,
        ChildOutcome::OutputTooLarge => CompletedChildOutcome::OutputTooLarge,
        ChildOutcome::ReaderFailed => CompletedChildOutcome::ReaderFailed,
    };

    let (stdout, stderr) = finish_pipe_reads(stdout_result, stderr_result, program, args)?;

    if output_too_large {
        return Err(CommandError::new(
            program,
            args,
            CommandErrorKind::OutputTooLarge {
                limit: limits.max_output_bytes,
            },
        ));
    }

    match child_outcome {
        CompletedChildOutcome::Exited(status) => Ok(CommandOutput {
            status,
            stdout,
            stderr,
        }),
        CompletedChildOutcome::TimedOut => Err(CommandError::new(
            program,
            args,
            CommandErrorKind::Timeout {
                timeout: limits.timeout,
            },
        )),
        CompletedChildOutcome::OutputTooLarge => Err(CommandError::new(
            program,
            args,
            CommandErrorKind::OutputTooLarge {
                limit: limits.max_output_bytes,
            },
        )),
        CompletedChildOutcome::ReaderFailed => Err(CommandError::new(
            program,
            args,
            CommandErrorKind::PipeRead {
                pipe: "output",
                error: io::Error::other("pipe reader failed without returning its error"),
            },
        )),
    }
}

fn finish_pipe_reads(
    stdout_result: PipeReadResult,
    stderr_result: PipeReadResult,
    program: &str,
    args: &[String],
) -> Result<(Vec<u8>, Vec<u8>), CommandError> {
    match (stdout_result, stderr_result) {
        (Ok(Err(error)), _) => Err(CommandError::new(
            program,
            args,
            CommandErrorKind::PipeRead {
                pipe: "stdout",
                error,
            },
        )),
        (_, Ok(Err(error))) => Err(CommandError::new(
            program,
            args,
            CommandErrorKind::PipeRead {
                pipe: "stderr",
                error,
            },
        )),
        (Err(_), _) => Err(CommandError::new(
            program,
            args,
            CommandErrorKind::PipeReaderPanicked("stdout"),
        )),
        (_, Err(_)) => Err(CommandError::new(
            program,
            args,
            CommandErrorKind::PipeReaderPanicked("stderr"),
        )),
        (Ok(Ok(stdout)), Ok(Ok(stderr))) => Ok((stdout, stderr)),
    }
}

enum ChildOutcome {
    Exited(ExitStatus),
    TimedOut,
    OutputTooLarge,
    ReaderFailed,
    ProcessGroupKillFailed(io::Error),
    WaitFailed(io::Error),
}

fn wait_for_child(
    child: &mut Child,
    process_group: u32,
    timeout: Duration,
    output_too_large: &AtomicBool,
    reader_failed: &AtomicBool,
    readers_finished: &AtomicUsize,
) -> ChildOutcome {
    let started_at = Instant::now();
    let mut exit_status = None;

    loop {
        if output_too_large.load(Ordering::SeqCst) {
            return stop_child(child, process_group, ChildOutcome::OutputTooLarge);
        }
        if reader_failed.load(Ordering::SeqCst) {
            return stop_child(child, process_group, ChildOutcome::ReaderFailed);
        }

        if exit_status.is_none() {
            match child.try_wait() {
                Ok(Some(status)) => exit_status = Some(status),
                Ok(None) => {}
                Err(error) => {
                    return stop_child(child, process_group, ChildOutcome::WaitFailed(error));
                }
            }
        }
        if readers_finished.load(Ordering::SeqCst) == 2
            && let Some(status) = exit_status.take()
        {
            return ChildOutcome::Exited(status);
        }

        let elapsed = started_at.elapsed();
        if elapsed >= timeout {
            return stop_child(child, process_group, ChildOutcome::TimedOut);
        }

        thread::sleep(COMMAND_POLL_INTERVAL.min(timeout.saturating_sub(elapsed)));
    }
}

fn stop_child(child: &mut Child, process_group: u32, outcome: ChildOutcome) -> ChildOutcome {
    let first_group_kill = posix::kill_process_group(process_group);
    if first_group_kill.is_err() {
        let _ = child.kill();
    }

    if let Err(error) = child.wait() {
        return ChildOutcome::WaitFailed(error);
    }
    if matches!(&outcome, ChildOutcome::WaitFailed(_)) {
        return outcome;
    }

    if first_group_kill.is_err()
        && let Err(error) = posix::kill_process_group(process_group)
    {
        return ChildOutcome::ProcessGroupKillFailed(error);
    }

    outcome
}

pub fn run_json<T: DeserializeOwned>(program: &str, args: &[String]) -> Result<T, CommandError> {
    let stdout = run(program, args)?;
    serde_json::from_str(&stdout)
        .map_err(|error| CommandError::new(program, args, CommandErrorKind::Json(error)))
}

#[cfg(test)]
mod tests {
    use super::{
        ChildOutcome, CommandError, CommandErrorKind, ExecutionLimits, finish_execution, run,
        run_json, run_with_limits,
    };
    use serde::Deserialize;
    use std::error::Error as _;
    use std::io;
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};

    #[derive(Debug, Deserialize, Eq, PartialEq)]
    struct Response {
        value: u32,
    }

    #[test]
    fn decodes_json_from_a_successful_command() {
        let args = vec![r#"{"value":42}"#.to_owned()];

        let response: Response = run_json("/usr/bin/printf", &args).unwrap();

        assert_eq!(response, Response { value: 42 });
    }

    #[test]
    fn reports_spawn_errors_with_command_context() {
        let program = "daily-routine-command-that-does-not-exist";
        let args = vec!["argument".to_owned()];

        let error = run(program, &args).unwrap_err();
        let message = error.to_string();

        assert!(message.contains("spawn"), "{message}");
        assert!(message.contains(program), "{message}");
        assert!(message.contains("argument"), "{message}");
    }

    #[test]
    fn rejects_nonzero_exit_status() {
        let error = run("/usr/bin/false", &[]).unwrap_err();
        let message = error.to_string();

        assert!(message.contains("exit status: 1"), "{message}");
        assert!(message.contains("/usr/bin/false"), "{message}");
    }

    #[test]
    fn terminates_a_command_after_its_timeout() {
        let args = vec!["1".to_owned()];
        let started_at = Instant::now();

        let error =
            run_with_limits("/bin/sleep", &args, Duration::from_millis(20), 1_024).unwrap_err();
        let elapsed = started_at.elapsed();
        let message = error.to_string();

        assert!(message.contains("timed out"), "{message}");
        assert!(message.contains("/bin/sleep"), "{message}");
        assert!(elapsed < Duration::from_millis(900), "elapsed {elapsed:?}");
    }

    #[test]
    fn terminates_a_command_when_output_exceeds_the_limit() {
        let args = vec!["123456789".to_owned()];

        let error =
            run_with_limits("/usr/bin/printf", &args, Duration::from_secs(1), 8).unwrap_err();
        let message = error.to_string();

        assert!(message.contains("exceeded 8 bytes"), "{message}");
        assert!(message.contains("/usr/bin/printf"), "{message}");
    }

    #[test]
    fn commands_receive_eof_instead_of_inheriting_parent_stdin() {
        let test_binary = std::env::current_exe().unwrap();
        let mut helper = Command::new(test_binary)
            .arg("command::tests::stdin_reader_receives_eof")
            .arg("--exact")
            .env("DAILY_ROUTINE_STDIN_HELPER", "1")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let inherited_stdin = helper.stdin.take().unwrap();

        let output = helper.wait_with_output().unwrap();
        drop(inherited_stdin);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert!(
            output.status.success(),
            "helper failed\nstdout: {stdout}\nstderr: {stderr}"
        );
        assert!(stdout.contains("1 passed"), "helper did not run: {stdout}");
    }

    #[test]
    fn stdin_reader_receives_eof() {
        if std::env::var_os("DAILY_ROUTINE_STDIN_HELPER").is_none() {
            return;
        }

        let stdout = run_with_limits("/bin/cat", &[], Duration::from_millis(100), 1_024).unwrap();

        assert!(stdout.is_empty());
    }

    #[test]
    fn timeout_terminates_descendants_holding_output_pipes() {
        let args = vec!["-c".to_owned(), "sleep 10 & wait".to_owned()];
        let started_at = Instant::now();

        let error =
            run_with_limits("/bin/sh", &args, Duration::from_millis(20), 1_024).unwrap_err();
        let elapsed = started_at.elapsed();
        let message = error.to_string();

        assert!(message.contains("timed out"), "{message}");
        assert!(elapsed < Duration::from_secs(1), "elapsed {elapsed:?}");
    }

    #[test]
    fn timeout_remains_active_after_the_direct_child_exits() {
        let args = vec!["-c".to_owned(), "sleep 10 &".to_owned()];
        let started_at = Instant::now();

        let error =
            run_with_limits("/bin/sh", &args, Duration::from_millis(20), 1_024).unwrap_err();
        let elapsed = started_at.elapsed();
        let message = error.to_string();

        assert!(message.contains("timed out"), "{message}");
        assert!(elapsed < Duration::from_secs(1), "elapsed {elapsed:?}");
    }

    #[test]
    fn reader_spawn_errors_are_explicit_and_keep_their_source() {
        let error = CommandError::new(
            "/usr/bin/printf",
            &[],
            CommandErrorKind::ReaderSpawn {
                pipe: "stdout",
                error: io::Error::other("thread unavailable"),
            },
        );
        let message = error.to_string();

        assert!(message.contains("start stdout reader"), "{message}");
        assert!(message.contains("thread unavailable"), "{message}");
        assert_eq!(error.source().unwrap().to_string(), "thread unavailable");
    }

    #[test]
    fn wait_errors_take_priority_over_pipe_errors_after_join() {
        let error = finish_execution(
            ChildOutcome::WaitFailed(io::Error::other("wait unavailable")),
            Ok(Err(io::Error::other("stdout unavailable"))),
            Ok(Ok(Vec::new())),
            false,
            ExecutionLimits {
                timeout: Duration::from_secs(1),
                max_output_bytes: 1_024,
            },
            "/usr/bin/printf",
            &[],
        )
        .unwrap_err();
        let message = error.to_string();

        assert!(message.contains("wait unavailable"), "{message}");
        assert!(!message.contains("stdout unavailable"), "{message}");
    }

    #[test]
    fn reader_failures_preserve_the_exact_pipe_error() {
        let error = finish_execution(
            ChildOutcome::ReaderFailed,
            Ok(Ok(Vec::new())),
            Ok(Err(io::Error::other("stderr unavailable"))),
            false,
            ExecutionLimits {
                timeout: Duration::from_secs(1),
                max_output_bytes: 1_024,
            },
            "/usr/bin/printf",
            &[],
        )
        .unwrap_err();
        let message = error.to_string();

        assert!(message.contains("read stderr"), "{message}");
        assert!(message.contains("stderr unavailable"), "{message}");
    }

    #[test]
    fn rejects_invalid_json_with_command_context() {
        let args = vec!["not-json".to_owned()];

        let error = run_json::<Response>("/usr/bin/printf", &args).unwrap_err();
        let message = error.to_string();

        assert!(message.contains("JSON"), "{message}");
        assert!(message.contains("/usr/bin/printf"), "{message}");
        assert!(message.contains("not-json"), "{message}");
    }

    #[test]
    fn rejects_non_utf8_stdout_explicitly() {
        let args = vec![r#"\377"#.to_owned()];

        let error = run("/usr/bin/printf", &args).unwrap_err();
        let message = error.to_string();

        assert!(message.contains("UTF-8"), "{message}");
        assert!(message.contains("/usr/bin/printf"), "{message}");
    }
}
