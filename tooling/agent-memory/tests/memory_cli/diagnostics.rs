use super::support::*;
use serde_json::{Value, json};
fn rejection(
    output: &std::process::Output,
    code: &str,
    field: &str,
) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
    assert_error(output, 2, code)?;
    let error: Value = serde_json::from_slice(&output.stderr)?;
    let error = (*(error)
        .get("error")
        .ok_or(concat!("missing fixture index ", stringify!("error")))?)
    .clone();
    assert_eq!(
        (*(error)
            .get("field")
            .ok_or(concat!("missing fixture index ", stringify!("field")))?),
        field
    );
    assert!(
        (*(error)
            .get("message")
            .ok_or(concat!("missing fixture index ", stringify!("message")))?)
        .as_str()
        .is_some_and(|text| text.len() > 20)
    );
    Ok(error)
}
#[test]
fn statement_rejection_supplies_the_boundary_for_a_single_repair()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = CliFixture::new()?;
    let draft = CliFixture::git_draft("invariant", &"é".repeat(558), "diagnostic")?;
    let error = rejection(
        &fixture.run(["admit", "--format", "json"], &draft)?,
        "invalid_field",
        "statement",
    )?;
    assert_eq!(
        (*(error)
            .get("minimum")
            .ok_or(concat!("missing fixture index ", stringify!("minimum")))?),
        1
    );
    assert_eq!(
        (*(error)
            .get("maximum")
            .ok_or(concat!("missing fixture index ", stringify!("maximum")))?),
        500
    );
    assert_eq!(
        (*(error)
            .get("unit")
            .ok_or(concat!("missing fixture index ", stringify!("unit")))?),
        "unicode_scalars"
    );
    assert!(!fixture.root().exists());
    let repaired = CliFixture::git_draft(
        "invariant",
        &"é".repeat(usize::try_from(
            (*(error)
                .get("maximum")
                .ok_or(concat!("missing fixture index ", stringify!("maximum")))?)
            .as_u64()
            .ok_or("expected unsigned JSON integer")?,
        )?),
        "diagnostic",
    )?;
    assert_exit(&fixture.run(["admit", "--format", "json"], &repaired)?, 0);
    Ok(())
}
#[test]
fn nested_schema_errors_name_a_safe_field_and_expected_shape()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = CliFixture::new()?;
    let draft = CliFixture::git_draft("invariant", "Diagnostic memory.", "diagnostic")?;
    let mut valid: Value = serde_yaml_ng::from_slice(&draft)?;
    {
        valid
            .as_object_mut()
            .ok_or("expected JSON object for fixture update")?
            .insert(("scope").to_owned(), json!("project"));
    };
    let cases = [
        ("/statement", json!([]), "statement", "string"),
        ("/scope", json!("private-canary"), "scope", "project"),
        ("/kind", json!("private-canary"), "kind", "invariant"),
        (
            "/proof/summary",
            json ! ({ "private-canary" : "secret" }),
            "proof.summary",
            "string",
        ),
        (
            "/oracle/human_fallback/question",
            json!([]),
            "oracle.human_fallback.question",
            "string",
        ),
        (
            "/oracle/automated/expected",
            json!("private-canary"),
            "oracle.automated.expected",
            "all-proof-sources-unchanged",
        ),
        (
            "/proof/sources/0/locator",
            json!([]),
            "proof.sources.locator",
            "string",
        ),
    ];
    let _: () = for (pointer, value, field, expected) in cases {
        let mut draft = valid.clone();
        *draft.pointer_mut(pointer).ok_or("expected JSON pointer")? = value;
        let output = fixture.run(["admit", "--format", "json"], &serde_json::to_vec(&draft)?)?;
        let error = rejection(&output, "invalid_field", field)?;
        assert!(
            (*(error)
                .get("message")
                .ok_or(concat!("missing fixture index ", stringify!("message")))?)
            .as_str()
            .ok_or("expected JSON string")?
            .contains(expected),
            "{error}"
        );
        assert!(!String::from_utf8_lossy(&output.stderr).contains("private-canary"));
    };
    Ok(())
}
#[test]
fn admission_rejection_classes_explain_the_required_correction()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = CliFixture::new()?;
    let draft = CliFixture::git_draft("invariant", "Diagnostic memory.", "diagnostic")?;
    let mut valid: Value = serde_yaml_ng::from_slice(&draft)?;
    {
        valid
            .as_object_mut()
            .ok_or("expected JSON object for fixture update")?
            .insert(("scope").to_owned(), json!("project"));
    };
    let cases = [
        (
            "/schema_version",
            json!(2),
            "unsupported_schema",
            "schema_version",
            "1",
        ),
        (
            "/proof/sources",
            json!([]),
            "missing_proof",
            "proof.sources",
            "source",
        ),
        (
            "/oracle/automated",
            Value::Null,
            "missing_oracle",
            "oracle.automated",
            "source-fingerprint",
        ),
        (
            "/proof/sources/0/kind",
            json!("private-canary"),
            "invalid_source_kind",
            "proof.sources.kind",
            "git-file",
        ),
        (
            "/statement",
            json!("secret=private-canary"),
            "sensitive_content",
            "statement",
            "secret",
        ),
        (
            "/retrieval_terms",
            json!(vec!["term"; 21]),
            "too_many_items",
            "retrieval_terms",
            "items",
        ),
    ];
    let _: () = for (pointer, value, code, field, expected) in cases {
        let mut draft = valid.clone();
        *draft.pointer_mut(pointer).ok_or("expected JSON pointer")? = value;
        let output = fixture.run(["admit", "--format", "json"], &serde_json::to_vec(&draft)?)?;
        let error = rejection(&output, code, field)?;
        assert!(
            (*(error)
                .get("message")
                .ok_or(concat!("missing fixture index ", stringify!("message")))?)
            .as_str()
            .ok_or("expected JSON string")?
            .contains(expected),
            "{error}"
        );
        if code == "too_many_items" {
            assert_eq!(
                (*(error)
                    .get("maximum")
                    .ok_or(concat!("missing fixture index ", stringify!("maximum")))?),
                20
            );
        }
        assert!(!String::from_utf8_lossy(&output.stderr).contains("private-canary"));
    };
    Ok(())
}
#[test]
fn yaml_errors_provide_locations_without_echoing_keys_or_values()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = CliFixture::new()?;
    let _: () = for (input, code) in [
        ("private-canary: [broken", "malformed_yaml"),
        ("private-canary: 1\nprivate-canary: 2\n", "duplicate_field"),
    ] {
        let output = fixture.run(["admit", "--format", "json"], input.as_bytes())?;
        let error = rejection(&output, code, "document")?;
        assert!(
            (*(error)
                .get("line")
                .ok_or(concat!("missing fixture index ", stringify!("line")))?)
            .as_u64()
            .is_some_and(|line| line > 0)
        );
        assert!(!String::from_utf8_lossy(&output.stderr).contains("private-canary"));
    };
    Ok(())
}
#[test]
fn retrieve_confirm_and_hook_explain_rejections_without_echoing_input()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = CliFixture::new()?;
    rejection(
        &fixture.run(["retrieve", "--query-stdin", "--format", "json"], b" \n")?,
        "empty_query",
        "query",
    )?;
    rejection(
        &fixture.run(["retrieve", "--query-stdin", "--format", "json"], &[255])?,
        "invalid_utf8",
        "query",
    )?;
    let error = rejection(
        &fixture.run(
            [
                "confirm",
                "--id",
                "private-canary",
                "--status",
                "confirmed",
                "--reason-stdin",
            ],
            "é".repeat(501).as_bytes(),
        )?,
        "invalid_field",
        "transition.reason",
    )?;
    assert_eq!(
        (*(error)
            .get("maximum")
            .ok_or(concat!("missing fixture index ", stringify!("maximum")))?),
        500
    );
    for agent in ["claude", "codex"] {
        let payload = json ! ({ "hook_event_name" : "UserPromptSubmit" , "prompt" : "private-canary" , "cwd" : "relative" });
        let output = fixture.run(["hook", "--agent", agent], &serde_json::to_vec(&payload)?)?;
        let error = rejection(&output, "invalid_hook_cwd", "cwd")?;
        assert!(
            (*(error)
                .get("message")
                .ok_or(concat!("missing fixture index ", stringify!("message")))?)
            .as_str()
            .ok_or("expected JSON string")?
            .contains("absolute")
        );
        assert!(!String::from_utf8_lossy(&output.stderr).contains("private-canary"));
    }
    let error = rejection(
        &fixture.run(["admit", "--format", "json"], &vec![b'a'; 1_048_577])?,
        "input_too_large",
        "stdin",
    )?;
    assert_eq!(
        (*(error)
            .get("maximum")
            .ok_or(concat!("missing fixture index ", stringify!("maximum")))?),
        1_048_576
    );
    assert_eq!(
        (*(error)
            .get("unit")
            .ok_or(concat!("missing fixture index ", stringify!("unit")))?),
        "bytes"
    );
    Ok(())
}
