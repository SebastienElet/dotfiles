use crate::support::json::{at, at_mut, put};
use crate::support::{Fixture, ISSUE, TEAM, config, output_value, pull_request, store};
use serde_json::{Value, json};
use std::{error::Error, fs};

fn rejects(value: &Value, reason: &str) -> Result<(), Box<dyn Error>> {
    let fixture = Fixture::new(value)?;
    let output = fixture.run(true)?;
    assert_eq!(output.status.code(), Some(1));
    let text = String::from_utf8(output.stdout)?;
    assert!(text.contains(reason), "missing reason {reason}: {text}");
    assert!(!fixture.calls()?.contains("mutation Complete"));
    Ok(())
}

#[test]
fn every_opaque_page_and_author_is_collected() -> Result<(), Box<dyn Error>> {
    let mut value = store();
    let mut other = pull_request(2, "ENG-1", "MERGED");
    put(
        json!("{00000000-0000-4000-8000-000000000013}"),
        &mut other,
        "/author/uuid",
    )?;
    put(
        json!([
            {"size":2,"page":1,"pagelen":100,"values":[pull_request(1,"ENG-1","MERGED")],"next":"https://api.bitbucket.org/2.0/repositories/acme/app/pullrequests?cursor=opaque"},
            {"size":2,"page":2,"pagelen":100,"values":[other]}
        ]),
        &mut value,
        "/pages",
    )?;
    let fixture = Fixture::new(&value)?;
    let output = fixture.run(false)?;
    assert!(output.status.success());
    let (plan, _) = output_value(&output)?;
    let prs = at(&plan, "/issues/0/pullRequests")?
        .as_array()
        .ok_or("prs")?;
    assert_eq!(prs.len(), 2);
    assert_ne!(
        prs.first()
            .ok_or("first")?
            .get("authorId")
            .ok_or("author")?,
        prs.get(1)
            .ok_or("second")?
            .get("authorId")
            .ok_or("author")?
    );
    assert!(fixture.calls()?.contains("cursor=opaque"));
    Ok(())
}

#[test]
fn malformed_page_or_pr_cannot_become_complete_inventory() -> Result<(), Box<dyn Error>> {
    let mut bad_state = pull_request(1, "ENG-1", "UNKNOWN");
    let mut missing_description = pull_request(1, "ENG-1", "MERGED");
    put(Value::Null, &mut missing_description, "/description")?;
    let mut wrong_destination = pull_request(1, "ENG-1", "MERGED");
    put(
        json!("acme/other"),
        &mut wrong_destination,
        "/destination/repository/full_name",
    )?;
    let mut wrong_url = pull_request(1, "ENG-1", "MERGED");
    put(
        json!("https://bitbucket.org/evil/app/pull-requests/1"),
        &mut wrong_url,
        "/links/html/href",
    )?;
    let mut unknown_author = pull_request(1, "ENG-1", "MERGED");
    put(json!("unknown"), &mut unknown_author, "/author/uuid")?;
    put(json!(""), &mut bad_state, "/description")?;
    for page in [
        json!({"values":null}),
        json!({"values":[],"next":null}),
        json!({"values":[],"next":"https://evil.test/collect"}),
        json!({"values":[bad_state]}),
        json!({"values":[missing_description]}),
        json!({"values":[wrong_destination]}),
        json!({"values":[wrong_url]}),
        json!({"values":[unknown_author]}),
    ] {
        let mut value = store();
        put(json!([page]), &mut value, "/pages")?;
        rejects(&value, "inventory-or-team-failed")?;
    }
    Ok(())
}

#[test]
fn malformed_intermediate_page_refuses_inventory() -> Result<(), Box<dyn Error>> {
    let mut value = store();
    put(
        json!("https://api.bitbucket.org/2.0/repositories/acme/app/pullrequests?cursor=opaque"),
        &mut value,
        "/pages/0/next",
    )?;
    at_mut(&mut value, "/pages")?
        .as_array_mut()
        .ok_or("pages")?
        .push(json!({"values":null}));
    rejects(&value, "inventory-or-team-failed")
}

