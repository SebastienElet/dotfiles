use super::*;
use std::{collections::BTreeMap, fs, os::unix::fs::PermissionsExt, path::Path};

fn provider(
    script: &str,
    authentication: Authentication,
) -> Result<(tempfile::TempDir, Codex), std::io::Error> {
    let directory = tempfile::tempdir()?;
    let command = directory.path().join("provider");
    fs::write(&command, format!("#!/bin/sh\n{script}\n"))?;
    fs::set_permissions(&command, fs::Permissions::from_mode(0o755))?;
    Ok((
        directory,
        Codex {
            command,
            authentication,
            version: "synthetic".into(),
        },
    ))
}

fn options() -> LiveOptions {
    LiveOptions {
        model: "explicit-model".into(),
        reasoning_effort: "low".into(),
        timeout_seconds: 2,
    }
}

#[test]
fn rejects_non_utf8_environment() -> Result<(), Box<dyn std::error::Error>> {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    use std::process::Command;

    if std::env::var_os("ARNES_UTF8_ENV_TEST").is_some() {
        assert!(
            Codex::discover().is_err_and(
                |error| error == "Codex environment must contain UTF-8 names and values"
            )
        );
        return Ok(());
    }
    let directory = tempfile::tempdir()?;
    let executable = directory.path().join("codex");
    fs::write(&executable, "#!/bin/sh\nprintf 'codex-test\\n'\n")?;
    fs::set_permissions(&executable, fs::Permissions::from_mode(0o755))?;
    for (name, value) in [
        (
            OsString::from("INVALID_VALUE"),
            OsString::from_vec(vec![0xff]),
        ),
        (
            OsString::from_vec(vec![0xff]),
            OsString::from("private-value"),
        ),
    ] {
        let output = Command::new(std::env::current_exe()?)
            .args(["--exact", "eval::live::tests::rejects_non_utf8_environment"])
            .env("ARNES_UTF8_ENV_TEST", "1")
            .env("PATH", directory.path())
            .env(name, value)
            .output()?;
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
    Ok(())
}

#[test]
fn live_protocol_runs_with_exact_stdin_fresh_home_and_explicit_controls()
-> Result<(), Box<dyn std::error::Error>> {
    let (_directory, codex) = provider(
        "/bin/cat > request.txt\nprintf '%s\\n' \"$HOME\" > home.txt\nprintf '%s\\n' \"$@\" > arguments.txt\nprintf '%s' \"${PRIVATE_SENTINEL-unset}\" > sentinel.txt\nprintf '%s\\n' '{\"type\":\"turn.completed\",\"usage\":{\"input_tokens\":4,\"cached_input_tokens\":0,\"output_tokens\":2}}'",
        Authentication::default(),
    )?;
    let fixture = Fixture::prepare(
        &BTreeMap::new(),
        "instructions",
        "skill",
        Path::new("/tmp/arnes"),
    )?;
    let result = codex.execute(&fixture, " é\n\n", &options())?;
    assert_eq!(result.error, None);
    assert_eq!(
        result.tokens,
        Some(Tokens {
            input: 4,
            cached_input: 0,
            output: 2
        })
    );
    assert_eq!(result.tool_calls, Some(0));
    assert!(result.duration_ms.ok_or("missing fixture value")? >= 0.0);
    assert_eq!(
        fs::read_to_string(fixture.workspace.join("request.txt"))?,
        " é\n\n"
    );
    assert_eq!(
        fs::read_to_string(fixture.workspace.join("home.txt"))?.trim(),
        fixture.home.to_str().ok_or("missing fixture value")?
    );
    assert_eq!(
        fs::read_to_string(fixture.workspace.join("sentinel.txt"))?,
        "unset"
    );
    let args = fs::read_to_string(fixture.workspace.join("arguments.txt"))?;
    for argument in [
        "--ignore-user-config",
        "--ephemeral",
        "explicit-model",
        "sandbox_workspace_write.network_access=false",
        "model_reasoning_effort=\"low\"",
    ] {
        assert!(args.lines().any(|line| line == argument));
    }
    Ok(())
}

#[test]
fn malformed_provider_output_is_protocol_invalid() -> Result<(), Box<dyn std::error::Error>> {
    let (_directory, codex) = provider(
        "/bin/cat > /dev/null\nprintf not-json",
        Authentication::default(),
    )?;
    let fixture = Fixture::prepare(
        &BTreeMap::new(),
        "instructions",
        "skill",
        Path::new("/tmp/arnes"),
    )?;
    let prompt = "synthetic".repeat(131_072);
    let result = codex.execute(&fixture, &prompt, &options())?;
    assert_eq!(result.error, Some(ExecutionError::ProtocolInvalid));
    assert_eq!(result.tokens, None);
    assert_eq!(result.tool_calls, None);
    Ok(())
}

#[test]
fn installs_saved_authentication_in_fixture_and_surfaces_missing_auth_file()
-> Result<(), Box<dyn std::error::Error>> {
    let auth = tempfile::NamedTempFile::new()?;
    fs::write(auth.path(), "synthetic-auth")?;
    let (_directory, codex) = provider(
        "/bin/cat \"$CODEX_HOME/auth.json\" > auth.txt\nprintf not-json",
        Authentication {
            file: Some(auth.path().into()),
            api_key: None,
        },
    )?;
    let fixture = Fixture::prepare(
        &BTreeMap::new(),
        "instructions",
        "skill",
        Path::new("/tmp/arnes"),
    )?;
    codex.execute(&fixture, "", &options())?;
    assert_eq!(
        fs::read_to_string(fixture.workspace.join("auth.txt"))?,
        "synthetic-auth"
    );
    drop(auth);
    assert!(codex.execute(&fixture, "", &options()).is_err());
    Ok(())
}

#[test]
fn api_key_authentication_only_reaches_explicit_provider_environment()
-> Result<(), Box<dyn std::error::Error>> {
    let (_directory, codex) = provider(
        "printf '%s' \"$CODEX_API_KEY\" > key.txt\nprintf not-json",
        Authentication {
            api_key: Some("synthetic-key".into()),
            file: None,
        },
    )?;
    let fixture = Fixture::prepare(
        &BTreeMap::new(),
        "instructions",
        "skill",
        Path::new("/tmp/arnes"),
    )?;
    codex.execute(&fixture, "", &options())?;
    assert_eq!(
        fs::read_to_string(fixture.workspace.join("key.txt"))?,
        "synthetic-key"
    );
    assert!(!fixture.env.contains_key("CODEX_API_KEY"));
    Ok(())
}
