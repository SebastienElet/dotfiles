use super::{codex::Tokens, fixture::Fixture, process::ExecutionError};
use super::{
    codex::parse_codex_events,
    process::{CaptureOptions, capture},
};
use std::{
    collections::BTreeMap,
    env, fs,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    time::{Duration, Instant},
};

#[derive(Default)]
pub struct Authentication {
    pub file: Option<PathBuf>,
    pub api_key: Option<String>,
}

pub struct LiveOptions {
    pub model: String,
    pub reasoning_effort: String,
    pub timeout_seconds: u64,
}

pub struct Execution {
    pub error: Option<ExecutionError>,
    pub tokens: Option<Tokens>,
    pub tool_calls: Option<u64>,
    pub duration_ms: Option<f64>,
}

pub struct Codex {
    pub command: PathBuf,
    pub authentication: Authentication,
    pub version: String,
}

impl Codex {
    /// # Errors
    /// Returns errors for unavailable Codex executables, non-UTF-8 environment entries,
    /// incompatible authentication, inaccessible working directories, or failed version discovery.
    pub fn discover() -> Result<Self, String> {
        let command = find_codex()?;
        let environment: BTreeMap<_, _> = env::vars_os()
            .map(|(name, value)| {
                let invalid = |_| "Codex environment must contain UTF-8 names and values";
                Ok((
                    name.into_string().map_err(invalid)?,
                    value.into_string().map_err(invalid)?,
                ))
            })
            .collect::<Result<_, &str>>()?;
        let authentication = authentication_from_environment(&environment)?;
        let cwd = env::current_dir().map_err(|error| error.to_string())?;
        let version = capture(
            &command,
            &["--version".into()],
            CaptureOptions {
                cwd: &cwd,
                env: &environment,
                stdin: "",
                timeout: Duration::from_secs(5),
            },
        );
        if version.error.is_some() {
            return Err("Cannot identify Codex version".into());
        }
        Ok(Self {
            command,
            authentication,
            version: version.output.trim().into(),
        })
    }

    /// # Errors
    /// Returns errors preparing the isolated authentication environment; provider failures are recorded in the returned execution.
    pub fn execute(
        &self,
        fixture: &Fixture,
        prompt: &str,
        options: &LiveOptions,
    ) -> Result<Execution, String> {
        let environment = self.environment(fixture)?;
        let started = Instant::now();
        let result = capture(
            &self.command,
            &codex_arguments(&options.model, &options.reasoning_effort),
            CaptureOptions {
                cwd: &fixture.workspace,
                env: &environment,
                stdin: prompt,
                timeout: Duration::from_secs(options.timeout_seconds),
            },
        );
        let duration_ms = Some((started.elapsed().as_secs_f64() * 1000.0).round());
        let mut execution = Execution {
            error: result.error,
            duration_ms,
            tokens: None,
            tool_calls: None,
        };
        if execution.error.is_some() {
            return Ok(execution);
        }
        match parse_codex_events(&result.output) {
            Ok(metrics) => {
                execution.tokens = Some(metrics.tokens);
                execution.tool_calls = Some(metrics.tool_calls);
            }
            Err(_) => execution.error = Some(ExecutionError::ProtocolInvalid),
        }
        Ok(execution)
    }

    fn environment(&self, fixture: &Fixture) -> Result<BTreeMap<String, String>, String> {
        if let Some(file) = &self.authentication.file {
            let codex_home = fixture
                .env
                .get("CODEX_HOME")
                .ok_or("Missing fixture CODEX_HOME")?;
            fs::copy(file, PathBuf::from(codex_home).join("auth.json"))
                .map_err(|error| error.to_string())?;
        }
        let mut environment = fixture.env.clone();
        let parent = self
            .command
            .parent()
            .and_then(|path| path.to_str())
            .ok_or("Invalid Codex executable path")?;
        let path = environment.get("PATH").ok_or("Missing fixture PATH")?;
        environment.insert("PATH".into(), format!("{path}:{parent}"));
        if let Some(key) = &self.authentication.api_key {
            environment.insert("CODEX_API_KEY".into(), key.clone());
        }
        Ok(environment)
    }
}

pub fn codex_arguments(model: &str, reasoning_effort: &str) -> Vec<String> {
    [
        "exec",
        "--json",
        "--ephemeral",
        "--ignore-user-config",
        "--ignore-rules",
        "--skip-git-repo-check",
        "--model",
        model,
        "--sandbox",
        "workspace-write",
        "-c",
        "approval_policy=\"never\"",
        "-c",
        "sandbox_workspace_write.network_access=false",
        "-c",
        "web_search=\"disabled\"",
        "-c",
        &format!("model_reasoning_effort=\"{reasoning_effort}\""),
        "-c",
        "shell_environment_policy.inherit=\"all\"",
        "-c",
        "shell_environment_policy.ignore_default_excludes=false",
        "-c",
        "shell_environment_policy.experimental_use_profile=false",
        "-",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

fn authentication_from_environment(
    environment: &BTreeMap<String, String>,
) -> Result<Authentication, String> {
    if let Some(key) = environment
        .get("CODEX_API_KEY")
        .filter(|key| !key.is_empty())
    {
        return Ok(Authentication {
            file: None,
            api_key: Some(key.clone()),
        });
    }
    let home = environment.get("CODEX_HOME").map_or_else(
        || PathBuf::from(environment.get("HOME").map_or("", String::as_str)).join(".codex"),
        PathBuf::from,
    );
    let file = home.join("auth.json");
    if file.exists() {
        return Ok(Authentication {
            file: Some(file),
            api_key: None,
        });
    }
    Err("Manual eval requires saved Codex auth or CODEX_API_KEY".into())
}

fn find_codex() -> Result<PathBuf, String> {
    let path = env::var_os("PATH").unwrap_or_default();
    env::split_paths(&path)
        .map(|directory| directory.join("codex"))
        .find(|candidate| {
            candidate.metadata().is_ok_and(|metadata| {
                metadata.is_file() && metadata.permissions().mode() & 0o111 != 0
            })
        })
        .and_then(|candidate| fs::canonicalize(candidate).ok())
        .ok_or_else(|| "Codex CLI is not installed".into())
}

#[cfg(test)]
mod tests;