#[test]
fn truncated_inventory_contradicting_total_refuses_before_writes() -> Result<(), Box<dyn Error>> {
    let mut value = store();
    put(json!(2), &mut value, "/pages/0/size")?;
    for apply in [false, true] {
        let fixture = Fixture::new(&value)?;
        let output = fixture.run(apply)?;
        assert_eq!(output.status.code(), Some(1));
        assert!(String::from_utf8(output.stdout)?.contains("inventory-or-team-failed"));
        assert!(!fixture.calls()?.contains("mutation"));
    }
    Ok(())
}

#[test]
fn optional_pagination_metadata_does_not_accept_null_or_wrong_types() -> Result<(), Box<dyn Error>>
{
    for (field, metadata) in [
        ("size", Value::Null),
        ("size", json!("2")),
        ("size", json!(-1)),
        ("page", json!(0)),
        ("pagelen", json!(0)),
        ("pagelen", json!("100")),
    ] {
        let mut value = store();
        put(metadata, &mut value, &format!("/pages/0/{field}"))?;
        rejects(&value, "inventory-or-team-failed")?;
    }
    Ok(())
}

#[test]
fn cyclic_and_cross_repository_pagination_refuse_inventory() -> Result<(), Box<dyn Error>> {
    for next in [
        "https://api.bitbucket.org/2.0/repositories/acme/other/pullrequests?cursor=one",
        "https://user:secret@api.bitbucket.org/2.0/repositories/acme/app/pullrequests?cursor=one",
        "https://api.bitbucket.org/2.0/repositories/acme/app/pullrequests?cursor=one",
    ] {
        let mut value = store();
        let page = json!({"values":[],"next":next});
        put(json!([page.clone(), page]), &mut value, "/pages")?;
        rejects(&value, "inventory-or-team-failed")?;
    }
    Ok(())
}

#[test]
fn duplicate_pr_identity_refuses_inventory() -> Result<(), Box<dyn Error>> {
    let mut value = store();
    put(
        json!([
            pull_request(1, "ENG-1", "MERGED"),
            pull_request(1, "ENG-1", "MERGED")
        ]),
        &mut value,
        "/pages/0/values",
    )?;
    rejects(&value, "inventory-or-team-failed")
}

#[test]
fn graphql_partial_errors_fail_identity_before_mutation() -> Result<(), Box<dyn Error>> {
    let mut value = store();
    put(json!(true), &mut value, "/graphqlErrors")?;
    let fixture = Fixture::new(&value)?;
    let output = fixture.run(true)?;
    assert_eq!(output.status.code(), Some(1));
    assert!(!fixture.calls()?.contains("mutation"));
    Ok(())
}

#[test]
fn incomplete_or_changed_linear_issue_blocks_mutations() -> Result<(), Box<dyn Error>> {
    for patch in [
        json!({"attachments":{"nodes":[],"pageInfo":{"hasNextPage":true,"endCursor":null}}}),
        json!({"state":{"id":ISSUE,"name":"Unknown","type":"unknown"}}),
        json!({"archivedAt":"not-a-date"}),
    ] {
        let mut value = store();
        for (key, item) in patch.as_object().ok_or("patch")? {
            put(item.clone(), &mut value, &format!("/issues/0/{key}"))?;
        }
        rejects(&value, "issue-or-attachments-incomplete")?;
    }
    Ok(())
}

#[test]
fn required_nullable_fields_cannot_disappear() -> Result<(), Box<dyn Error>> {
    for field in ["archivedAt", "state", "team"] {
        let mut value = store();
        at_mut(&mut value, "/issues/0")?
            .as_object_mut()
            .ok_or("issue")?
            .remove(field);
        rejects(&value, "issue-or-attachments-incomplete")?;
    }
    let mut value = store();
    at_mut(&mut value, "/issues/0/attachments/pageInfo")?
        .as_object_mut()
        .ok_or("pageinfo")?
        .remove("endCursor");
    rejects(&value, "issue-or-attachments-incomplete")
}

