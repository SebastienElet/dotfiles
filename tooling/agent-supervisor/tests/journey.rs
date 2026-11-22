mod support;
use serde_json::json;
use std::{fs, path::Path};
use support::{Fixture, Result, invoke};

#[test]
fn fresh_sessions_continue_one_task_once() -> Result<()> {
    let fixture = Fixture::new()?;
    fixture.init()?;
    let result = fixture.run()?;
    assert_eq!(result["phase"], "complete");
    assert_eq!(result["receipts"][0]["thread_id"], "thread-1");
    assert_eq!(result["receipts"][1]["thread_id"], "thread-2");
    assert_eq!(fixture.count()?, "2");
    fixture.run()?;
    assert_eq!(fixture.count()?, "2");
    assert_eq!(result["receipts"][0]["turns_started"], 1);
    assert_eq!(result["receipts"][1]["turns_started"], 1);
    Ok(())
}

#[test]
fn hook_continuations_stay_in_one_owned_process_before_the_fresh_session() -> Result<()> {
    let fixture = Fixture::new()?;
    fixture.scenario("hook-continuation")?;
    fixture.init()?;
    let result = fixture.run()?;
    assert_eq!(result["phase"], "complete");
    assert_eq!(fixture.count()?, "2");
    assert_eq!(result["receipts"][0]["turns_started"], 2);
    assert_eq!(result["receipts"][1]["turns_started"], 2);
    Ok(())
}

#[test]
fn uncertain_or_invalid_turns_are_never_resent() -> Result<()> {
    for scenario in [
        "missing-start",
        "old-thread",
        "invalid-result",
        "truncated",
        "failed",
        "wrong-task",
        "new-permission",
        "missing-note",
    ] {
        let fixture = Fixture::new()?;
        fixture.scenario(scenario)?;
        fixture.init()?;
        let result = fixture.run()?;
        assert_eq!(result["phase"], "unconfirmed", "{scenario}");
        let count = fixture.count()?;
        fixture.run()?;
        assert_eq!(fixture.count()?, count, "{scenario}");
    }
    Ok(())
}

#[test]
fn handoff_port_refusals_preserve_valid_output_but_prevent_continuation() -> Result<()> {
    for command in [
        "reject-note-validate",
        "reject-note-schema",
        "reject-skill-path",
    ] {
        let mut fixture = Fixture::new()?;
        let executable = fixture.root.path().join(command);
        std::os::unix::fs::symlink(env!("CARGO_BIN_EXE_supervisor-test-agent"), &executable)?;
        fixture.manifest["handoff"] = json!(executable);
        fixture.init()?;
        assert_eq!(fixture.run()?["phase"], "unconfirmed", "{command}");
        if command == "reject-note-validate" {
            assert_eq!(fixture.count()?, "1");
        } else {
            assert!(!fixture.repository.join("count").exists());
        }
        fixture.run()?;
        if command == "reject-note-validate" {
            assert_eq!(fixture.count()?, "1");
        }
    }
    Ok(())
}

#[test]
fn decisions_and_dependencies_exit_until_explicit_resolution() -> Result<()> {
    for scenario in ["decision", "dependency"] {
        let fixture = Fixture::new()?;
        fixture.scenario(scenario)?;
        fixture.init()?;
        let result = fixture.run()?;
        assert_eq!(result["phase"], format!("waiting_{scenario}"));
        fixture.run()?;
        assert_eq!(fixture.count()?, "1");
        let resolution = invoke(
            &["resolve", &fixture.directory],
            Some(&json!({"attempt":1,"answer":"synthetic human resolution"})),
        )?;
        assert!(resolution.status.success());
        fixture.scenario("complete")?;
        assert_eq!(fixture.run()?["phase"], "complete");
        assert_eq!(fixture.count()?, "2");
    }
    Ok(())
}

#[test]
fn capacity_recovers_without_switching_provider() -> Result<()> {
    let fixture = Fixture::new()?;
    fs::write(fixture.repository.join("capacity"), "exhausted-once")?;
    fixture.init()?;
    let result = fixture.run()?;
    assert_eq!(result["phase"], "complete");
    assert_eq!(result["capacity_probes"], 3);
    assert_eq!(fixture.count()?, "2");
    Ok(())
}

#[test]
fn absent_or_exhausted_capacity_is_bounded_and_fail_closed() -> Result<()> {
    for capacity in [
        "exhausted",
        "empty",
        "invalid",
        "no-windows",
        "failed",
        "wrong-provider",
        "provider-error",
    ] {
        let mut fixture = Fixture::new()?;
        fixture.manifest["max_capacity_probes"] = json!(1);
        fs::write(fixture.repository.join("capacity"), capacity)?;
        fixture.init()?;
        let result = fixture.run()?;
        assert_eq!(result["phase"], "budget_exhausted", "{capacity}");
        assert!(!fixture.repository.join("count").exists());
    }
    Ok(())
}

