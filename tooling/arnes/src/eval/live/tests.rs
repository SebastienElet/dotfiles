use super::*;
use std::{collections::BTreeMap, fs, os::unix::fs::PermissionsExt, path::Path};

fn provider(script: &str, authentication: Authentication) -> (tempfile::TempDir, Codex) {
    let directory = tempfile::tempdir().unwrap();
    let command = directory.path().join("provider");
    fs::write(&command, format!("#!/bin/sh\n{script}\n")).unwrap();
    fs::set_permissions(&command, fs::Permissions::from_mode(0o755)).unwrap();
    (
        directory,
        Codex {
            command,
            authentication,
            version: "synthetic".into(),
        },
    )
}

fn options() -> LiveOptions {
    LiveOptions {
        model: "explicit-model".into(),
        reasoning_effort: "low".into(),
        timeout_seconds: 2,
    }
}

#[test]
fn live_protocol_runs_with_exact_stdin_fresh_home_and_explicit_controls() {
    let (_directory, codex) = provider(
        "/bin/cat > request.txt\nprintf '%s\\n' \"$HOME\" > home.txt\nprintf '%s\\n' \"$@\" > arguments.txt\nprintf '%s' \"${PRIVATE_SENTINEL-unset}\" > sentinel.txt\nprintf '%s\\n' '{\"type\":\"turn.completed\",\"usage\":{\"input_tokens\":4,\"cached_input_tokens\":0,\"output_tokens\":2}}'",
        Authentication::default(),
    );
    let fixture = Fixture::prepare(
        &BTreeMap::new(),
        "instructions",
        "skill",
        Path::new("/tmp/arnes"),
    )
    .unwrap();
    let result = codex.execute(&fixture, " é\n\n", &options()).unwrap();
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
    assert!(result.duration_ms.unwrap() >= 0.0);
    assert_eq!(
        fs::read_to_string(fixture.workspace.join("request.txt")).unwrap(),
        " é\n\n"
    );
    assert_eq!(
        fs::read_to_string(fixture.workspace.join("home.txt"))
            .unwrap()
            .trim(),
        fixture.home.to_str().unwrap()
    );
    assert_eq!(
        fs::read_to_string(fixture.workspace.join("sentinel.txt")).unwrap(),
        "unset"
    );
    let args = fs::read_to_string(fixture.workspace.join("arguments.txt")).unwrap();
    for argument in [
        "--ignore-user-config",
        "--ephemeral",
        "explicit-model",
        "sandbox_workspace_write.network_access=false",
        "model_reasoning_effort=\"low\"",
    ] {
        assert!(args.lines().any(|line| line == argument));
    }
}

#[test]
fn malformed_provider_output_is_protocol_invalid() {
    let (_directory, codex) = provider(
        "/bin/cat > /dev/null\nprintf not-json",
        Authentication::default(),
    );
    let fixture = Fixture::prepare(
        &BTreeMap::new(),
        "instructions",
        "skill",
        Path::new("/tmp/arnes"),
    )
    .unwrap();
    let prompt = "synthetic".repeat(131_072);
    let result = codex.execute(&fixture, &prompt, &options()).unwrap();
    assert_eq!(result.error, Some(ExecutionError::ProtocolInvalid));
    assert_eq!(result.tokens, None);
    assert_eq!(result.tool_calls, None);
}

#[test]
fn installs_saved_authentication_in_fixture_and_surfaces_missing_auth_file() {
    let auth = tempfile::NamedTempFile::new().unwrap();
    fs::write(auth.path(), "synthetic-auth").unwrap();
    let (_directory, codex) = provider(
        "/bin/cat \"$CODEX_HOME/auth.json\" > auth.txt\nprintf not-json",
        Authentication {
            file: Some(auth.path().into()),
            api_key: None,
        },
    );
    let fixture = Fixture::prepare(
        &BTreeMap::new(),
        "instructions",
        "skill",
        Path::new("/tmp/arnes"),
    )
    .unwrap();
    codex.execute(&fixture, "", &options()).unwrap();
    assert_eq!(
        fs::read_to_string(fixture.workspace.join("auth.txt")).unwrap(),
        "synthetic-auth"
    );
    drop(auth);
    assert!(codex.execute(&fixture, "", &options()).is_err());
}

#[test]
fn api_key_authentication_only_reaches_explicit_provider_environment() {
    let (_directory, codex) = provider(
        "printf '%s' \"$CODEX_API_KEY\" > key.txt\nprintf not-json",
        Authentication {
            api_key: Some("synthetic-key".into()),
            file: None,
        },
    );
    let fixture = Fixture::prepare(
        &BTreeMap::new(),
        "instructions",
        "skill",
        Path::new("/tmp/arnes"),
    )
    .unwrap();
    codex.execute(&fixture, "", &options()).unwrap();
    assert_eq!(
        fs::read_to_string(fixture.workspace.join("key.txt")).unwrap(),
        "synthetic-key"
    );
    assert!(!fixture.env.contains_key("CODEX_API_KEY"));
}
