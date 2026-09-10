use super::support::*;
use serde_json::{Value, json};
#[test]
fn unknown_keys_identify_the_containing_mapping_without_echoing_the_key()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = CliFixture::new()?;
    let draft = CliFixture::git_draft("invariant", "Schema diagnostics.", "schema")?;
    let valid: Value = serde_yaml_ng::from_slice(&draft)?;
    let _: () = for (pointer, field, allowed) in [
        ("/proof", "proof", "summary and sources"),
        ("/proof/sources/0", "proof.sources", "kind and locator"),
        (
            "/oracle/human_fallback",
            "oracle.human_fallback",
            "question and valid_when",
        ),
    ] {
        let mut draft = valid.clone();
        {
            draft
                .pointer_mut(pointer)
                .ok_or("expected JSON pointer")?
                .as_object_mut()
                .ok_or("expected JSON object for fixture update")?
                .insert(("private-canary").to_owned(), json!("private-value"));
        };
        let output = fixture.run(["admit", "--format", "json"], &serde_json::to_vec(&draft)?)?;
        assert_error(&output, 2, "unknown_field")?;
        let error: Value = serde_json::from_slice(&output.stderr)?;
        assert_eq!(
            (*(*(error)
                .get("error")
                .ok_or(concat!("missing fixture index ", stringify!("error")))?)
            .get("field")
            .ok_or(concat!("missing fixture index ", stringify!("field")))?),
            field
        );
        assert!(
            (*(*(error)
                .get("error")
                .ok_or(concat!("missing fixture index ", stringify!("error")))?)
            .get("message")
            .ok_or(concat!("missing fixture index ", stringify!("message")))?)
            .as_str()
            .ok_or("expected JSON string")?
            .contains(allowed)
        );
        assert!(!String::from_utf8_lossy(&output.stderr).contains("private-"));
    };
    Ok(())
}
#[test]
fn numeric_scalars_remain_rejected_in_persisted_text_fields()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = CliFixture::new()?;
    let draft = CliFixture::git_draft("invariant", "Schema diagnostics.", "schema")?;
    let valid: Value = serde_yaml_ng::from_slice(&draft)?;
    let _: () = for pointer in [
        "/statement",
        "/retrieval_terms/0",
        "/proof/summary",
        "/proof/sources/0/locator",
        "/oracle/human_fallback/question",
        "/oracle/outcomes/valid",
    ] {
        for scalar in [json!(123), json!(true), json!(null), json!(1.2)] {
            let mut draft = valid.clone();
            *draft.pointer_mut(pointer).ok_or("expected JSON pointer")? = scalar;
            let output =
                fixture.run(["admit", "--format", "json"], &serde_json::to_vec(&draft)?)?;
            assert_error(&output, 2, "invalid_field")?;
        }
    };
    Ok(())
}
#[test]
fn custom_yaml_tags_are_rejected_with_a_safe_repair()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = CliFixture::new()?;
    let draft = String::from_utf8(CliFixture::git_draft(
        "invariant",
        "Schema diagnostics.",
        "schema",
    )?)?;
    for (from, to, field) in [
        ("statement:", "statement: !private-canary", "statement"),
        ("summary:", "summary: !private-canary", "proof.summary"),
        ("proof:", "proof: !private-canary", "proof"),
    ] {
        let output = fixture.run(
            ["admit", "--format", "json"],
            draft.replace(from, to).as_bytes(),
        )?;
        assert_error(&output, 2, "invalid_field")?;
        let error: Value = serde_json::from_slice(&output.stderr)?;
        assert_eq!(
            (*(*(error)
                .get("error")
                .ok_or(concat!("missing fixture index ", stringify!("error")))?)
            .get("field")
            .ok_or(concat!("missing fixture index ", stringify!("field")))?),
            field
        );
        assert!(
            (*(*(error)
                .get("error")
                .ok_or(concat!("missing fixture index ", stringify!("error")))?)
            .get("message")
            .ok_or(concat!("missing fixture index ", stringify!("message")))?)
            .as_str()
            .ok_or("expected JSON string")?
            .contains("tag")
        );
        assert!(!String::from_utf8_lossy(&output.stderr).contains("private-canary"));
    }
    let output = fixture.run(
        ["admit", "--format", "json"],
        draft
            .replace("kind: invariant", "kind: !custom invariant")
            .as_bytes(),
    )?;
    assert_exit(&output, 0);
    Ok(())
}
#[test]
fn an_accepted_kind_tag_does_not_hide_the_actual_rejected_field()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = CliFixture::new()?;
    let draft = String::from_utf8(CliFixture::git_draft(
        "invariant",
        "Schema diagnostics.",
        "schema",
    )?)?;
    let draft = draft
        .replace("kind: invariant", "kind: !custom invariant")
        .replace("statement: \"Schema diagnostics.\"", "statement: []");
    let output = fixture.run(["admit", "--format", "json"], draft.as_bytes())?;
    assert_error(&output, 2, "invalid_field")?;
    let error: Value = serde_json::from_slice(&output.stderr)?;
    assert_eq!(
        (*(*(error)
            .get("error")
            .ok_or(concat!("missing fixture index ", stringify!("error")))?)
        .get("field")
        .ok_or(concat!("missing fixture index ", stringify!("field")))?),
        "statement"
    );
    Ok(())
}
#[test]
fn missing_nested_fields_identify_the_required_field()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = CliFixture::new()?;
    let draft = CliFixture::git_draft("invariant", "Schema diagnostics.", "schema")?;
    let valid: Value = serde_yaml_ng::from_slice(&draft)?;
    let _: () = for (pointer, key, field) in [
        ("", "statement", "statement"),
        ("/proof", "summary", "proof.summary"),
        ("/proof/sources/0", "locator", "proof.sources.locator"),
        (
            "/oracle/human_fallback",
            "question",
            "oracle.human_fallback.question",
        ),
        ("/oracle/automated", "expected", "oracle.automated.expected"),
    ] {
        let mut draft = valid.clone();
        draft
            .pointer_mut(pointer)
            .ok_or("expected JSON pointer")?
            .as_object_mut()
            .ok_or("expected JSON object")?
            .remove(key);
        let output = fixture.run(["admit", "--format", "json"], &serde_json::to_vec(&draft)?)?;
        assert_error(&output, 2, "invalid_field")?;
        let error: Value = serde_json::from_slice(&output.stderr)?;
        assert_eq!(
            (*(*(error)
                .get("error")
                .ok_or(concat!("missing fixture index ", stringify!("error")))?)
            .get("field")
            .ok_or(concat!("missing fixture index ", stringify!("field")))?),
            field
        );
    };
    Ok(())
}
