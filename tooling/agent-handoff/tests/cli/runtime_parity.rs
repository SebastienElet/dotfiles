use super::TestResult;
use super::{Fixture, event, run, run_event};
use std::ffi::OsString;
use std::fs::{self, File};
use std::os::unix::ffi::OsStringExt;
use std::os::unix::fs::symlink;
use std::process::Output;

fn assert_high_usage_block(output: &Output) {
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        output.stdout,
        b"{\n  \"decision\": \"block\",\n  \"reason\": \"Context is at 90k tokens, past the 85k handoff threshold. Start no new work. Use /handoff to emit the resume prompt for a fresh session, then stop.\"\n}\n"
    );
    assert_eq!(output.stderr, b"");
}

#[test]
fn state_environment_paths_preserve_non_utf8_bytes() -> TestResult {
    for variable in ["XDG_STATE_HOME", "HOME"] {
        let fixture = Fixture::new()?;
        fixture.write_claude_usage(90_000)?;
        let root = fixture
            .root
            .path()
            .join(OsString::from_vec(b"state-\xff".to_vec()));
        let filesystem_accepts_path = fs::create_dir(&root).is_ok();
        let mut command = fixture.command();
        command.env_remove("XDG_STATE_HOME").env(variable, &root);

        let output = run(command, &event(&fixture.transcript, "non-utf8", false))?;

        if filesystem_accepts_path {
            assert_high_usage_block(&output);
            let state = if variable == "HOME" {
                root.join(".local/state")
            } else {
                root
            };
            assert!(state.join("dotfiles/handoff/non-utf8").is_file());
        } else {
            assert_eq!(output.status.code(), Some(3));
            assert!(output.stdout.is_empty());
            assert!(!std::path::Path::new(root.to_string_lossy().as_ref()).exists());
        }
    }
    Ok(())
}

#[test]
fn isolated_invalid_utf8_in_a_transcript_is_a_json_error() -> TestResult {
    let fixture = Fixture::new()?;
    fs::write(&fixture.transcript, [0xff])?;

    let output = run_event(&fixture, "isolated-invalid-utf8")?;

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    assert_eq!(
        output.stderr,
        b"agent-handoff: malformed transcript JSON at retained line 1\n"
    );
    assert!(!fixture.sentinel("isolated-invalid-utf8").exists());
    Ok(())
}

#[test]
fn invalid_utf8_inside_ignored_json_text_preserves_a_blocking_record() -> TestResult {
    let fixture = Fixture::new()?;
    let mut transcript = br#"{"metadata":""#.to_vec();
    transcript.push(0xff);
    transcript.extend_from_slice(
        br#"","type":"assistant","isSidechain":false,"message":{"usage":{"cache_creation_input_tokens":0,"cache_read_input_tokens":0,"input_tokens":90000}}}
"#,
    );
    fs::write(&fixture.transcript, transcript)?;

    let output = run_event(&fixture, "embedded-invalid-utf8")?;

    assert_high_usage_block(&output);
    assert!(fixture.sentinel("embedded-invalid-utf8").is_file());
    Ok(())
}

#[test]
fn xdg_state_root_lexically_removes_a_non_directory_parent_component() -> TestResult {
    let fixture = Fixture::new()?;
    fixture.write_claude_usage(90_000)?;
    fs::create_dir_all(&fixture.state)?;
    File::create(fixture.state.join("file"))?;
    let xdg_state_home = format!("{}/file/..", fixture.state.display());
    let mut command = fixture.command();
    command.env("XDG_STATE_HOME", xdg_state_home);

    let output = run(
        command,
        &event(&fixture.transcript, "non-directory-parent", false),
    )?;

    assert_high_usage_block(&output);
    assert!(fixture.sentinel("non-directory-parent").is_file());
    Ok(())
}

#[test]
fn xdg_state_root_lexically_removes_a_symlink_parent_component() -> TestResult {
    let fixture = Fixture::new()?;
    fixture.write_claude_usage(90_000)?;
    fs::create_dir_all(&fixture.state)?;
    let target = fixture.root.path().join("target");
    fs::create_dir(&target)?;
    symlink(&target, fixture.state.join("alias"))?;
    let xdg_state_home = format!("{}/alias/..", fixture.state.display());
    let mut command = fixture.command();
    command.env("XDG_STATE_HOME", xdg_state_home);

    let output = run(
        command,
        &event(&fixture.transcript, "symlink-parent", false),
    )?;

    assert_high_usage_block(&output);
    assert!(fixture.sentinel("symlink-parent").is_file());
    assert!(
        !fixture
            .root
            .path()
            .join("dotfiles/handoff/symlink-parent")
            .exists()
    );
    Ok(())
}
