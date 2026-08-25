use super::{ExportError, decode_git_paths, run_git};
use std::path::Path;

pub(super) fn reject_ignored_harness_paths(repository: &Path) -> Result<(), ExportError> {
    let output = run_git(
        repository,
        &[
            "ls-files",
            "--others",
            "--ignored",
            "--exclude-standard",
            "-z",
            "--",
            "harness",
        ],
    )?;
    for path in decode_git_paths(output)? {
        if !is_ignored_noise(&path) {
            return Err(ExportError::new(format!(
                "refusing ignored harness source {path}"
            )));
        }
    }
    Ok(())
}

fn is_ignored_noise(path: &str) -> bool {
    path.ends_with("/.DS_Store") || path.contains("/.claude-flow/")
}

pub(super) fn reject_sensitive_path(path: &str) -> Result<(), ExportError> {
    let name = path.rsplit('/').next().unwrap_or("").to_ascii_lowercase();
    let sensitive = name.starts_with(".env")
        || name == "api_key"
        || name.starts_with("api_key.")
        || name == "api-key"
        || name.starts_with("api-key.")
        || name == "apikey"
        || name.starts_with("apikey.")
        || name == "access_key"
        || name.starts_with("access_key.")
        || name == "password"
        || name.starts_with("password.")
        || name == "private_key"
        || name.starts_with("private_key.")
        || name == "credentials"
        || name.starts_with("credentials.")
        || name.contains("-credentials.")
        || name == "secret"
        || name.starts_with("secret.")
        || name.contains("-secret.")
        || name == "secrets"
        || name.starts_with("secrets.")
        || name.contains("-secrets.")
        || name == "token"
        || name.starts_with("token.")
        || name.contains("-token.")
        || name.contains(".local.")
        || name.ends_with(".pem")
        || name.ends_with(".key");
    if sensitive {
        Err(ExportError::new(format!(
            "refusing sensitive harness path {path}"
        )))
    } else {
        Ok(())
    }
}
