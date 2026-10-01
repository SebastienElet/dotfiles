use crate::support::{config, invoke};
use std::{error::Error, fs, process::Command};

#[test]
fn help_explains_configuration_and_safe_replay() -> Result<(), Box<dyn Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_bitbucket-linear-sync"))
        .arg("--help")
        .output()?;
    assert!(output.status.success());
    assert!(!output.stdout.is_empty());
    Ok(())
}

#[test]
fn invalid_usage_fails_before_providers() -> Result<(), Box<dyn Error>> {
    for args in [
        vec![],
        vec!["--apply"],
        vec!["--unknown"],
        vec!["--config", "a", "--config", "b"],
        vec!["--json", "--json"],
    ] {
        let output = invoke(&args, None)?;
        assert_eq!(output.status.code(), Some(2));
    }
    Ok(())
}

#[test]
fn invalid_config_fails_without_printing_input() -> Result<(), Box<dyn Error>> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("config.json");
    fs::write(&path, r#"{"token":"DO-NOT-PRINT"}"#)?;
    let output = invoke(
        &["--config", path.to_str().ok_or("path encoding")?, "--apply"],
        None,
    )?;
    assert_eq!(output.status.code(), Some(2));
    assert!(!String::from_utf8(output.stderr)?.contains("DO-NOT-PRINT"));
    Ok(())
}

#[test]
fn missing_provider_is_failure_not_empty_inventory() -> Result<(), Box<dyn Error>> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("config.json");
    fs::write(&path, config().to_string())?;
    let output = invoke(
        &["--config", path.to_str().ok_or("path encoding")?, "--json"],
        None,
    )?;
    assert_eq!(output.status.code(), Some(1));
    Ok(())
}

#[test]
fn invalid_argument_diagnostics_do_not_echo_sensitive_values() -> Result<(), Box<dyn Error>> {
    let output = invoke(&["DO-NOT-PRINT-ARGUMENT"], None)?;
    assert_eq!(output.status.code(), Some(2));
    assert!(!String::from_utf8(output.stderr)?.contains("DO-NOT-PRINT-ARGUMENT"));
    Ok(())
}
