use agent_memory::{HookAgent, HookErrorClass, parse_hook_request};
use std::path::PathBuf;

const CODEX_MINIMAL: &[u8] = include_bytes!("../fixtures/hooks/codex-minimal.json");
const CODEX_COMPLETE: &[u8] = include_bytes!("../fixtures/hooks/codex-complete.json");
const CLAUDE_MINIMAL: &[u8] = include_bytes!("../fixtures/hooks/claude-minimal.json");
const CLAUDE_COMPLETE: &[u8] = include_bytes!("../fixtures/hooks/claude-complete.json");

#[test]
fn parses_minimal_and_complete_host_payloads() {
    let cases = [
        (
            HookAgent::Codex,
            CODEX_MINIMAL,
            "durable memory",
            "/workspace",
        ),
        (
            HookAgent::Codex,
            CODEX_COMPLETE,
            "Apply durable memory",
            "/workspace/project",
        ),
        (
            HookAgent::Claude,
            CLAUDE_MINIMAL,
            "durable memory",
            "/workspace",
        ),
        (
            HookAgent::Claude,
            CLAUDE_COMPLETE,
            "Apply durable memory",
            "/workspace/project",
        ),
    ];

    for (agent, bytes, query, cwd) in cases {
        let request = parse_hook_request(agent, bytes).unwrap();
        assert_eq!(request.query, query);
        assert_eq!(request.cwd, PathBuf::from(cwd));
        assert_eq!(request.cwd.to_str(), Some(cwd));
    }
}

#[test]
fn rejects_wrong_events_and_ambiguous_payloads() {
    let cases = [
        (
            br#"{"prompt":"durable memory","cwd":"/workspace"}"#.as_slice(),
            "missing_hook_event",
        ),
        (
            br#"{"hook_event_name":[],"prompt":"durable memory","cwd":"/workspace"}"#.as_slice(),
            "invalid_hook_event",
        ),
        (
            br#"{"hook_event_name":null,"prompt":"durable memory","cwd":"/workspace"}"#.as_slice(),
            "invalid_hook_event",
        ),
        (
            br#"{"hook_event_name":"Stop","prompt":"durable memory","cwd":"/workspace"}"#.as_slice(),
            "invalid_hook_event",
        ),
        (br#"{"#.as_slice(), "invalid_hook_payload"),
        (
            br#"{"hook_event_name":"UserPromptSubmit","prompt":"first","prompt":"second","cwd":"/workspace"}"#.as_slice(),
            "invalid_hook_payload",
        ),
    ];

    for agent in [HookAgent::Codex, HookAgent::Claude] {
        for (bytes, code) in cases {
            let error = parse_hook_request(agent, bytes).unwrap_err();
            assert_eq!(error.class(), HookErrorClass::Rejection);
            assert_eq!(error.code(), code);
        }
    }
}

#[test]
fn rejects_missing_wrong_type_or_empty_query_and_cwd() {
    let cases = [
        (
            br#"{"hook_event_name":"UserPromptSubmit","cwd":"/workspace"}"#.as_slice(),
            "missing_hook_query",
        ),
        (
            br#"{"hook_event_name":"UserPromptSubmit","prompt":[],"cwd":"/workspace"}"#.as_slice(),
            "invalid_hook_query",
        ),
        (
            br#"{"hook_event_name":"UserPromptSubmit","prompt":null,"cwd":"/workspace"}"#.as_slice(),
            "invalid_hook_query",
        ),
        (
            br#"{"hook_event_name":"UserPromptSubmit","prompt":"   ","cwd":"/workspace"}"#.as_slice(),
            "invalid_hook_query",
        ),
        (
            br#"{"hook_event_name":"UserPromptSubmit","prompt":"durable memory"}"#.as_slice(),
            "missing_hook_cwd",
        ),
        (
            br#"{"hook_event_name":"UserPromptSubmit","prompt":"durable memory","cwd":4}"#.as_slice(),
            "invalid_hook_cwd",
        ),
        (
            br#"{"hook_event_name":"UserPromptSubmit","prompt":"durable memory","cwd":null}"#.as_slice(),
            "invalid_hook_cwd",
        ),
        (
            br#"{"hook_event_name":"UserPromptSubmit","prompt":"durable memory","cwd":"relative"}"#.as_slice(),
            "invalid_hook_cwd",
        ),
        (
            br#"{"hook_event_name":"UserPromptSubmit","prompt":"durable memory","cwd":"/workspace/../private"}"#.as_slice(),
            "invalid_hook_cwd",
        ),
    ];

    for agent in [HookAgent::Codex, HookAgent::Claude] {
        for (bytes, code) in cases {
            assert_eq!(parse_hook_request(agent, bytes).unwrap_err().code(), code);
        }
    }
}

#[test]
fn rejects_empty_and_oversized_payloads_before_json_parsing() {
    for agent in [HookAgent::Codex, HookAgent::Claude] {
        assert_eq!(
            parse_hook_request(agent, b"").unwrap_err().code(),
            "empty_stdin"
        );
        let oversized = vec![b'{'; 1024 * 1024 + 1];
        assert_eq!(
            parse_hook_request(agent, &oversized).unwrap_err().code(),
            "input_too_large"
        );
    }
}
