#[path = "measure_pr_timeline/retention.rs"]
mod retention;
#[path = "measure_pr_timeline/storage.rs"]
mod storage;
#[path = "measure_pr_timeline/support.rs"]
mod support;

use support::*;

#[test]
fn records_a_compact_verdict_on_the_explicit_pr_head() {
    let harness = Harness::new();
    assert_status(&harness.record(&[]), "recorded");
    let event = harness.events().remove(0);
    assert_eq!(event["schema_version"], 1);
    assert_eq!(event["event_type"], "pr-verdict");
    assert!(event["timestamp_ms"].as_u64().unwrap() > 0);
    assert_eq!(
        event["pr"],
        json!({"forge": "github.com", "repository": "example/project", "pr_id": 1042})
    );
    assert_eq!(event["head_sha"], SHA);
    assert_eq!(event["agent"], "codex");
    assert_eq!(event["operating_system"], std::env::consts::OS);
    assert_eq!(event["architecture"], std::env::consts::ARCH);
    assert_eq!(
        event["data"],
        json!({
            "verdict": "changes-required", "blocking_findings": 2,
            "non_blocking_findings": 1, "observable_behaviors": 3, "evidence_gaps": 2
        })
    );
    assert!(event.to_string().len() < 1000);
    assert!(
        !event
            .to_string()
            .contains(harness.root.path().to_str().unwrap())
    );
}

#[test]
fn retries_across_agents_do_not_duplicate_the_same_sha() {
    let harness = Harness::new();
    assert_status(&harness.record(&[]), "recorded");
    let original = fs::read(harness.timeline()).unwrap();
    assert_status(&harness.record(&[("--agent", "cursor")]), "duplicate");
    assert_eq!(fs::read(harness.timeline()).unwrap(), original);
    assert_eq!(harness.events().len(), 1);
}

#[test]
fn a_different_verdict_for_the_same_key_is_an_observable_conflict() {
    let harness = Harness::new();
    assert_status(&harness.record(&[]), "recorded");
    let original = fs::read(harness.timeline()).unwrap();
    let output = harness.record(&[("--blocking-findings", "3")]);
    assert_error(&output, "different verdict already recorded");
    assert_eq!(fs::read(harness.timeline()).unwrap(), original);
}

#[test]
fn different_heads_append_without_replacing_history() {
    let harness = Harness::new();
    assert_status(&harness.record(&[]), "recorded");
    let original = fs::read(harness.timeline()).unwrap();
    assert_status(&harness.record(&[("--head-sha", OTHER_SHA)]), "recorded");
    assert!(fs::read(harness.timeline()).unwrap().starts_with(&original));
    assert_eq!(harness.events()[1]["head_sha"], OTHER_SHA);
}

#[test]
fn forge_repository_and_pr_id_each_partition_the_timeline() {
    let harness = Harness::new();
    for overrides in [
        vec![],
        vec![("--forge", "bitbucket.org")],
        vec![("--repository", "example/other")],
        vec![("--pr-id", "1043")],
    ] {
        assert_status(&harness.record(&overrides), "recorded");
    }
    assert_eq!(
        fs::read_dir(harness.measure_root().join("pull-requests"))
            .unwrap()
            .count(),
        4
    );
}

#[test]
fn concurrent_retries_record_exactly_one_event() {
    let harness = Harness::new();
    let children: Vec<_> = (0..8)
        .map(|_| harness.command(&[]).spawn().unwrap())
        .collect();
    let outputs: Vec<_> = children
        .into_iter()
        .map(|child| child.wait_with_output().unwrap())
        .collect();
    assert_eq!(
        outputs
            .iter()
            .filter(|output| output.stdout == b"recorded\n")
            .count(),
        1
    );
    assert_eq!(
        outputs
            .iter()
            .filter(|output| output.stdout == b"duplicate\n")
            .count(),
        7
    );
    assert!(outputs.iter().all(|output| output.status.success()));
    assert_eq!(harness.events().len(), 1);
}