#[test]
fn attachment_connection_exhausts_pages_and_rejects_cycles() -> Result<(), Box<dyn Error>> {
    let link = json!({"id":"link","url":"https://bitbucket.org/acme/app/pull-requests/1","title":"ENG-1","subtitle":"MERGED"});
    let mut value = store();
    put(
        json!([
            {"nodes":[],"pageInfo":{"hasNextPage":true,"endCursor":"opaque"}},
            {"nodes":[link],"pageInfo":{"hasNextPage":false,"endCursor":null}}
        ]),
        &mut value,
        "/attachmentPages",
    )?;
    let fixture = Fixture::new(&value)?;
    let output = fixture.run(false)?;
    assert!(output.status.success());
    let (plan, _) = output_value(&output)?;
    assert_eq!(at(&plan, "/issues/0/attachments")?, &json!([]));
    put(
        json!({"hasNextPage":true,"endCursor":"opaque"}),
        &mut value,
        "/attachmentPages/1/pageInfo",
    )?;
    rejects(&value, "issue-or-attachments-incomplete")
}

#[test]
fn team_states_exhaust_pages_and_duplicate_ids_are_rejected() -> Result<(), Box<dyn Error>> {
    let mut value = store();
    let mut second = at(&value, "/teamPages/0")?.clone();
    put(json!([]), &mut second, "/states/nodes")?;
    put(
        json!({"hasNextPage":true,"endCursor":"opaque"}),
        &mut value,
        "/teamPages/0/states/pageInfo",
    )?;
    at_mut(&mut value, "/teamPages")?
        .as_array_mut()
        .ok_or("team pages")?
        .push(second);
    let fixture = Fixture::new(&value)?;
    assert!(fixture.run(false)?.status.success());
    put(
        json!([at(&value, "/done")?]),
        &mut value,
        "/teamPages/1/states/nodes",
    )?;
    rejects(&value, "inventory-or-team-failed")
}

#[test]
fn false_success_null_results_or_wrong_state_refuse_completion() -> Result<(), Box<dyn Error>> {
    for payload in [
        json!({"success":false,"issue":null}),
        json!({"success":true,"issue":null}),
        json!({"success":true,"issue":{"id":ISSUE,"state":{"id":ISSUE,"name":"Todo","type":"unstarted"}}}),
    ] {
        let mut value = store();
        put(
            json!({"mutation Complete":{"data":{"issueUpdate":payload}}}),
            &mut value,
            "/overrides",
        )?;
        let fixture = Fixture::new(&value)?;
        let output = fixture.run(true)?;
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(at(&fixture.read()?, "/issues/0/state/type")?, "unstarted");
    }
    Ok(())
}

#[test]
fn explicit_context_and_team_identity_are_checked() -> Result<(), Box<dyn Error>> {
    for host in ["api.evil.test", "bitbucket.org"] {
        let mut value = store();
        put(json!(host), &mut value, "/contexts/contexts/0/host")?;
        let fixture = Fixture::new(&value)?;
        assert_eq!(fixture.run(true)?.status.code(), Some(1));
        assert!(!fixture.calls()?.contains("mutation"));
    }
    let mut value = store();
    put(json!(ISSUE), &mut value, "/teamPages/0/id")?;
    rejects(&value, "inventory-or-team-failed")
}

