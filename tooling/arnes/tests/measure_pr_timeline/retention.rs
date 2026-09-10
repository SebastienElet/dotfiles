use super::support::*;
use std::io::Write;
use std::os::unix::fs::symlink;
use std::time::{SystemTime, UNIX_EPOCH};
const DAY_MS: u64 = 86_400_000;
#[test]
fn expires_the_whole_pr_timeline_at_90_days() -> Result<(), Box<dyn std::error::Error + Send + Sync>>
{
    let _: () = for age in [89, 90, 91] {
        let harness = Harness::new()?;
        assert_status(&harness.record(&[])?, "recorded");
        let timeline = harness.timeline()?;
        age_events(&harness, age)?;
        force_sweep(&harness)?;
        assert!(trigger_hook(&harness)?.stderr.is_empty());
        assert_eq!(
            timeline
                .parent()
                .ok_or("fixture path has no parent")?
                .exists(),
            age < 90
        );
        let state = retention_state(&harness)?;
        assert_eq!(
            *(state)
                .get("status")
                .ok_or("missing fixture index status")?,
            "complete"
        );
        assert_eq!(
            *(state)
                .get("removed_prs")
                .ok_or("missing fixture index removed_prs")?,
            u64::from(age >= 90)
        );
    };
    Ok(())
}
#[test]
fn a_new_head_preserves_the_entire_history_and_triggers_maintenance()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    assert_status(&harness.record(&[])?, "recorded");
    age_events(&harness, 91)?;
    force_sweep(&harness)?;
    assert_status(&harness.record(&[("--head-sha", OTHER_SHA)])?, "recorded");
    assert_eq!(harness.events()?.len(), 2);
    assert_eq!(
        *(retention_state(&harness)?)
            .get("removed_prs")
            .ok_or("missing fixture index removed_prs")?,
        0
    );
    Ok(())
}
#[test]
fn sweeps_prs_at_most_once_per_day() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    assert_status(&harness.record(&[])?, "recorded");
    age_events(&harness, 91)?;
    assert!(trigger_hook(&harness)?.stderr.is_empty());
    assert_eq!(harness.events()?.len(), 1);
    force_sweep(&harness)?;
    let path = harness.timeline()?;
    assert!(trigger_hook(&harness)?.stderr.is_empty());
    assert!(!path.exists());
    Ok(())
}
#[test]
fn a_duplicate_does_not_refresh_the_expiration_clock()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    assert_status(&harness.record(&[])?, "recorded");
    age_events(&harness, 91)?;
    let path = harness.timeline()?;
    force_sweep(&harness)?;
    assert_status(&harness.record(&[])?, "duplicate");
    assert!(!path.exists());
    assert_status(&harness.record(&[])?, "recorded");
    assert_eq!(harness.events()?.len(), 1);
    Ok(())
}
#[test]
fn refuses_unsafe_or_unproven_expiration_without_deleting_data()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _: () = for damage in ["future", "empty", "truncated", "identity", "symlink"] {
        let harness = Harness::new()?;
        assert_status(&harness.record(&[])?, "recorded");
        age_events(&harness, 91)?;
        let timeline = harness.timeline()?;
        let mut event = harness.events()?.remove(0);
        match damage {
            "future" => {
                {
                    event
                        .as_object_mut()
                        .ok_or("expected JSON object for fixture update")?
                        .insert(("timestamp_ms").to_owned(), json!(now_ms()? + DAY_MS));
                };
                write_events(&harness, &[event])?;
            }
            "empty" => fs::write(&timeline, "")?,
            "truncated" => fs::write(&timeline, "{")?,
            "identity" => {
                {
                    (event)
                        .get_mut("pr")
                        .ok_or("missing fixture index pr")?
                        .as_object_mut()
                        .ok_or("expected JSON object for fixture update")?
                        .insert(("pr_id").to_owned(), json!(999));
                };
                write_events(&harness, &[event])?;
            }
            _ => {
                let outside = harness.root.path().join("outside");
                fs::write(&outside, "sentinel")?;
                symlink(
                    outside,
                    timeline
                        .parent()
                        .ok_or("fixture path has no parent")?
                        .join("unsafe"),
                )?;
            }
        }
        let before = fs::read(&timeline)?;
        force_sweep(&harness)?;
        let output = trigger_hook(&harness)?;
        assert_eq!(output.status.code(), Some(0));
        assert!(!output.stderr.is_empty(), "{damage}");
        assert_eq!(
            *(retention_state(&harness)?)
                .get("status")
                .ok_or("missing fixture index status")?,
            "failed"
        );
        assert_eq!(fs::read(timeline)?, before);
    };
    Ok(())
}
#[test]
fn concurrent_writers_and_purge_preserve_every_new_head()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    assert_status(&harness.record(&[])?, "recorded");
    age_events(&harness, 91)?;
    force_sweep(&harness)?;
    let children: Vec<_> = (0..16)
        .map(
            |index| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                Ok({
                    let sha = format!("{index:040x}");
                    harness.command(&[("--head-sha", &sha)]).spawn()?
                })
            },
        )
        .collect::<Result<Vec<_>, _>>()?;
    let hook = trigger_hook(&harness)?;
    assert!(
        hook.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&hook.stderr)
    );
    for child in children {
        assert_status(&child.wait_with_output()?, "recorded");
    }
    let events = harness.events()?;
    let _: () = for index in 0..16 {
        assert_eq!(
            events
                .iter()
                .map(
                    |event| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                        let include = *(event)
                            .get("head_sha")
                            .ok_or("missing fixture index head_sha")?
                            == format!("{index:040x}");
                        Ok((include, event))
                    }
                )
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .filter(|(include, _)| *include)
                .count(),
            1
        );
    };
    Ok(())
}
#[test]
fn reads_the_existing_retention_state_before_upgrading_it()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    assert_status(&harness.record(&[])?, "recorded");
    let path = harness.measure_root().join("retention.json");
    fs :: write (& path , json ! ({ "schema_version" : 1 , "status" : "complete" , "swept_at_ms" : 1 , "next_sweep_at_ms" : 2 , "candidate_runs" : 0 , "removed_runs" : 0 }) . to_string ()) ? ;
    assert!(trigger_hook(&harness)?.stderr.is_empty());
    assert_eq!(
        *(retention_state(&harness)?)
            .get("schema_version")
            .ok_or("missing fixture index schema_version")?,
        2
    );
    Ok(())
}
#[test]
fn invalid_maintenance_state_reports_failure_after_preserving_the_new_verdict()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _: () = for invalid in [
        json!({}),
        json ! ({ "schema_version" : 2 , "status" : "complete" , "swept_at_ms" : 1 , "next_sweep_at_ms" : 2 , "candidate_runs" : 0 , "removed_runs" : 0 }),
        json ! ({ "schema_version" : 2 , "status" : "complete" , "swept_at_ms" : 1 , "next_sweep_at_ms" : 2 , "candidate_runs" : 0 , "removed_runs" : 0 , "candidate_prs" : 0 , "removed_prs" : 1 }),
    ] {
        let harness = Harness::new()?;
        assert_status(&harness.record(&[])?, "recorded");
        fs::write(
            harness.measure_root().join("retention.json"),
            invalid.to_string(),
        )?;
        assert_error(
            &harness.record(&[("--head-sha", OTHER_SHA)])?,
            "PR event recorded; retention failed:",
        );
        assert_eq!(harness.events()?.len(), 2);
        assert_eq!(
            *(*(*(harness.events()?)
                .get(1)
                .ok_or("missing fixture index 1")?)
            .get("data")
            .ok_or("missing fixture index data")?)
            .get("verdict")
            .ok_or("missing fixture index verdict")?,
            "changes-required"
        );
        assert_eq!(retention_state(&harness)?, invalid);
    };
    Ok(())
}
fn trigger_hook(harness: &Harness) -> Result<Output, Box<dyn std::error::Error + Send + Sync>> {
    let mut child = harness
        .base_command()
        .args(["measure", "hook", "--agent", "codex"])
        .stdin(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("required test value is missing")?
        .write_all(br#"{"session_id":"retention-trigger","hook_event_name":"Stop"}"#)?;
    Ok(child.wait_with_output()?)
}
fn age_events(
    harness: &Harness,
    days: u64,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut events = harness.events()?;
    for event in &mut events {
        {
            event
                .as_object_mut()
                .ok_or("expected JSON object for fixture update")?
                .insert(
                    ("timestamp_ms").to_owned(),
                    json!(now_ms()? - days * DAY_MS),
                );
        };
    }
    write_events(harness, &events)?;
    Ok(())
}
fn write_events(
    harness: &Harness,
    events: &[Value],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut file = fs::File::create(harness.timeline()?)?;
    let _: () = for event in events {
        serde_json::to_writer(&mut file, event)?;
        writeln!(file)?;
    };
    Ok(())
}
fn force_sweep(harness: &Harness) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let path = harness.measure_root().join("retention.json");
    let _: () = if path.exists() {
        fs::remove_file(path)?;
    };
    Ok(())
}
fn retention_state(harness: &Harness) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
    Ok(serde_json::from_slice(&fs::read(
        harness.measure_root().join("retention.json"),
    )?)?)
}
fn now_ms() -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis()
        .try_into()?)
}
