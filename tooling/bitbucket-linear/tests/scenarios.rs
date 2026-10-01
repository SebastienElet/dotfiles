use crate::support::json::{at, at_mut, put};
use crate::support::{Fixture, ISSUE, TEAM, output_value, pull_request, store};
use serde_json::{Value, json};
use std::{
    error::Error,
    fs,
    process::{Command, Stdio},
};

#[test]
fn inspection_has_complete_plan_and_no_mutations() -> Result<(), Box<dyn Error>> {
    let fixture = Fixture::new(&store())?;
    let before = fixture.read()?;
    let output = fixture.run(false)?;
    assert!(output.status.success());
    let (plan, result) = output_value(&output)?;
    assert_eq!(at(&plan, "/phase")?, "plan");
    assert_eq!(at(&result, "/mode")?, "inspect");
    assert_eq!(at(&plan, "/issues/0/identifier")?, "ENG-1");
    assert!(!at(&plan, "/issues/0/completeStateId")?.is_null());
    assert_eq!(fixture.read()?, before);
    assert!(!fixture.calls()?.contains("mutation"));
    Ok(())
}

#[test]
fn apply_links_then_completes_and_replays_without_writes() -> Result<(), Box<dyn Error>> {
    let fixture = Fixture::new(&store())?;
    let output = fixture.run(true)?;
    assert!(output.status.success());
    let (_, result) = output_value(&output)?;
    assert_eq!(at(&result, "/observations/0/status")?, "linked");
    assert_eq!(at(&result, "/observations/1/status")?, "completed");
    let stored = fixture.read()?;
    assert_eq!(at(&stored, "/issues/0/state/type")?, "completed");
    assert_eq!(
        at(&stored, "/issues/0/attachments/nodes")?
            .as_array()
            .ok_or("nodes")?
            .len(),
        1
    );
    let first_calls = fixture.calls()?;
    assert!(
        first_calls.find("mutation Attach").ok_or("attach")?
            < first_calls.find("mutation Complete").ok_or("complete")?
    );
    assert!(fixture.run(true)?.status.success());
    assert_eq!(fixture.read()?, stored);
    assert_eq!(fixture.calls()?.matches("mutation").count(), 2);
    assert_eq!(at(&stored, "/issues/0/id")?, ISSUE);
    Ok(())
}

fn no_completion(value: &Value, expected_code: i32) -> Result<Fixture, Box<dyn Error>> {
    let fixture = Fixture::new(value)?;
    let output = fixture.run(true)?;
    assert_eq!(
        output.status.code(),
        Some(expected_code),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!fixture.calls()?.contains("mutation Complete"));
    assert_ne!(at(&fixture.read()?, "/issues/0/state/type")?, "completed");
    Ok(fixture)
}

#[test]
fn open_or_declined_work_does_not_complete() -> Result<(), Box<dyn Error>> {
    for state in ["OPEN", "DECLINED"] {
        let mut value = store();
        put(json!(state), &mut value, "/pages/0/values/0/state")?;
        no_completion(&value, 0)?;
    }
    Ok(())
}

#[test]
fn one_open_pr_prevents_completion_of_merged_work() -> Result<(), Box<dyn Error>> {
    let mut value = store();
    at_mut(&mut value, "/pages/0/values")?
        .as_array_mut()
        .ok_or("prs")?
        .push(pull_request(2, "ENG-1", "OPEN"));
    no_completion(&value, 0)?;
    Ok(())
}

#[test]
fn canceled_and_completed_issues_are_not_moved() -> Result<(), Box<dyn Error>> {
    for state in ["canceled", "completed"] {
        let mut value = store();
        put(json!(state), &mut value, "/issues/0/state/type")?;
        let fixture = Fixture::new(&value)?;
        assert!(fixture.run(true)?.status.success());
        assert!(!fixture.calls()?.contains("mutation Complete"));
        assert_eq!(at(&fixture.read()?, "/issues/0/state/type")?, state);
    }
    Ok(())
}

#[test]
fn completed_issue_with_open_pr_is_reported_without_state_change() -> Result<(), Box<dyn Error>> {
    let mut value = store();
    put(json!("completed"), &mut value, "/issues/0/state/type")?;
    put(json!("OPEN"), &mut value, "/pages/0/values/0/state")?;
    let fixture = Fixture::new(&value)?;
    let output = fixture.run(true)?;
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8(output.stdout)?.contains("completed-with-open-pr"));
    assert!(!fixture.calls()?.contains("mutation Complete"));
    Ok(())
}

fn attachment(title: &str, state: &str) -> Value {
    json!({"id":"link","url":"https://bitbucket.org/acme/app/pull-requests/1","title":title,"subtitle":state})
}