#[test]
fn a_provider_with_only_a_weekly_window_can_run() -> Result<()> {
    let fixture = Fixture::new()?;
    fs::write(fixture.repository.join("capacity"), "secondary-only")?;
    fixture.init()?;
    assert_eq!(fixture.run()?["phase"], "complete");
    Ok(())
}

#[test]
fn stopped_or_expired_tasks_cannot_launch() -> Result<()> {
    let fixture = Fixture::new()?;
    fixture.init()?;
    invoke(&["stop", &fixture.directory], None)?;
    assert_eq!(fixture.run()?["phase"], "stopped");
    assert!(!fixture.repository.join("count").exists());
    let result = invoke(&["resolve", &fixture.directory], Some(&json!("resume")))?;
    assert!(!result.status.success());
    Ok(())
}

#[test]
fn state_cannot_live_inside_the_worktree_even_from_a_subdirectory() -> Result<()> {
    let mut fixture = Fixture::new()?;
    let subdirectory = fixture.repository.join("nested");
    fs::create_dir(&subdirectory)?;
    fixture.manifest["repository"] = json!(subdirectory);
    let directory = fixture.repository.join("task");
    let result = invoke(
        &["init", directory.to_str().ok_or("path")?],
        Some(&fixture.manifest),
    )?;
    assert!(!result.status.success());
    assert!(!directory.exists());
    Ok(())
}

#[test]
fn empty_permission_entries_are_refused_before_creating_the_task() -> Result<()> {
    for permission in ["", " ", "\n"] {
        let mut fixture = Fixture::new()?;
        fixture.manifest["permissions"] = json!([permission]);
        let output = invoke(&["init", &fixture.directory], Some(&fixture.manifest))?;
        assert!(!output.status.success());
        assert!(!Path::new(&fixture.directory).exists());
        assert!(!fixture.repository.join("count").exists());
    }
    Ok(())
}

#[test]
fn draft_and_pending_question_survive_checkout_removal() -> Result<()> {
    let fixture = Fixture::new()?;
    fixture.scenario("decision")?;
    fixture.init()?;
    let draft = Path::new(&fixture.directory).join("drafts/proposal");
    fs::write(&draft, "private synthetic draft")?;
    fixture.run()?;
    fs::remove_dir_all(&fixture.repository)?;
    let status = invoke(&["status", &fixture.directory], None)?;
    let state: serde_json::Value = serde_json::from_slice(&status.stdout)?;
    assert_eq!(
        state["checkpoint"]["decision"]["question"],
        "Which synthetic option?"
    );
    assert_eq!(fs::read_to_string(draft)?, "private synthetic draft");
    assert!(fixture.root.path().is_dir());
    Ok(())
}

fn wait_for(path: &Path) -> Result<()> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while !path.exists() {
        assert!(
            std::time::Instant::now() < deadline,
            "fixture never reached synchronization point"
        );
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    Ok(())
}

#[test]
fn concurrent_supervisors_cannot_share_a_task_or_worktree() -> Result<()> {
    let fixture = Fixture::new()?;
    fixture.scenario("active")?;
    fixture.init()?;
    let active = support::command(&["run", &fixture.directory]).spawn()?;
    wait_for(&fixture.repository.join("entered"))?;
    let duplicate = invoke(&["run", &fixture.directory], None)?;
    assert!(!duplicate.status.success());
    let second = fixture.root.path().join("second-task");
    let second = second.to_str().ok_or("path")?;
    assert!(
        invoke(&["init", second], Some(&fixture.manifest))?
            .status
            .success()
    );
    assert!(!invoke(&["run", second], None)?.status.success());
    invoke(&["stop", &fixture.directory], None)?;
    let result = active.wait_with_output()?;
    assert!(result.status.success());
    assert_eq!(fixture.run()?["phase"], "stopped");
    assert_eq!(fixture.count()?, "1");
    Ok(())
}

#[test]
fn signals_and_deadlines_stop_an_active_process_without_relaunch() -> Result<()> {
    for reason in ["signal", "deadline"] {
        let mut fixture = Fixture::new()?;
        fixture.scenario("active")?;
        fixture.manifest["turn_timeout_seconds"] = json!(1);
        fixture.init()?;
        let active = support::command(&["run", &fixture.directory]).spawn()?;
        wait_for(&fixture.repository.join("entered"))?;
        if reason == "signal" {
            for signal in ["-INT", "-TERM"] {
                let _ = std::process::Command::new("/bin/kill")
                    .args([signal, &active.id().to_string()])
                    .output()?;
            }
        }
        let result = active.wait_with_output()?;
        assert!(result.status.success());
        let result: serde_json::Value = serde_json::from_slice(&result.stdout)?;
        assert_eq!(
            result["phase"],
            if reason == "signal" {
                "stopped"
            } else {
                "unconfirmed"
            }
        );
        let child_pid = fs::read_to_string(fixture.repository.join("child-pid"))?;
        let process = std::process::Command::new("ps")
            .args(["-o", "stat=", "-p", &child_pid])
            .output()?;
        assert!(
            process.stdout.is_empty()
                || String::from_utf8_lossy(&process.stdout)
                    .trim()
                    .starts_with('Z')
        );
        fixture.run()?;
        assert_eq!(fixture.count()?, "1");
    }
    Ok(())
}

