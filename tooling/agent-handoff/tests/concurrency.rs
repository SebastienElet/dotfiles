#![cfg(test)]

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

use agent_handoff::{
    Environment, HandoffError, SentinelState, create_sentinel, inspect_sentinel, state_root,
};
use std::ffi::OsString;
use std::fs::{self, File};
use std::os::unix::ffi::OsStringExt;
use std::path::PathBuf;
use std::sync::{Arc, Barrier};
use std::thread;
use tempfile::TempDir;

fn environment(xdg_state_home: Option<&str>, home: Option<&str>) -> Environment {
    Environment {
        xdg_state_home: xdg_state_home.map(Into::into),
        home: home.map(Into::into),
        ..Environment::default()
    }
}

#[test]
fn state_root_prefers_nonempty_xdg_state_home() -> TestResult {
    assert_eq!(
        state_root(&environment(Some("/xdg"), Some("/home")))?,
        PathBuf::from("/xdg")
    );
    Ok(())
}

#[test]
fn state_root_keeps_opaque_environment_path_bytes() -> TestResult {
    let path = OsString::from_vec(b"/state-\xff".to_vec());
    for variable in ["XDG_STATE_HOME", "HOME"] {
        let environment = Environment::from_iter([(OsString::from(variable), path.clone())]);
        let expected = if variable == "HOME" {
            PathBuf::from(&path).join(".local/state")
        } else {
            PathBuf::from(&path)
        };
        assert_eq!(state_root(&environment)?, expected);
    }
    Ok(())
}

#[test]
fn state_root_falls_back_to_home_for_absent_or_empty_xdg_state_home() -> TestResult {
    for xdg_state_home in [None, Some("")] {
        assert_eq!(
            state_root(&environment(xdg_state_home, Some("/home")))?,
            PathBuf::from("/home/.local/state")
        );
    }
    Ok(())
}

#[test]
fn state_root_lexically_normalizes_the_home_fallback() -> TestResult {
    assert_eq!(
        state_root(&environment(Some(""), Some("/home/file/..")))?,
        PathBuf::from("/home/.local/state")
    );
    Ok(())
}

#[test]
fn state_root_rejects_absent_home_and_xdg_state_home() -> TestResult {
    assert_eq!(
        state_root(&Environment::default())
            .err()
            .ok_or("expected operation to fail")?,
        HandoffError::usage("missing HOME and XDG_STATE_HOME")
    );
    Ok(())
}

#[test]
fn sentinel_inspection_distinguishes_absence_from_an_existing_file() -> TestResult {
    let fixture = TempDir::new()?;
    let path = fixture.path().join("handoff/sentinel");

    assert!(!inspect_sentinel(&path)?);
    fs::create_dir_all(path.parent().ok_or("fixture path has no parent")?)?;
    File::create(&path)?;
    assert!(inspect_sentinel(&path)?);
    Ok(())
}

#[test]
fn sentinel_inspection_rejects_directories_and_filesystem_errors() -> TestResult {
    let fixture = TempDir::new()?;
    let directory = fixture.path().join("directory");
    fs::create_dir(&directory)?;
    assert_eq!(
        inspect_sentinel(&directory)
            .err()
            .ok_or("expected operation to fail")?,
        HandoffError::unexpected("cannot inspect handoff sentinel")
    );

    let blocked_parent = fixture.path().join("blocked");
    File::create(&blocked_parent)?;
    assert_eq!(
        inspect_sentinel(&blocked_parent.join("child"))
            .err()
            .ok_or("expected operation to fail")?,
        HandoffError::unexpected("cannot inspect handoff sentinel")
    );
    Ok(())
}

#[test]
fn sentinel_creation_reports_created_then_existing() -> TestResult {
    let fixture = TempDir::new()?;
    let path = fixture.path().join("nested/handoff/sentinel");

    assert_eq!(create_sentinel(&path)?, SentinelState::Created);
    assert_eq!(create_sentinel(&path)?, SentinelState::Existing);
    assert!(path.is_file());
    Ok(())
}

#[test]
fn concurrent_sentinel_creation_has_exactly_one_creator() -> TestResult {
    let fixture = TempDir::new()?;
    let path = Arc::new(fixture.path().join("handoff/sentinel"));
    let thread_count = 16;
    let barrier = Arc::new(Barrier::new(thread_count));
    let mut handles = Vec::new();
    for _ in 0..thread_count {
        let barrier = Arc::clone(&barrier);
        let path = Arc::clone(&path);
        handles.push(thread::Builder::new().spawn(move || {
            barrier.wait();
            create_sentinel(&path)
        })?);
    }
    let mut states = Vec::new();
    for handle in handles {
        states.push(handle.join().map_err(|_| "sentinel worker panicked")??);
    }

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
    Ok(())
}

#[test]
fn sentinel_creation_rejects_a_parent_that_is_a_file() -> TestResult {
    let fixture = TempDir::new()?;
    let blocked_parent = fixture.path().join("blocked");
    File::create(&blocked_parent)?;

    assert_eq!(
        create_sentinel(&blocked_parent.join("child"))
            .err()
            .ok_or("expected operation to fail")?,
        HandoffError::unexpected("cannot create handoff sentinel")
    );
    Ok(())
}