#[test]
fn conforming_attachment_is_unchanged_and_stale_fields_are_updated() -> Result<(), Box<dyn Error>> {
    for (title, state, operation) in [
        ("ENG-1", "MERGED", "unchanged"),
        ("Old title", "OPEN", "updated"),
    ] {
        let mut value = store();
        put(json!("OPEN"), &mut value, "/pages/0/values/0/state")?;
        let current_state = if operation == "unchanged" {
            "OPEN"
        } else {
            state
        };
        put(
            json!([attachment(title, current_state)]),
            &mut value,
            "/issues/0/attachments/nodes",
        )?;
        let fixture = Fixture::new(&value)?;
        let output = fixture.run(true)?;
        assert!(output.status.success());
        let (_, result) = output_value(&output)?;
        assert_eq!(at(&result, "/observations/0/status")?, operation);
        assert_eq!(
            at(&fixture.read()?, "/issues/0/attachments/nodes")?
                .as_array()
                .ok_or("nodes")?
                .len(),
            1
        );
    }
    Ok(())
}

#[test]
fn duplicate_attachments_and_archived_issue_block_all_its_writes() -> Result<(), Box<dyn Error>> {
    for archived in [false, true] {
        let mut value = store();
        if archived {
            put(
                json!("2026-09-01T00:00:00Z"),
                &mut value,
                "/issues/0/archivedAt",
            )?;
        } else {
            let mut other = attachment("ENG-1", "MERGED");
            put(json!("other"), &mut other, "/id")?;
            put(
                json!([attachment("ENG-1", "MERGED"), other]),
                &mut value,
                "/issues/0/attachments/nodes",
            )?;
        }
        let fixture = no_completion(&value, 1)?;
        assert!(!fixture.calls()?.contains("mutation"));
    }
    Ok(())
}

#[test]
fn absent_or_ambiguous_completed_state_blocks_only_completion() -> Result<(), Box<dyn Error>> {
    for multiple in [false, true] {
        let mut value = store();
        let states = if multiple {
            let mut second = at(&value, "/done")?.clone();
            put(json!(ISSUE), &mut second, "/id")?;
            json!([at(&value, "/done")?, second])
        } else {
            json!([])
        };
        put(states, &mut value, "/teamPages/0/states/nodes")?;
        let fixture = no_completion(&value, 1)?;
        assert!(fixture.calls()?.contains("mutation Attach"));
    }
    Ok(())
}

#[test]
fn explicit_completed_state_resolves_multiple_choices() -> Result<(), Box<dyn Error>> {
    let mut value = store();
    let mut second = at(&value, "/done")?.clone();
    put(json!(ISSUE), &mut second, "/id")?;
    put(
        json!([at(&value, "/done")?, second]),
        &mut value,
        "/teamPages/0/states/nodes",
    )?;
    let fixture = Fixture::new(&value)?;
    let mut configured = crate::support::config();
    put(
        at(&value, "/done/id")?.clone(),
        &mut configured,
        "/repositories/0/completedStateId",
    )?;
    fs::write(
        fixture.dir.path().join("config.json"),
        configured.to_string(),
    )?;
    assert!(fixture.run(true)?.status.success());
    assert_eq!(at(&fixture.read()?, "/issues/0/state/type")?, "completed");
    Ok(())
}

#[test]
fn lexical_boundaries_case_and_all_signals_select_one_issue() -> Result<(), Box<dyn Error>> {
    let mut value = store();
    put(
        json!("xENG-9 ENG-12x eng-1"),
        &mut value,
        "/pages/0/values/0/title",
    )?;
    put(
        json!("ENG-1 _ENG-3"),
        &mut value,
        "/pages/0/values/0/description",
    )?;
    let fixture = Fixture::new(&value)?;
    assert!(fixture.run(true)?.status.success());
    assert_eq!(at(&fixture.read()?, "/issues/0/state/type")?, "completed");
    Ok(())
}

#[test]
fn ambiguous_pr_blocks_issue_also_referenced_by_merged_pr() -> Result<(), Box<dyn Error>> {
    let mut value = store();
    at_mut(&mut value, "/pages/0/values")?
        .as_array_mut()
        .ok_or("prs")?
        .push(pull_request(2, "ENG-1 ENG-2", "OPEN"));
    no_completion(&value, 1)?;
    Ok(())
}

#[test]
fn zero_identifier_is_ignored_without_mutation() -> Result<(), Box<dyn Error>> {
    let mut value = store();
    put(
        json!([pull_request(1, "Routine maintenance", "OPEN")]),
        &mut value,
        "/pages/0/values",
    )?;
    let fixture = Fixture::new(&value)?;
    let output = fixture.run(true)?;
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout)?.contains("no-configured-identifier"));
    assert!(!fixture.calls()?.contains("mutation"));
    Ok(())
}

#[test]
fn wrong_issue_team_is_visible_and_never_mutated() -> Result<(), Box<dyn Error>> {
    let mut value = store();
    put(json!(ISSUE), &mut value, "/issues/0/team/id")?;
    let fixture = no_completion(&value, 1)?;
    assert!(!fixture.calls()?.contains("mutation"));
    Ok(())
}

