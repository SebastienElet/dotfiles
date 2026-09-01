use super::{Fixture, event, run, run_event};
use std::fs::{self, File};
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
fn isolated_invalid_utf8_in_a_transcript_is_a_json_error() {
    let fixture = Fixture::new();
    fs::write(&fixture.transcript, [0xff]).unwrap();

    let output = run_event(&fixture, "isolated-invalid-utf8");

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    assert_eq!(
        output.stderr,
        b"agent-handoff: malformed transcript JSON at retained line 1\n"
    );
    assert!(!fixture.sentinel("isolated-invalid-utf8").exists());
}

#[test]
fn invalid_utf8_inside_ignored_json_text_preserves_a_blocking_record() {
    let fixture = Fixture::new();
    let mut transcript = br#"{"metadata":""#.to_vec();
    transcript.push(0xff);
    transcript.extend_from_slice(
        br#"","type":"assistant","isSidechain":false,"message":{"usage":{"cache_creation_input_tokens":0,"cache_read_input_tokens":0,"input_tokens":90000}}}
"#,
    );
    fs::write(&fixture.transcript, transcript).unwrap();

    let output = run_event(&fixture, "embedded-invalid-utf8");

    assert_high_usage_block(&output);
    assert!(fixture.sentinel("embedded-invalid-utf8").is_file());
}

#[test]
fn xdg_state_root_lexically_removes_a_non_directory_parent_component() {
    let fixture = Fixture::new();
    fixture.write_claude_usage(90_000);
    fs::create_dir_all(&fixture.state).unwrap();
    File::create(fixture.state.join("file")).unwrap();
    let xdg_state_home = format!("{}/file/..", fixture.state.display());
    let mut command = fixture.command();
    command.env("XDG_STATE_HOME", xdg_state_home);

    let output = run(
        command,
        &event(&fixture.transcript, "non-directory-parent", false),
    );

    assert_high_usage_block(&output);
    assert!(fixture.sentinel("non-directory-parent").is_file());
}

#[test]
fn xdg_state_root_lexically_removes_a_symlink_parent_component() {
    let fixture = Fixture::new();
    fixture.write_claude_usage(90_000);
    fs::create_dir_all(&fixture.state).unwrap();
    let target = fixture._root.path().join("target");
    fs::create_dir(&target).unwrap();
    symlink(&target, fixture.state.join("alias")).unwrap();
    let xdg_state_home = format!("{}/alias/..", fixture.state.display());
    let mut command = fixture.command();
    command.env("XDG_STATE_HOME", xdg_state_home);

    let output = run(
        command,
        &event(&fixture.transcript, "symlink-parent", false),
    );

    assert_high_usage_block(&output);
    assert!(fixture.sentinel("symlink-parent").is_file());
    assert!(
        !fixture
            ._root
            .path()
            .join("dotfiles/handoff/symlink-parent")
            .exists()
    );
}
