use super::sources::fingerprint;
use std::path::Path;
use std::process::Command;

pub fn runner_revision(executable: &Path) -> Result<String, String> {
    std::fs::read(executable)
        .map(fingerprint)
        .map_err(|error| format!("Cannot identify evaluation executable: {error}"))
}

pub fn git_revision(repository: &Path) -> Result<String, String> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repository)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .output()
        .map_err(|error| format!("Cannot identify harness Git revision: {error}"))?;
    if !output.status.success() {
        return Err("Cannot identify harness Git revision".to_owned());
    }
    String::from_utf8(output.stdout)
        .map(|revision| revision.trim().to_owned())
        .map_err(|error| error.to_string())
}

pub fn date() -> Result<String, String> {
    let output = Command::new("date")
        .args(["-u", "+%Y-%m-%dT%H:%M:%SZ"])
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .output()
        .map_err(|error| format!("Cannot identify evaluation date: {error}"))?;
    if !output.status.success() {
        return Err("Cannot identify evaluation date".to_owned());
    }
    String::from_utf8(output.stdout)
        .map(|date| date.trim().to_owned())
        .map_err(|error| error.to_string())
}
