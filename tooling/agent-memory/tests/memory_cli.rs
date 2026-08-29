#[path = "memory_cli/admission_order.rs"]
mod admission_order;
#[path = "memory_cli/argument_redaction.rs"]
mod argument_redaction;
#[path = "memory_cli/canonical_reads.rs"]
mod canonical_reads;
#[path = "memory_cli/error_classes.rs"]
mod error_classes;
#[path = "memory_cli/support.rs"]
mod support;

use serde_json::Value;
use support::*;

#[test]
fn admit_returns_stored_then_duplicate_as_json() {
    let fixture = CliFixture::new();
    let draft = fixture.git_draft("invariant", "Durable project memory.", "durable project");

    let stored = fixture.run(["admit", "--format", "json"], &draft);
    assert_exit(&stored, 0);
    let stored_json = stdout_json(&stored);
    assert_eq!(stored_json["status"], "stored");
    assert_eq!(stored_json["index_rebuild_required"], false);

    let duplicate = fixture.run(["admit", "--format", "json"], &draft);
    assert_exit(&duplicate, 0);
    let duplicate_json = stdout_json(&duplicate);
    assert_eq!(duplicate_json["status"], "duplicate");
    assert_eq!(duplicate_json["id"], stored_json["id"]);
    assert!(duplicate.stderr.is_empty());
}

#[test]
fn retrieve_injects_only_relevant_fresh_memory() {
    let fixture = CliFixture::new();
    let draft = fixture.git_draft(
        "invariant",
        "Durable retrieval architecture.",
        "durable retrieval",
    );
    assert_exit(&fixture.run(["admit", "--format", "json"], &draft), 0);

    let relevant = fixture.run(
        ["retrieve", "--query-stdin", "--format", "json"],
        b"Apply durable retrieval architecture",
    );
    assert_exit(&relevant, 0);
    let relevant_json = stdout_json(&relevant);
    assert_eq!(
        relevant_json["injected"][0]["statement"],
        "Durable retrieval architecture."
    );
    assert_eq!(
        relevant_json["injected"][0]["sources"][0]["locator"],
        "proof.txt"
    );

    let unrelated = fixture.run(
        ["retrieve", "--query-stdin", "--format", "json"],
        b"unrelated request",
    );
    assert_exit(&unrelated, 0);
    assert_eq!(
        stdout_json(&unrelated)["injected"],
        Value::Array(Vec::new())
    );
}

#[test]
fn confirm_and_audit_include_a_terminal_entry_without_reactivation() {
    let fixture = CliFixture::new();
    let draft = fixture.git_draft(
        "assumption",
        "The durable assumption is confirmed.",
        "durable assumption",
    );
    let admitted = fixture.run(["admit", "--format", "json"], &draft);
    let id = stdout_json(&admitted)["id"].as_str().unwrap().to_owned();

    let confirmed = fixture.run(
        [
            "confirm",
            "--id",
            &id,
            "--status",
            "confirmed",
            "--reason-stdin",
        ],
        b"The proof is now conclusive.",
    );
    assert_exit(&confirmed, 0);
    assert_eq!(stdout_json(&confirmed)["status"], "confirmed");

    let audit = fixture.run(["audit", "--include-terminal", "--format", "json"], b"");
    assert_exit(&audit, 0);
    let audit_json = stdout_json(&audit);
    assert_eq!(audit_json["entries"][0]["id"], id);
    assert_eq!(audit_json["entries"][0]["status"], "confirmed");

    let active_only = fixture.run(["audit", "--format", "json"], b"");
    assert_exit(&active_only, 0);
    assert_eq!(
        stdout_json(&active_only)["entries"],
        Value::Array(Vec::new())
    );
}

#[test]
fn confirm_rejects_a_shell_command_without_mutating_the_entry() {
    let fixture = CliFixture::new();
    let draft = fixture.git_draft(
        "assumption",
        "The durable assumption remains active.",
        "durable assumption",
    );
    let admitted = fixture.run(["admit", "--format", "json"], &draft);
    let id = stdout_json(&admitted)["id"].as_str().unwrap().to_owned();
    let before = tree_snapshot(fixture.root());

    let output = fixture.run(
        [
            "confirm",
            "--id",
            &id,
            "--status",
            "confirmed",
            "--reason-stdin",
        ],
        b"$ rm -f unsafe",
    );

    assert_error(&output, 2, "shell_command");
    assert_eq!(tree_snapshot(fixture.root()), before);
}

#[test]
fn rejects_empty_and_oversized_stdin() {
    let fixture = CliFixture::new();
    let empty = fixture.run(["admit", "--format", "json"], b"");
    assert_error(&empty, 2, "empty_stdin");

    let oversized = vec![b'a'; 1024 * 1024 + 1];
    let too_large = fixture.run(["admit", "--format", "json"], &oversized);
    assert_error(&too_large, 2, "input_too_large");

    let empty_query = fixture.run(["retrieve", "--query-stdin", "--format", "json"], b"");
    assert_error(&empty_query, 2, "empty_stdin");
}

#[test]
fn unknown_options_are_usage_errors() {
    let fixture = CliFixture::new();
    let output = fixture.run(["admit", "--unknown"], b"");

    assert_error(&output, 2, "invalid_arguments");
}

#[test]
fn rejected_sensitive_content_is_never_repeated() {
    let fixture = CliFixture::new();
    let secret = "ghp_abcdefghijklmnopqrstuvwxyz1234567890";
    let draft = fixture.git_draft("invariant", secret, "durable secret");

    let output = fixture.run(["admit", "--format", "json"], &draft);

    assert_error(&output, 2, "sensitive_content");
    assert!(!String::from_utf8_lossy(&output.stderr).contains(secret));
    assert!(output.stdout.is_empty());
}

#[test]
fn divergent_identity_is_a_conflict() {
    let fixture = CliFixture::new();
    let first = fixture.git_draft("invariant", "Stable identity.", "first term");
    let second = fixture.git_draft("invariant", "Stable identity.", "second term");
    assert_exit(&fixture.run(["admit", "--format", "json"], &first), 0);

    let output = fixture.run(["admit", "--format", "json"], &second);

    assert_error(&output, 3, "entry_conflict");
}

#[test]
fn unavailable_root_uses_the_unavailability_exit() {
    let fixture = CliFixture::new();
    let output = fixture.run_with_root(["audit", "--format", "json"], b"", "relative-store");

    assert_error(&output, 4, "unsafe_store_path");
}

#[test]
fn hook_is_recognized_and_fails_closed_until_its_adapter_exists() {
    let fixture = CliFixture::new();
    let output = fixture.run(["hook", "--agent", "codex"], b"{}");

    assert_error(&output, 4, "adapter_unavailable");
}
