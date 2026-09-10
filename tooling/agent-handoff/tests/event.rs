#![cfg(test)]

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

use agent_handoff::{HandoffError, HookEvent, parse_hook_event};
use std::path::PathBuf;

#[test]
fn invalid_utf8_in_transcript_path_is_rejected_without_rewriting() -> TestResult {
    let input = b"{\"hook_event_name\":\"Stop\",\"session_id\":\"session\",\"transcript_path\":\"/tmp/\xff\"}";
    assert_eq!(
        parse_hook_event(input)
            .err()
            .ok_or("expected operation to fail")?,
        HandoffError::usage("invalid hook event: expected JSON")
    );
    Ok(())
}

#[test]
fn error_constructors_assign_their_contract_exit_codes() {
    assert_eq!(HandoffError::usage("usage failure").exit_code, 1);
    assert_eq!(HandoffError::unexpected("unexpected failure").exit_code, 3);
}

#[test]
fn invalid_json_and_non_object_values_are_rejected() -> TestResult {
    assert_eq!(
        parse_hook_event(b"not-json")
            .err()
            .ok_or("expected operation to fail")?,
        HandoffError::usage("invalid hook event: expected JSON")
    );
    assert_eq!(
        parse_hook_event(&[0xff])
            .err()
            .ok_or("expected operation to fail")?,
        HandoffError::usage("invalid hook event: expected JSON")
    );
    assert_eq!(
        parse_hook_event(b"null")
            .err()
            .ok_or("expected operation to fail")?,
        HandoffError::usage("invalid hook event: expected an object")
    );
    Ok(())
}

#[test]
fn event_name_is_required_and_each_present_name_must_be_stop() -> TestResult {
    let cases = [
        (
            br#"{"session_id":"session","transcript_path":"/tmp/transcript"}"#.as_slice(),
            "missing Stop event",
        ),
        (
            br#"{"hook_event_name":"UserPromptSubmit","session_id":"session","transcript_path":"/tmp/transcript"}"#.as_slice(),
            "unsupported hook event",
        ),
        (
            br#"{"event":"UserPromptSubmit","session_id":"session","transcript_path":"/tmp/transcript"}"#.as_slice(),
            "unsupported hook event",
        ),
        (
            br#"{"hook_event_name":"Stop","event":"UserPromptSubmit","session_id":"session","transcript_path":"/tmp/transcript"}"#.as_slice(),
            "unsupported hook event",
        ),
        (
            br#"{"hook_event_name":null,"session_id":"session","transcript_path":"/tmp/transcript"}"#.as_slice(),
            "unsupported hook event",
        ),
    ];

    for (input, message) in cases {
        assert_eq!(
            parse_hook_event(input)
                .err()
                .ok_or("expected operation to fail")?,
            HandoffError::usage(message)
        );
    }
    Ok(())
}

#[test]
fn session_id_must_be_a_nonempty_safe_component() -> TestResult {
    let cases = [
        (
            br#"{"hook_event_name":"Stop","transcript_path":"/tmp/transcript"}"#.as_slice(),
            "missing session_id",
        ),
        (
            br#"{"hook_event_name":"Stop","session_id":"","transcript_path":"/tmp/transcript"}"#
                .as_slice(),
            "missing session_id",
        ),
        (
            br#"{"hook_event_name":"Stop","session_id":1,"transcript_path":"/tmp/transcript"}"#
                .as_slice(),
            "missing session_id",
        ),
        (
            br#"{"hook_event_name":"Stop","session_id":".","transcript_path":"/tmp/transcript"}"#
                .as_slice(),
            "invalid session_id",
        ),
        (
            br#"{"hook_event_name":"Stop","session_id":"..","transcript_path":"/tmp/transcript"}"#
                .as_slice(),
            "invalid session_id",
        ),
        (
            br#"{"hook_event_name":"Stop","session_id":"a/b","transcript_path":"/tmp/transcript"}"#
                .as_slice(),
            "invalid session_id",
        ),
        (
            br#"{"hook_event_name":"Stop","session_id":"a b","transcript_path":"/tmp/transcript"}"#
                .as_slice(),
            "invalid session_id",
        ),
        (
            r#"{"hook_event_name":"Stop","session_id":"é","transcript_path":"/tmp/transcript"}"#
                .as_bytes(),
            "invalid session_id",
        ),
    ];

    for (input, message) in cases {
        assert_eq!(
            parse_hook_event(input)
                .err()
                .ok_or("expected operation to fail")?,
            HandoffError::usage(message)
        );
    }
    Ok(())
}

#[test]
fn transcript_path_must_be_a_nonempty_string() -> TestResult {
    let cases = [
        br#"{"hook_event_name":"Stop","session_id":"session"}"#.as_slice(),
        br#"{"hook_event_name":"Stop","session_id":"session","transcript_path":""}"#.as_slice(),
        br#"{"hook_event_name":"Stop","session_id":"session","transcript_path":1}"#.as_slice(),
    ];

    for input in cases {
        assert_eq!(
            parse_hook_event(input)
                .err()
                .ok_or("expected operation to fail")?,
            HandoffError::usage("missing transcript_path")
        );
    }
    Ok(())
}

#[test]
fn stop_hook_active_must_be_boolean_when_present() -> TestResult {
    assert_eq!(
        parse_hook_event(
            br#"{"hook_event_name":"Stop","session_id":"session","transcript_path":"/tmp/transcript","stop_hook_active":"true"}"#
        )
        .err().ok_or("expected operation to fail")?,
        HandoffError::usage("invalid stop_hook_active")
    );
    Ok(())
}

#[test]
fn claude_and_codex_stop_events_are_parsed() -> TestResult {
    assert_eq!(
        parse_hook_event(
            br#"{"hook_event_name":"Stop","session_id":"Ab9._-","transcript_path":"/tmp/t","stop_hook_active":true}"#
        )
        ?,
        HookEvent {
            session_id: "Ab9._-".into(),
            stop_hook_active: true,
            transcript_path: PathBuf::from("/tmp/t"),
        }
    );
    assert_eq!(
        parse_hook_event(br#"{"event":"Stop","session_id":"s-1","transcript_path":"/tmp/t"}"#)?,
        HookEvent {
            session_id: "s-1".into(),
            stop_hook_active: false,
            transcript_path: PathBuf::from("/tmp/t"),
        }
    );
    assert_eq!(
        parse_hook_event(
            br#"{"hook_event_name":"Stop","event":"Stop","session_id":"s.1","transcript_path":"relative"}"#
        )
        ?,
        HookEvent {
            session_id: "s.1".into(),
            stop_hook_active: false,
            transcript_path: PathBuf::from("relative"),
        }
    );
    Ok(())
}
