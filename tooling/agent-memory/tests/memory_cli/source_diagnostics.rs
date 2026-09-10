use super::support::*;
use serde_json::{Value, json};
#[test]
fn source_rejections_identify_the_criterion_and_source_without_its_locator()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = CliFixture::new()?;
    let draft = CliFixture::git_draft("invariant", "Source diagnostics.", "source diagnostics")?;
    let valid: Value = serde_yaml_ng::from_slice(&draft)?;
    std::fs::write(fixture.repository().join("private-canary"), "proof")?;
    let cases = [
        ("git-file", "../private-canary", "relative"),
        ("git-file", "missing-private-canary", "does not exist"),
        ("git-file", "private-canary", "tracked"),
        ("local-file", "private-canary", "absolute"),
        ("official-url", "http://private-canary.example", "HTTPS"),
    ];
    let _: () = for (kind, locator, expected) in cases {
        let mut draft = valid.clone();
        {
            (* (draft) . get_mut ("proof") . ok_or (concat ! ("missing fixture index " , stringify ! ("proof"))) ?) . as_object_mut () . ok_or ("expected JSON object for fixture update") ? . insert (("sources") . to_owned () , json ! ([{ "kind" : "user-decision" , "locator" : "The user approved the primary domain." } , { "kind" : kind , "locator" : locator } ,])) ;
        };
        let output = fixture.run(["admit", "--format", "json"], &serde_json::to_vec(&draft)?)?;
        assert_error(&output, 2, "source_invalid")?;
        let error: Value = serde_json::from_slice(&output.stderr)?;
        assert_eq!(
            (*(*(error)
                .get("error")
                .ok_or(concat!("missing fixture index ", stringify!("error")))?)
            .get("item_index")
            .ok_or(concat!("missing fixture index ", stringify!("item_index")))?),
            1
        );
        assert!(
            (*(*(error)
                .get("error")
                .ok_or(concat!("missing fixture index ", stringify!("error")))?)
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
fn official_url_requires_an_explicit_user_decision_before_fetching()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = CliFixture::new()?;
    let draft = CliFixture::git_draft("invariant", "Source diagnostics.", "source diagnostics")?;
    let mut draft: Value = serde_yaml_ng::from_slice(&draft)?;
    (*(*(*(draft)
        .get_mut("proof")
        .ok_or(concat!("missing fixture index ", stringify!("proof")))?)
    .get_mut("sources")
    .ok_or(concat!("missing fixture index ", stringify!("sources")))?)
    .get_mut(0)
    .ok_or(concat!("missing fixture index ", stringify!(0)))?) =
        json ! ({ "kind" : "official-url" , "locator" : "https://unused.invalid" });
    let output = fixture.run(["admit", "--format", "json"], &serde_json::to_vec(&draft)?)?;
    assert_error(&output, 2, "source_invalid")?;
    let error: Value = serde_json::from_slice(&output.stderr)?;
    assert!(
        (*(*(error)
            .get("error")
            .ok_or(concat!("missing fixture index ", stringify!("error")))?)
        .get("message")
        .ok_or(concat!("missing fixture index ", stringify!("message")))?)
        .as_str()
        .ok_or("expected JSON string")?
        .contains("user-decision")
    );
    Ok(())
}
#[test]
fn oversized_source_keeps_unavailable_exit_and_names_the_size_limit()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = CliFixture::new()?;
    std::fs::write(
        fixture.repository().join("proof.txt"),
        vec![b'a'; 1_048_577],
    )?;
    let draft = CliFixture::git_draft("invariant", "Source diagnostics.", "source diagnostics")?;
    let output = fixture.run(["admit", "--format", "json"], &draft)?;
    assert_error(&output, 4, "source_unavailable")?;
    let error: Value = serde_json::from_slice(&output.stderr)?;
    assert!(
        (*(*(error)
            .get("error")
            .ok_or(concat!("missing fixture index ", stringify!("error")))?)
        .get("message")
        .ok_or(concat!("missing fixture index ", stringify!("message")))?)
        .as_str()
        .ok_or("expected JSON string")?
        .contains("1048576")
    );
    Ok(())
}
#[test]
fn incompatible_confirmation_names_the_actual_kinds_allowed_statuses()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = CliFixture::new()?;
    let draft = CliFixture::git_draft("decision", "Source diagnostics.", "source diagnostics")?;
    let stored = fixture.run(["admit", "--format", "json"], &draft)?;
    let stored = stdout_json(&stored)?;
    let id = (*(stored)
        .get("id")
        .ok_or(concat!("missing fixture index ", stringify!("id")))?)
    .as_str()
    .ok_or("expected JSON string")?;
    let output = fixture.run(
        [
            "confirm",
            "--id",
            id,
            "--status",
            "achieved",
            "--reason-stdin",
        ],
        b"A human concluded the decision.",
    )?;
    assert_error(&output, 2, "invalid_human_conclusion")?;
    let error: Value = serde_json::from_slice(&output.stderr)?;
    assert_eq!(
        (*(*(error)
            .get("error")
            .ok_or(concat!("missing fixture index ", stringify!("error")))?)
        .get("message")
        .ok_or(concat!("missing fixture index ", stringify!("message")))?),
        "For a decision, the only human terminal status is superseded."
    );
    assert_exit(
        &fixture.run(
            [
                "confirm",
                "--id",
                id,
                "--status",
                "superseded",
                "--reason-stdin",
            ],
            b"A human concluded the decision.",
        )?,
        0,
    );
    Ok(())
}
