use super::CliFailure;
use rustix::fs::{Mode, OFlags};
use serde_json::json;
use std::env;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const ROOT_ENVIRONMENT: &str = "AGENT_MEMORY_EVAL_ROOT";
const TRACE_ENVIRONMENT: &str = "AGENT_MEMORY_EVAL_TRACE";
const AGENT_ENVIRONMENT: &str = "AGENT_MEMORY_EVAL_AGENT";

pub(super) struct EvaluationTrace {
    writer: Option<TraceWriter>,
}

struct TraceWriter {
    agent: String,
    command: &'static str,
    output: BufWriter<File>,
}

impl EvaluationTrace {
    pub(super) fn from_environment(command: &'static str) -> Result<Self, CliFailure> {
        let Some(path) = env::var_os(TRACE_ENVIRONMENT) else {
            return Ok(Self { writer: None });
        };
        let root = required_path(ROOT_ENVIRONMENT)?;
        let agent = required_agent()?;
        let output = open_trace(&root, &PathBuf::from(path))?;
        let mut writer = TraceWriter {
            agent,
            command,
            output: BufWriter::new(output),
        };
        writer.write("started", "started")?;
        Ok(Self {
            writer: Some(writer),
        })
    }

    pub(super) fn finish(&mut self, exit: u8) -> Result<(), CliFailure> {
        let Some(writer) = &mut self.writer else {
            return Ok(());
        };
        if exit == 0 {
            writer.write("completed", "success")
        } else {
            writer.write("error", exit_class(exit))
        }
    }
}

impl TraceWriter {
    fn write(&mut self, event: &str, exit_class: &str) -> Result<(), CliFailure> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| failure())?
            .as_millis();
        let value = json!({
            "agent": self.agent,
            "command": self.command,
            "event": event,
            "exit_class": exit_class,
            "pid": std::process::id(),
            "timestamp_ms": timestamp,
        });
        serde_json::to_writer(&mut self.output, &value).map_err(|_| failure())?;
        self.output
            .write_all(b"\n")
            .and_then(|()| self.output.flush())
            .map_err(|_| failure())
    }
}

fn required_path(name: &str) -> Result<PathBuf, CliFailure> {
    env::var_os(name).map(PathBuf::from).ok_or_else(failure)
}

fn required_agent() -> Result<String, CliFailure> {
    let agent = env::var(AGENT_ENVIRONMENT).map_err(|_| failure())?;
    if matches!(agent.as_str(), "codex" | "claude" | "cursor") {
        return Ok(agent);
    }
    Err(failure())
}

fn open_trace(root: &Path, path: &Path) -> Result<File, CliFailure> {
    if !is_absolute_normal(root) || !is_absolute_normal(path) {
        return Err(failure());
    }
    let root_metadata = fs::symlink_metadata(root).map_err(|_| failure())?;
    if root_metadata.file_type().is_symlink() || !root_metadata.is_dir() {
        return Err(failure());
    }
    let relative = path.strip_prefix(root).map_err(|_| failure())?;
    let parent = relative.parent().ok_or_else(failure)?;
    let mut directory = rustix::fs::open(
        root,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(|_| failure())?;
    for component in parent.components() {
        let Component::Normal(name) = component else {
            return Err(failure());
        };
        directory = rustix::fs::openat(
            &directory,
            name,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(|_| failure())?;
    }
    let name = relative.file_name().ok_or_else(failure)?;
    let file = rustix::fs::openat(
        &directory,
        name,
        OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::RUSR | Mode::WUSR,
    )
    .map(File::from)
    .map_err(|_| failure())?;
    let mode = file.metadata().map_err(|_| failure())?;
    if mode.permissions().mode() & 0o777 != 0o600 {
        return Err(failure());
    }
    Ok(file)
}

fn is_absolute_normal(path: &Path) -> bool {
    path.is_absolute()
        && path
            .components()
            .all(|part| matches!(part, Component::RootDir | Component::Normal(_)))
}

fn failure() -> CliFailure {
    CliFailure::evaluation_trace_unavailable()
}

fn exit_class(exit: u8) -> &'static str {
    match exit {
        2 => "rejection",
        3 => "conflict",
        _ => "unavailable",
    }
}
