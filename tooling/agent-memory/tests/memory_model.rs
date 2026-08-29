use agent_memory::{MemoryKind, Status, parse_entry};

const ACTIVE: &str = "active";
const INVALIDATED: &str = "invalidated";

fn transition(status: &str) -> String {
    let verdict = if status == INVALIDATED {
        "invalid"
    } else {
        "valid"
    };
    format!(
        "transition:\n  from: active\n  to: {status}\n  at: 2026-08-28T10:00:00Z\n  verdict: {verdict}\n  reason: The observable outcome was established.\n"
    )
}

fn entry(kind: &str, status: &str) -> Vec<u8> {
    let transition = if status == ACTIVE {
        String::new()
    } else {
        transition(status)
    };
    format!(
        "schema_version: 1\nid: mem_0123456789abcdef01234567\nkind: {kind}\nstatus: {status}\nstatement: This durable statement is independently useful.\nscope:\n  type: project\n  key: project_0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef\nretrieval_terms:\n  - durable statement\nproof:\n  summary: A tracked source establishes the statement.\n  sources:\n    - kind: git-file\n      locator: docs/contract.md\n      fingerprint: sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef\n  established_at: 2026-08-28T09:00:00Z\noracle:\n  automated:\n    kind: source-fingerprint\n    expected: all-proof-sources-unchanged\n  human_fallback:\n    question: Does the tracked source still establish this statement?\n    valid_when: The source retains the same normative requirement.\n  outcomes:\n    valid: The fingerprints and requirement are unchanged.\n    invalidated: The source changed or removed the requirement.\ncreated_at: 2026-08-28T09:00:00Z\n{transition}"
    )
    .into_bytes()
}

#[test]
fn parses_every_closed_kind() {
    let cases = [
        ("goal", MemoryKind::Goal),
        ("decision", MemoryKind::Decision),
        ("evidence", MemoryKind::Evidence),
        ("invariant", MemoryKind::Invariant),
        ("unknown", MemoryKind::Unknown),
        ("assumption", MemoryKind::Assumption),
    ];

    for (kind, expected) in cases {
        assert_eq!(parse_entry(&entry(kind, ACTIVE)).unwrap().kind(), expected);
    }
}

#[test]
fn enforces_the_kind_status_matrix() {
    let statuses = [
        ACTIVE,
        "achieved",
        "abandoned",
        "superseded",
        INVALIDATED,
        "resolved",
        "confirmed",
    ];
    let cases: [(&str, &[&str]); 6] = [
        ("goal", &[ACTIVE, "achieved", "abandoned", INVALIDATED]),
        ("decision", &[ACTIVE, "superseded", INVALIDATED]),
        ("evidence", &[ACTIVE, INVALIDATED]),
        ("invariant", &[ACTIVE, INVALIDATED]),
        ("unknown", &[ACTIVE, "resolved", INVALIDATED]),
        ("assumption", &[ACTIVE, "confirmed", INVALIDATED]),
    ];

    for (kind, allowed) in cases {
        for status in statuses {
            let result = parse_entry(&entry(kind, status));
            if allowed.contains(&status) {
                assert!(result.is_ok(), "{kind}/{status}: {result:?}");
            } else {
                assert_eq!(
                    result.unwrap_err().code(),
                    "invalid_kind_status",
                    "{kind}/{status}"
                );
            }
        }
    }
}

#[test]
fn enforces_status_transition_coherence() {
    let active_invariant = entry("invariant", ACTIVE);
    assert_eq!(
        parse_entry(&active_invariant).unwrap().status(),
        Status::Active
    );

    let goal_with_status = entry("goal", "superseded");
    assert_eq!(
        parse_entry(&goal_with_status).unwrap_err().code(),
        "invalid_kind_status"
    );

    let mut active_with_transition = String::from_utf8(entry("goal", ACTIVE)).unwrap();
    active_with_transition.push_str(&transition("achieved"));
    assert_eq!(
        parse_entry(active_with_transition.as_bytes())
            .unwrap_err()
            .code(),
        "unexpected_transition"
    );

    let terminal_without_transition = String::from_utf8(entry("goal", "achieved"))
        .unwrap()
        .replace("transition:\n  from: active\n  to: achieved\n  at: 2026-08-28T10:00:00Z\n  verdict: valid\n  reason: The observable outcome was established.\n", "");
    assert_eq!(
        parse_entry(terminal_without_transition.as_bytes())
            .unwrap_err()
            .code(),
        "missing_transition"
    );
}

#[test]
fn rejects_incoherent_transition_values() {
    let cases = [
        ("from", "from: active", "from: achieved"),
        ("to", "to: achieved", "to: abandoned"),
        ("verdict", "verdict: valid", "verdict: invalid"),
    ];

    for (label, from, to) in cases {
        let yaml = String::from_utf8(entry("goal", "achieved"))
            .unwrap()
            .replace(from, to);
        assert_eq!(
            parse_entry(yaml.as_bytes()).unwrap_err().code(),
            "invalid_transition",
            "{label}"
        );
    }
}

#[test]
fn rejects_future_schema_and_duplicate_yaml_keys() {
    let future_schema = String::from_utf8(entry("invariant", ACTIVE))
        .unwrap()
        .replacen("schema_version: 1", "schema_version: 2", 1);
    assert_eq!(
        parse_entry(future_schema.as_bytes()).unwrap_err().code(),
        "unsupported_schema"
    );

    let duplicate_yaml_key = String::from_utf8(entry("invariant", ACTIVE))
        .unwrap()
        .replacen("status: active", "status: active\nstatus: active", 1);
    assert_eq!(
        parse_entry(duplicate_yaml_key.as_bytes())
            .unwrap_err()
            .code(),
        "duplicate_field"
    );
}

#[test]
fn closed_schema_refuses_executable_shapes() {
    let command_field = String::from_utf8(entry("invariant", ACTIVE))
        .unwrap()
        .replacen(
            "statement: This durable statement is independently useful.",
            "statement: This durable statement is independently useful.\ncommand: printf unsafe",
            1,
        );
    assert_eq!(
        parse_entry(command_field.as_bytes()).unwrap_err().code(),
        "unknown_field"
    );

    let command_source = String::from_utf8(entry("invariant", ACTIVE))
        .unwrap()
        .replacen("kind: git-file", "kind: command", 1);
    assert_eq!(
        parse_entry(command_source.as_bytes()).unwrap_err().code(),
        "invalid_source_kind"
    );
}

#[test]
fn rejects_invalid_validated_newtypes() {
    let cases = [
        ("id", "mem_0123456789abcdef01234567", "mem_short"),
        (
            "project key",
            "project_0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            "project_short",
        ),
        (
            "fingerprint",
            "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            "sha256:not-hex",
        ),
        ("timestamp", "2026-08-28T09:00:00Z", "2026-08-28 09:00:00"),
    ];

    for (label, valid, invalid) in cases {
        let yaml = String::from_utf8(entry("invariant", ACTIVE))
            .unwrap()
            .replacen(valid, invalid, 1);
        assert_eq!(
            parse_entry(yaml.as_bytes()).unwrap_err().code(),
            "invalid_field",
            "{label}"
        );
    }
}
