use agent_memory::{DeadlineProcessRunner, ProcessRunner};
use std::ffi::{OsStr, OsString};
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
fn kills_and_reaps_a_process_at_the_deadline() {
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
fn shares_one_deadline_across_processes() {
    let started = Instant::now();
    let runner = DeadlineProcessRunner::new(started + Duration::from_millis(120));
    runner
        .run(OsStr::new("sleep"), &[OsString::from("0.08")], None)
        .unwrap();
    let error = runner
        .run(OsStr::new("sleep"), &[OsString::from("5")], None)
        .unwrap_err();

    assert_eq!(error.kind(), io::ErrorKind::TimedOut);
    assert!(started.elapsed() < Duration::from_millis(300));
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
