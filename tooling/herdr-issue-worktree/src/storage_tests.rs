use super::storage::{Request, dispatch};
use std::collections::BTreeMap;
use std::error::Error;
use std::fs::{self, File};

#[test]
fn stale_update_cannot_overwrite_another_writer() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let first = BTreeMap::from([("identity".into(), "original".into())]);
    dispatch(
        directory.path(),
        Request::Replace {
            pane_id: "p1".into(),
            expected: BTreeMap::new(),
            tokens: first.clone(),
        },
    )?;
    let second = BTreeMap::from([("identity".into(), "updated".into())]);
    dispatch(
        directory.path(),
        Request::Replace {
            pane_id: "p1".into(),
            expected: BTreeMap::from([("p1".into(), first.clone())]),
            tokens: second.clone(),
        },
    )?;
    let bytes = fs::read(directory.path().join("bindings.json"))?;
    assert!(
        dispatch(
            directory.path(),
            Request::Replace {
                pane_id: "p1".into(),
                expected: BTreeMap::from([("p1".into(), first)]),
                tokens: BTreeMap::new()
            }
        )
        .expect_err("contended mutation must be unapplied")
        .is::<super::storage::Conflict>()
    );
    assert_eq!(fs::read(directory.path().join("bindings.json"))?, bytes);
    assert_eq!(
        dispatch(directory.path(), Request::Read {})?.get("p1"),
        Some(&second)
    );
    Ok(())
}

#[test]
fn live_writer_lock_refuses_another_mutation() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let lock = File::create(directory.path().join("bindings.lock"))?;
    lock.lock()?;
    assert!(
        dispatch(
            directory.path(),
            Request::Replace {
                pane_id: "p1".into(),
                expected: BTreeMap::new(),
                tokens: BTreeMap::new()
            }
        )
        .expect_err("contended mutation must be unapplied")
        .is::<super::storage::Conflict>()
    );
    assert!(!directory.path().join("bindings.json").exists());
    drop(lock);
    dispatch(
        directory.path(),
        Request::Replace {
            pane_id: "p1".into(),
            expected: BTreeMap::new(),
            tokens: BTreeMap::new(),
        },
    )?;
    assert!(directory.path().join("bindings.json").exists());
    Ok(())
}

#[test]
fn unsupported_or_corrupt_state_is_never_replaced() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("bindings.json");
    for contents in [
        "malformed{",
        "{\"version\":2,\"bindings\":{}}",
        "{\"version\":1,\"bindings\":{},\"future\":true}",
    ] {
        fs::write(&path, contents)?;
        assert!(dispatch(directory.path(), Request::Read {}).is_err());
        assert!(
            dispatch(
                directory.path(),
                Request::Replace {
                    pane_id: "p1".into(),
                    expected: BTreeMap::new(),
                    tokens: BTreeMap::new()
                }
            )
            .is_err()
        );
        assert_eq!(fs::read_to_string(&path)?, contents);
    }
    Ok(())
}

#[test]
fn unknown_request_is_rejected_before_storage() {
    assert!(
        serde_json::from_str::<Request>("{\"operation\":\"read\",\"unexpected\":true}").is_err()
    );
    assert!(serde_json::from_str::<Request>("{\"operation\":\"unknown\"}").is_err());
}

#[test]
fn stale_snapshot_for_another_pane_is_refused() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let snapshot = dispatch(directory.path(), Request::Read {})?;
    dispatch(
        directory.path(),
        Request::Replace {
            pane_id: "p1".into(),
            expected: snapshot.clone(),
            tokens: BTreeMap::from([("identity".into(), "one".into())]),
        },
    )?;
    let bytes = fs::read(directory.path().join("bindings.json"))?;
    assert!(
        dispatch(
            directory.path(),
            Request::Replace {
                pane_id: "p2".into(),
                expected: snapshot,
                tokens: BTreeMap::from([("identity".into(), "two".into())])
            }
        )
        .is_err()
    );
    assert_eq!(fs::read(directory.path().join("bindings.json"))?, bytes);
    Ok(())
}
