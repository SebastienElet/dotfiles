use agent_handoff::{
    Environment, HandoffError, SentinelState, create_sentinel, inspect_sentinel, state_root,
};
use std::fs::{self, File};
use std::path::PathBuf;
use std::sync::{Arc, Barrier};
use std::thread;
use tempfile::TempDir;

fn environment(xdg_state_home: Option<&str>, home: Option<&str>) -> Environment {
    Environment {
        xdg_state_home: xdg_state_home.map(str::to_owned),
        home: home.map(str::to_owned),
        ..Environment::default()
    }
}

#[test]
fn state_root_prefers_nonempty_xdg_state_home() {
    assert_eq!(
        state_root(&environment(Some("/xdg"), Some("/home"))).unwrap(),
        PathBuf::from("/xdg")
    );
}

#[test]
fn state_root_falls_back_to_home_for_absent_or_empty_xdg_state_home() {
    for xdg_state_home in [None, Some("")] {
        assert_eq!(
            state_root(&environment(xdg_state_home, Some("/home"))).unwrap(),
            PathBuf::from("/home/.local/state")
        );
    }
}

#[test]
fn state_root_rejects_absent_home_and_xdg_state_home() {
    assert_eq!(
        state_root(&Environment::default()).unwrap_err(),
        HandoffError::usage("missing HOME and XDG_STATE_HOME")
    );
}

#[test]
fn sentinel_inspection_distinguishes_absence_from_an_existing_file() {
    let fixture = TempDir::new().unwrap();
    let path = fixture.path().join("handoff/sentinel");

    assert!(!inspect_sentinel(&path).unwrap());
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    File::create(&path).unwrap();
    assert!(inspect_sentinel(&path).unwrap());
}

#[test]
fn sentinel_inspection_rejects_directories_and_filesystem_errors() {
    let fixture = TempDir::new().unwrap();
    let directory = fixture.path().join("directory");
    fs::create_dir(&directory).unwrap();
    assert_eq!(
        inspect_sentinel(&directory).unwrap_err(),
        HandoffError::unexpected("cannot inspect handoff sentinel")
    );

    let blocked_parent = fixture.path().join("blocked");
    File::create(&blocked_parent).unwrap();
    assert_eq!(
        inspect_sentinel(&blocked_parent.join("child")).unwrap_err(),
        HandoffError::unexpected("cannot inspect handoff sentinel")
    );
}

#[test]
fn sentinel_creation_reports_created_then_existing() {
    let fixture = TempDir::new().unwrap();
    let path = fixture.path().join("nested/handoff/sentinel");

    assert_eq!(create_sentinel(&path).unwrap(), SentinelState::Created);
    assert_eq!(create_sentinel(&path).unwrap(), SentinelState::Existing);
    assert!(path.is_file());
}

#[test]
fn concurrent_sentinel_creation_has_exactly_one_creator() {
    let fixture = TempDir::new().unwrap();
    let path = Arc::new(fixture.path().join("handoff/sentinel"));
    let thread_count = 16;
    let barrier = Arc::new(Barrier::new(thread_count));
    let handles: Vec<_> = (0..thread_count)
        .map(|_| {
            let barrier = Arc::clone(&barrier);
            let path = Arc::clone(&path);
            thread::spawn(move || {
                barrier.wait();
                create_sentinel(&path).unwrap()
            })
        })
        .collect();
    let states: Vec<_> = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect();

    assert_eq!(
        states
            .iter()
            .filter(|state| **state == SentinelState::Created)
            .count(),
        1
    );
    assert_eq!(
        states
            .iter()
            .filter(|state| **state == SentinelState::Existing)
            .count(),
        thread_count - 1
    );
}

#[test]
fn sentinel_creation_rejects_a_parent_that_is_a_file() {
    let fixture = TempDir::new().unwrap();
    let blocked_parent = fixture.path().join("blocked");
    File::create(&blocked_parent).unwrap();

    assert_eq!(
        create_sentinel(&blocked_parent.join("child")).unwrap_err(),
        HandoffError::unexpected("cannot create handoff sentinel")
    );
}
