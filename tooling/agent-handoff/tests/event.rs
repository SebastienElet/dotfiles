use agent_handoff::{HandoffError, HookEvent, parse_hook_event};
use std::path::PathBuf;

#[test]
fn error_constructors_assign_their_contract_exit_codes() {
    assert_eq!(HandoffError::usage("usage failure").exit_code, 1);
    assert_eq!(HandoffError::unexpected("unexpected failure").exit_code, 3);
}

#[test]
fn invalid_json_and_non_object_values_are_rejected() {
    assert_eq!(
        parse_hook_event(b"not-json").unwrap_err(),
        HandoffError::usage("invalid hook event: expected JSON")
    );
    assert_eq!(
        parse_hook_event(&[0xff]).unwrap_err(),
        HandoffError::usage("invalid hook event: expected JSON")
    );
    assert_eq!(
        parse_hook_event(b"null").unwrap_err(),
        HandoffError::usage("invalid hook event: expected an object")
    );
}

#[test]
fn event_name_is_required_and_each_present_name_must_be_stop() {
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
            parse_hook_event(input).unwrap_err(),
            HandoffError::usage(message)
        );
    }
}

#[test]
fn session_id_must_be_a_nonempty_safe_component() {
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
            parse_hook_event(input).unwrap_err(),
            HandoffError::usage(message)
        );
    }
}

#[test]
fn transcript_path_must_be_a_nonempty_string() {
    let cases = [
        br#"{"hook_event_name":"Stop","session_id":"session"}"#.as_slice(),
        br#"{"hook_event_name":"Stop","session_id":"session","transcript_path":""}"#.as_slice(),
        br#"{"hook_event_name":"Stop","session_id":"session","transcript_path":1}"#.as_slice(),
    ];

    for input in cases {
        assert_eq!(
            parse_hook_event(input).unwrap_err(),
            HandoffError::usage("missing transcript_path")
        );
    }
}

#[test]
fn stop_hook_active_must_be_boolean_when_present() {
    assert_eq!(
        parse_hook_event(
            br#"{"hook_event_name":"Stop","session_id":"session","transcript_path":"/tmp/transcript","stop_hook_active":"true"}"#
        )
        .unwrap_err(),
        HandoffError::usage("invalid stop_hook_active")
    );
}

#[test]
fn claude_and_codex_stop_events_are_parsed() {
    assert_eq!(
        parse_hook_event(
            br#"{"hook_event_name":"Stop","session_id":"Ab9._-","transcript_path":"/tmp/t","stop_hook_active":true}"#
        )
        .unwrap(),
        HookEvent {
            session_id: "Ab9._-".into(),
            stop_hook_active: true,
            transcript_path: PathBuf::from("/tmp/t"),
        }
    );
    assert_eq!(
        parse_hook_event(br#"{"event":"Stop","session_id":"s-1","transcript_path":"/tmp/t"}"#)
            .unwrap(),
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
        .unwrap(),
        HookEvent {
            session_id: "s.1".into(),
            stop_hook_active: false,
            transcript_path: PathBuf::from("relative"),
        }
    );
}
