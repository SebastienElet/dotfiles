use agent_memory::{DeadlineProcessRunner, ProcessRunner};
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io;
use std::time::{Duration, Instant};

#[test]
fn kills_and_drains_a_process_that_exceeds_the_output_bound() {
    let runner = DeadlineProcessRunner::new(Instant::now() + Duration::from_secs(2));
    let started = Instant::now();
    let error = runner.run(OsStr::new("yes"), &[], None).unwrap_err();

    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    assert!(started.elapsed() < Duration::from_secs(1));
}

#[test]
fn kills_and_reaps_after_the_cooperative_work_cutoff_by_the_cleanup_deadline() {
    let runner = DeadlineProcessRunner::new(Instant::now() + Duration::from_millis(50));
    let started = Instant::now();
    let arguments = [OsString::from("5")];
    let error = runner
        .run(OsStr::new("sleep"), &arguments, None)
        .unwrap_err();

    assert_eq!(error.kind(), io::ErrorKind::TimedOut);
    assert!(started.elapsed() < Duration::from_secs(1));
}

#[test]
fn shares_one_work_budget_and_cleanup_deadline_across_processes() {
    let started = Instant::now();
    let runner = DeadlineProcessRunner::new(started + Duration::from_millis(500));
    runner
        .run(OsStr::new("sleep"), &[OsString::from("0.08")], None)
        .unwrap();
    let error = runner
        .run(OsStr::new("sleep"), &[OsString::from("5")], None)
        .unwrap_err();

    assert_eq!(error.kind(), io::ErrorKind::TimedOut);
    assert!(started.elapsed() < Duration::from_secs(1));
}

#[test]
fn kills_descendants_that_keep_output_pipes_open() {
    let runner = DeadlineProcessRunner::new(Instant::now() + Duration::from_millis(50));
    let started = Instant::now();
    let arguments = [OsString::from("-c"), OsString::from("sleep 5 & wait")];
    let error = runner.run(OsStr::new("sh"), &arguments, None).unwrap_err();

    assert_eq!(error.kind(), io::ErrorKind::TimedOut);
    assert!(started.elapsed() < Duration::from_secs(1));
}

#[test]
fn kills_descendants_after_the_parent_exits_with_open_pipes() {
    let runner = DeadlineProcessRunner::new(Instant::now() + Duration::from_millis(50));
    let started = Instant::now();
    let arguments = [OsString::from("-c"), OsString::from("sleep 2 &")];
    let error = runner.run(OsStr::new("sh"), &arguments, None).unwrap_err();

    assert_eq!(error.kind(), io::ErrorKind::TimedOut);
    assert!(started.elapsed() < Duration::from_secs(1));
}

#[test]
fn rejects_combined_stdout_and_stderr_larger_than_one_mebibyte() {
    let runner = DeadlineProcessRunner::new(Instant::now() + Duration::from_secs(2));
    let arguments = [
        OsString::from("-c"),
        OsString::from("head -c 700000 /dev/zero; head -c 700000 /dev/zero >&2"),
    ];

    let error = runner.run(OsStr::new("sh"), &arguments, None).unwrap_err();

    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
}

#[test]
fn closes_a_descendant_group_after_a_successful_leader_with_redirected_descriptors() {
    let fixture = tempfile::tempdir().unwrap();
    let state = fixture.path().join("descendant-pid");
    let runner = DeadlineProcessRunner::new(Instant::now() + Duration::from_secs(2));
    let arguments = [
        OsString::from("-c"),
        OsString::from("sleep 10 </dev/null >/dev/null 2>&1 & printf '%s' \"$!\" > \"$1\""),
        OsString::from("sh"),
        state.as_os_str().to_owned(),
    ];

    let output = runner.run(OsStr::new("sh"), &arguments, None).unwrap();
    let pid = fs::read_to_string(state)
        .unwrap()
        .trim()
        .parse::<i32>()
        .unwrap();
    let pid = rustix::process::Pid::from_raw(pid).unwrap();
    let alive = rustix::process::test_kill_process(pid).is_ok();
    if alive {
        rustix::process::kill_process(pid, rustix::process::Signal::KILL).unwrap();
    }

    assert!(output.success());
    assert!(!alive);
}