#[test]
fn changed_candidate_refuses_a_launch() -> Result<()> {
    let fixture = Fixture::new()?;
    fixture.init()?;
    assert!(
        std::process::Command::new("git")
            .arg("-C")
            .arg(&fixture.repository)
            .args(["checkout", "-qb", "another-branch"])
            .status()?
            .success()
    );
    assert!(!invoke(&["run", &fixture.directory], None)?.status.success());
    assert!(!fixture.repository.join("count").exists());
    Ok(())
}

#[test]
fn stale_resolution_cannot_release_a_pending_decision() -> Result<()> {
    let fixture = Fixture::new()?;
    fixture.scenario("decision")?;
    fixture.init()?;
    fixture.run()?;
    assert!(
        !invoke(
            &["resolve", &fixture.directory],
            Some(&json!({"attempt":0,"answer":"stale choice"}))
        )?
        .status
        .success()
    );
    assert_eq!(fixture.run()?["phase"], "waiting_decision");
    assert_eq!(fixture.count()?, "1");
    Ok(())
}

#[test]
fn checkpoint_is_published_even_if_checkout_disappears_during_the_turn() -> Result<()> {
    let fixture = Fixture::new()?;
    fixture.scenario("remove-checkout")?;
    fixture.init()?;
    let output = invoke(&["run", &fixture.directory], None)?;
    let status = invoke(&["status", &fixture.directory], None)?;
    let state: serde_json::Value = serde_json::from_slice(&status.stdout)?;
    assert_eq!(state["phase"], "waiting_decision");
    assert!(output.status.success());
    assert_eq!(
        state["checkpoint"]["decision"]["question"],
        "Which synthetic option?"
    );
    Ok(())
}

#[test]
fn oversized_state_is_refused_without_replacing_readable_state() -> Result<()> {
    let mut fixture = Fixture::new()?;
    let original_length = serde_json::to_vec(&fixture.manifest)?.len();
    fixture.manifest["goal"] = json!("X".repeat(1_048_576 - original_length - 2));
    assert!(
        !invoke(&["init", &fixture.directory], Some(&fixture.manifest))?
            .status
            .success()
    );
    let fixture = Fixture::new()?;
    fixture.scenario("decision")?;
    fixture.init()?;
    fixture.run()?;
    let before = fs::read(Path::new(&fixture.directory).join("state.json"))?;
    let answer = json!({"attempt":1,"answer":"X".repeat(1_048_000)});
    assert!(
        !invoke(&["resolve", &fixture.directory], Some(&answer))?
            .status
            .success()
    );
    assert_eq!(
        fs::read(Path::new(&fixture.directory).join("state.json"))?,
        before
    );
    assert!(
        invoke(&["status", &fixture.directory], None)?
            .status
            .success()
    );
    Ok(())
}

#[test]
fn observed_start_survives_failed_turn_and_crashed_supervisor() -> Result<()> {
    let fixture = Fixture::new()?;
    fixture.scenario("failed-after-start")?;
    fixture.init()?;
    let state = fixture.run()?;
    assert_eq!(state["phase"], "unconfirmed");
    assert_eq!(state["receipts"][0]["started"], true);
    let fixture = Fixture::new()?;
    fixture.scenario("active")?;
    fixture.init()?;
    let mut active = support::command(&["run", &fixture.directory]).spawn()?;
    wait_for(&fixture.repository.join("entered"))?;
    active.kill()?;
    active.wait()?;
    let agent = fs::read_to_string(fixture.repository.join("agent-pid"))?.parse::<i32>()?;
    nix::sys::signal::killpg(
        nix::unistd::Pid::from_raw(agent),
        nix::sys::signal::Signal::SIGKILL,
    )?;
    assert_eq!(fixture.run()?["phase"], "unconfirmed");
    assert_eq!(fixture.count()?, "1");
    Ok(())
}

#[test]
fn mandate_expires_while_waiting_for_capacity_without_any_launch() -> Result<()> {
    let mut fixture = Fixture::new()?;
    fixture.manifest["expires_at"] = json!(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs()
            + 2
    );
    fixture.manifest["retry_seconds"] = json!(5);
    fs::write(fixture.repository.join("capacity"), "exhausted")?;
    fixture.init()?;
    assert_eq!(fixture.run()?["phase"], "expired");
    assert!(!fixture.repository.join("count").exists());
    Ok(())
}