#[test]
fn incomplete_repository_blocks_same_team_completion_across_repositories()
-> Result<(), Box<dyn Error>> {
    let fixture = Fixture::new(&store())?;
    let mut configured = crate::support::config();
    at_mut(&mut configured, "/repositories")?
        .as_array_mut()
        .ok_or("repos")?
        .push(json!({"workspace":"acme","repository":"other","teamKey":"ENG","teamId":TEAM}));
    fs::write(
        fixture.dir.path().join("config.json"),
        configured.to_string(),
    )?;
    let output = fixture.run(true)?;
    assert_eq!(output.status.code(), Some(1));
    assert!(!fixture.calls()?.contains("mutation Complete"));
    Ok(())
}

#[test]
fn partial_failure_keeps_independent_progress_and_replay_repairs_failure()
-> Result<(), Box<dyn Error>> {
    let mut value = store();
    at_mut(&mut value, "/pages/0/values")?
        .as_array_mut()
        .ok_or("prs")?
        .push(pull_request(2, "ENG-2", "MERGED"));
    put(json!(ISSUE), &mut value, "/failMutationIssue")?;
    let fixture = Fixture::new(&value)?;
    assert_eq!(fixture.run(true)?.status.code(), Some(1));
    let mut stored = fixture.read()?;
    assert_eq!(at(&stored, "/issues/0/state/type")?, "unstarted");
    assert_eq!(at(&stored, "/issues/1/state/type")?, "completed");
    put(Value::Null, &mut stored, "/failMutationIssue")?;
    fixture.write(&stored)?;
    assert!(fixture.run(true)?.status.success());
    assert_eq!(at(&fixture.read()?, "/issues/0/state/type")?, "completed");
    Ok(())
}

#[test]
fn failed_plan_output_prevents_every_mutation() -> Result<(), Box<dyn Error>> {
    let fixture = Fixture::new(&store())?;
    let mut child = Command::new(env!("CARGO_BIN_EXE_bitbucket-linear-sync"))
        .args([
            "--config",
            fixture
                .dir
                .path()
                .join("config.json")
                .to_str()
                .ok_or("path")?,
            "--apply",
            "--json",
        ])
        .env("PATH", fixture.dir.path())
        .env("FIXTURE_ROOT", fixture.dir.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    drop(child.stdout.take());
    assert_eq!(child.wait()?.code(), Some(1));
    assert!(!fixture.calls()?.contains("mutation"));
    Ok(())
}

#[test]
fn attachment_removed_after_plan_prevents_completion_without_extra_repair()
-> Result<(), Box<dyn Error>> {
    let mut value = store();
    put(
        json!([attachment("ENG-1", "MERGED")]),
        &mut value,
        "/issues/0/attachments/nodes",
    )?;
    put(json!(2), &mut value, "/removeOnRead")?;
    let fixture = no_completion(&value, 1)?;
    assert!(!fixture.calls()?.contains("mutation Attach"));
    Ok(())
}

#[test]
fn attachment_not_persisted_prevents_completion() -> Result<(), Box<dyn Error>> {
    let mut value = store();
    put(json!(true), &mut value, "/loseAttachment")?;
    no_completion(&value, 1)?;
    Ok(())
}

#[test]
fn multiple_issue_and_url_mutations_are_ordered() -> Result<(), Box<dyn Error>> {
    let mut value = store();
    put(
        json!([
            pull_request(3, "ENG-2", "OPEN"),
            pull_request(2, "ENG-1", "OPEN"),
            pull_request(1, "ENG-1", "OPEN")
        ]),
        &mut value,
        "/pages/0/values",
    )?;
    let fixture = Fixture::new(&value)?;
    let output = fixture.run(true)?;
    assert!(output.status.success());
    let (_, result) = output_value(&output)?;
    let events = at(&result, "/observations")?
        .as_array()
        .ok_or("observations")?;
    let urls: Vec<_> = events
        .iter()
        .map(|e| {
            (
                e.get("identifier").and_then(Value::as_str),
                e.get("url").and_then(Value::as_str),
            )
        })
        .collect();
    assert_eq!(
        urls,
        [
            (
                Some("ENG-1"),
                Some("https://bitbucket.org/acme/app/pull-requests/1")
            ),
            (
                Some("ENG-1"),
                Some("https://bitbucket.org/acme/app/pull-requests/2")
            ),
            (
                Some("ENG-2"),
                Some("https://bitbucket.org/acme/app/pull-requests/3")
            )
        ]
    );
    Ok(())
}

#[test]
fn public_plan_never_includes_pull_request_descriptions() -> Result<(), Box<dyn Error>> {
    let mut value = store();
    put(
        json!("DO-NOT-PRINT-DESCRIPTION"),
        &mut value,
        "/pages/0/values/0/description",
    )?;
    let fixture = Fixture::new(&value)?;
    let output = fixture.run(false)?;
    assert!(output.status.success());
    assert!(!String::from_utf8(output.stdout)?.contains("DO-NOT-PRINT-DESCRIPTION"));
    Ok(())
}
