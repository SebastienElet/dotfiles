#![cfg(test)]
#[path = "measure_pr_timeline/retention.rs"]
mod retention;
#[path = "measure_pr_timeline/storage.rs"]
mod storage;
#[path = "measure_pr_timeline/support.rs"]
mod support;
use support::*;
#[test]
fn records_a_compact_verdict_on_the_explicit_pr_head()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    assert_status(&harness.record(&[])?, "recorded");
    let event = harness.events()?.remove(0);
    assert_eq!(
        *(event)
            .get("schema_version")
            .ok_or("missing fixture index schema_version")?,
        1
    );
    assert_eq!(
        *(event)
            .get("event_type")
            .ok_or("missing fixture index event_type")?,
        "pr-verdict"
    );
    assert!(
        (*(event)
            .get("timestamp_ms")
            .ok_or("missing fixture index timestamp_ms")?)
        .as_u64()
        .ok_or("expected JSON unsigned integer")?
            > 0
    );
    assert_eq!(
        *(event).get("pr").ok_or("missing fixture index pr")?,
        json ! ({ "forge" : "github.com" , "repository" : "example/project" , "pr_id" : 1042 })
    );
    assert_eq!(
        *(event)
            .get("head_sha")
            .ok_or("missing fixture index head_sha")?,
        SHA
    );
    assert_eq!(
        *(event).get("agent").ok_or("missing fixture index agent")?,
        "codex"
    );
    assert_eq!(
        *(event)
            .get("operating_system")
            .ok_or("missing fixture index operating_system")?,
        std::env::consts::OS
    );
    assert_eq!(
        *(event)
            .get("architecture")
            .ok_or("missing fixture index architecture")?,
        std::env::consts::ARCH
    );
    assert_eq!(
        *(event).get("data").ok_or("missing fixture index data")?,
        json ! ({ "verdict" : "changes-required" , "blocking_findings" : 2 , "non_blocking_findings" : 1 , "observable_behaviors" : 3 , "evidence_gaps" : 2 })
    );
    assert!(event.to_string().len() < 1000);
    assert!(
        !event.to_string().contains(
            harness
                .root
                .path()
                .to_str()
                .ok_or("required test value is missing")?
        )
    );
    Ok(())
}
#[test]
fn retries_across_agents_do_not_duplicate_the_same_sha()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    assert_status(&harness.record(&[])?, "recorded");
    let original = fs::read(harness.timeline()?)?;
    assert_status(&harness.record(&[("--agent", "cursor")])?, "duplicate");
    assert_eq!(fs::read(harness.timeline()?)?, original);
    assert_eq!(harness.events()?.len(), 1);
    Ok(())
}
#[test]
fn a_different_verdict_for_the_same_key_is_an_observable_conflict()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    assert_status(&harness.record(&[])?, "recorded");
    let original = fs::read(harness.timeline()?)?;
    let output = harness.record(&[("--blocking-findings", "3")])?;
    assert_error(&output, "different verdict already recorded");
    assert_eq!(fs::read(harness.timeline()?)?, original);
    Ok(())
}
#[test]
fn different_heads_append_without_replacing_history()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    assert_status(&harness.record(&[])?, "recorded");
    let original = fs::read(harness.timeline()?)?;
    assert_status(&harness.record(&[("--head-sha", OTHER_SHA)])?, "recorded");
    assert!(fs::read(harness.timeline()?)?.starts_with(&original));
    assert_eq!(
        *(*(harness.events()?)
            .get(1)
            .ok_or("missing fixture index 1")?)
        .get("head_sha")
        .ok_or("missing fixture index head_sha")?,
        OTHER_SHA
    );
    Ok(())
}
#[test]
fn forge_repository_and_pr_id_each_partition_the_timeline()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    for overrides in [
        vec![],
        vec![("--forge", "bitbucket.org")],
        vec![("--repository", "example/other")],
        vec![("--pr-id", "1043")],
    ] {
        assert_status(&harness.record(&overrides)?, "recorded");
    }
    assert_eq!(
        fs::read_dir(harness.measure_root().join("pull-requests"))?.count(),
        4
    );
    Ok(())
}
#[test]
fn concurrent_retries_record_exactly_one_event()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let children: Vec<_> = (0..8)
        .map(|_| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
            Ok(harness.command(&[]).spawn()?)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let outputs: Vec<_> = children
        .into_iter()
        .map(
            |child| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                Ok(child.wait_with_output()?)
            },
        )
        .collect::<Result<Vec<_>, _>>()?;
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
    assert_eq!(harness.events()?.len(), 1);
    Ok(())
}
