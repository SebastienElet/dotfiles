use crate::model::Fallible;
use rustix::process::{Pid, Signal, kill_process_group};
use serde_json::Value;
use std::{
    io::{Read, Seek, SeekFrom},
    os::unix::process::CommandExt,
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

const OUTPUT_LIMIT: u64 = 4_194_304;

fn stop(child: &mut Child) -> Fallible<()> {
    match kill_process_group(Pid::from_child(child), Signal::KILL) {
        Ok(()) | Err(rustix::io::Errno::SRCH) => Ok(()),
        Err(_) => {
            let _ = child.kill();
            Err("provider process cleanup failed")
        }
    }
}

pub fn command(name: &str, args: &[String], timeout: Duration) -> Fallible<Value> {
    let mut output = tempfile::tempfile().map_err(|_| "provider capture unavailable")?;
    let errors = tempfile::tempfile().map_err(|_| "provider capture unavailable")?;
    let mut child = Command::new(name)
        .args(args)
        .env("LINEAR_IGNORE_ENV_FILE", "1")
        .stdin(Stdio::null())
        .stdout(
            output
                .try_clone()
                .map_err(|_| "provider capture unavailable")?,
        )
        .stderr(
            errors
                .try_clone()
                .map_err(|_| "provider capture unavailable")?,
        )
        .process_group(0)
        .spawn()
        .map_err(|_| "provider unavailable")?;
    let started = Instant::now();
    let result = loop {
        match (output.metadata(), errors.metadata()) {
            (Ok(out), Ok(err)) if out.len().saturating_add(err.len()) <= OUTPUT_LIMIT => {}
            _ => break Err("provider capture failed or exceeded output limit"),
        }
        if started.elapsed() >= timeout {
            break Err("provider timed out; mutation outcome may be unknown");
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                break if status.success() {
                    Ok(())
                } else {
                    Err("provider exited unsuccessfully")
                };
            }
            Ok(None) => thread::sleep(Duration::from_millis(5)),
            Err(_) => break Err("provider status unavailable"),
        }
    };
    let cleanup = stop(&mut child);
    let waited = child.wait().map_err(|_| "provider wait failed");
    result?;
    cleanup?;
    waited?;
    output
        .seek(SeekFrom::Start(0))
        .map_err(|_| "provider capture failed")?;
    let mut bytes = Vec::new();
    output
        .take(OUTPUT_LIMIT + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "provider capture failed")?;
    if bytes.len() > usize::try_from(OUTPUT_LIMIT).map_err(|_| "provider output limit invalid")? {
        return Err("provider output exceeded limit");
    }
    serde_json::from_slice(&bytes).map_err(|_| "provider returned invalid JSON")
}

#[cfg(test)]
mod tests {
    use super::command;
    use std::time::Duration;

    #[test]
    fn timed_out_provider_is_failure_with_unknown_mutation_outcome() {
        assert_eq!(
            command("/bin/sleep", &["2".into()], Duration::from_millis(30)),
            Err("provider timed out; mutation outcome may be unknown")
        );
    }

    #[test]
    fn successful_process_with_non_json_output_is_failure() {
        assert_eq!(
            command("/bin/echo", &["not-json".into()], Duration::from_secs(2)),
            Err("provider returned invalid JSON")
        );
    }

    #[test]
    fn unbounded_provider_output_is_stopped_and_refused() {
        assert_eq!(
            command("/usr/bin/yes", &[], Duration::from_secs(2)),
            Err("provider capture failed or exceeded output limit")
        );
    }
}