#[test]
fn invalid_config_fails_before_any_provider_call() -> Result<(), Box<dyn Error>> {
    for changed in [
        json!({}),
        json!({"bktContext":""}),
        json!({"bktContext":null}),
        json!({"unknown":"secret"}),
        json!({"repositories":[]}),
        json!({"repositories":[{"workspace":"acme","repository":"app","teamKey":"eng","teamId":TEAM}]}),
        json!({"repositories":[{"workspace":"acme","repository":"app","teamKey":"ENG","teamId":"unknown"}]}),
    ] {
        let fixture = Fixture::new(&store())?;
        let configured = if changed.as_object().is_some_and(serde_json::Map::is_empty) {
            changed
        } else {
            let mut configured = config();
            for (key, item) in changed.as_object().ok_or("patch")? {
                put(item.clone(), &mut configured, &format!("/{key}"))?;
            }
            configured
        };
        fs::write(
            fixture.dir.path().join("config.json"),
            configured.to_string(),
        )?;
        assert_eq!(fixture.run(true)?.status.code(), Some(2));
        assert!(!fixture.dir.path().join("calls").exists());
    }
    Ok(())
}

#[test]
fn duplicate_or_conflicting_repository_team_mappings_refuse_config() -> Result<(), Box<dyn Error>> {
    for repository in [
        json!({"workspace":"acme","repository":"app","teamKey":"ENG","teamId":TEAM}),
        json!({"workspace":"acme","repository":"other","teamKey":"ENG","teamId":ISSUE}),
        json!({"workspace":"acme","repository":"other","teamKey":"OPS","teamId":TEAM}),
    ] {
        let fixture = Fixture::new(&store())?;
        let mut configured = config();
        at_mut(&mut configured, "/repositories")?
            .as_array_mut()
            .ok_or("repositories")?
            .push(repository);
        fs::write(
            fixture.dir.path().join("config.json"),
            configured.to_string(),
        )?;
        assert_eq!(fixture.run(true)?.status.code(), Some(2));
        assert!(!fixture.dir.path().join("calls").exists());
    }
    Ok(())
}

#[test]
fn false_null_or_mismatched_attachment_response_refuses_completion() -> Result<(), Box<dyn Error>> {
    let expected = json!({"id":"link","url":"https://bitbucket.org/acme/app/pull-requests/1","title":"ENG-1","subtitle":"MERGED"});
    let mut wrong = expected.clone();
    put(
        json!("https://bitbucket.org/acme/app/pull-requests/99"),
        &mut wrong,
        "/url",
    )?;
    for payload in [
        json!({"success":false,"attachment":expected}),
        json!({"success":true,"attachment":null}),
        json!({"success":true,"attachment":wrong}),
    ] {
        let mut value = store();
        put(
            json!({"mutation Attach":{"data":{"attachmentCreate":payload}}}),
            &mut value,
            "/overrides",
        )?;
        let fixture = Fixture::new(&value)?;
        assert_eq!(fixture.run(true)?.status.code(), Some(1));
        assert!(!fixture.calls()?.contains("mutation Complete"));
    }
    Ok(())
}

#[test]
fn completion_response_without_persisted_transition_is_refused() -> Result<(), Box<dyn Error>> {
    let mut value = store();
    let done = at(&value, "/done")?.clone();
    put(
        json!({"mutation Complete":{"data":{"issueUpdate":{"success":true,"issue":{"id":ISSUE,"state":done}}}}}),
        &mut value,
        "/overrides",
    )?;
    let fixture = Fixture::new(&value)?;
    let output = fixture.run(true)?;
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(at(&fixture.read()?, "/issues/0/state/type")?, "unstarted");
    assert!(String::from_utf8(output.stdout)?.contains("readback-failed"));
    Ok(())
}

#[test]
fn explicit_state_outside_team_completed_states_refuses_completion() -> Result<(), Box<dyn Error>> {
    let fixture = Fixture::new(&store())?;
    let mut configured = config();
    put(
        json!(ISSUE),
        &mut configured,
        "/repositories/0/completedStateId",
    )?;
    fs::write(
        fixture.dir.path().join("config.json"),
        configured.to_string(),
    )?;
    assert_eq!(fixture.run(true)?.status.code(), Some(1));
    assert!(!fixture.calls()?.contains("mutation Complete"));
    Ok(())
}
