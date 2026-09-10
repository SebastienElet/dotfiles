#![cfg(test)]

use crate::memory_support::{FakeProcessRunner, FakeResponse, git};
use agent_memory::{
    AdmissionAuthorization, MemoryError, ProcessOutput, ProcessRunner, ResolvedDraft,
    SourceContext, SystemProcessRunner, parse_draft, resolve_sources, validate_draft,
};
use sha2::{Digest, Sha256};
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::Path;
use std::time::Duration;

const BODY: &[u8] = b"authoritative body";
const SUCCESS_METADATA: &str = "200\nhttps://docs.example.test/final\n203.0.113.10\n";

fn draft(kind: &str, locator: &str) -> Result<agent_memory::ValidatedDraft, MemoryError> {
    draft_with_sources(&[(kind, locator)])
}

fn draft_with_sources(
    sources: &[(&str, &str)],
) -> Result<agent_memory::ValidatedDraft, MemoryError> {
    draft_with_scope_sources("project", sources)
}

fn draft_with_scope_sources(
    scope: &str,
    sources: &[(&str, &str)],
) -> Result<agent_memory::ValidatedDraft, MemoryError> {
    let automated = if sources.iter().all(|(kind, _)| *kind == "user-decision") {
        ""
    } else {
        "  automated:\n    kind: source-fingerprint\n    expected: all-proof-sources-unchanged\n"
    };
    let proof_sources = serde_json::Value::Array(
        sources
            .iter()
            .map(|(kind, locator)| serde_json::json!({"kind": kind, "locator": locator}))
            .collect(),
    );
    let yaml = format!(
        "schema_version: 1\nkind: invariant\nstatement: A durable invariant remains independently useful.\nscope: {scope}\nretrieval_terms:\n  - durable invariant\nproof:\n  summary: The source establishes the invariant.\n  sources: {proof_sources}\noracle:\n{automated}  human_fallback:\n    question: Does the evidence still establish the invariant?\n    valid_when: The evidence remains observable.\n  outcomes:\n    valid: The evidence is unchanged.\n    invalidated: The evidence no longer establishes the invariant.\n"
    );
    validate_draft(
        parse_draft(yaml.as_bytes())?,
        AdmissionAuthorization::ExplicitRequest,
    )
}

fn official_url_draft(locator: &str) -> Result<agent_memory::ValidatedDraft, MemoryError> {
    draft_with_sources(&[
        ("official-url", locator),
        (
            "user-decision",
            "The user designated this proof as official.",
        ),
    ])
}

fn verdict(result: Result<ResolvedDraft, MemoryError>) -> Result<&'static str, MemoryError> {
    match result {
        Ok(_) => Ok("valid"),
        Err(error) if error.code() == "source_invalid" => Ok("invalid"),
        Err(error) if error.code() == "source_unavailable" => Ok("unavailable"),
        Err(error) => Err(error),
    }
}

fn expected_fingerprint(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

#[test]
fn fingerprints_a_tracked_git_file_and_observes_modifications()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let repository = tempfile::tempdir()?;
    git(repository.path(), &["init"])?;
    fs::create_dir(repository.path().join("docs"))?;
    fs::write(repository.path().join("docs/contract.md"), "first")?;
    git(repository.path(), &["add", "docs/contract.md"])?;
    let curl = FakeProcessRunner::default();
    let git_runner = SystemProcessRunner;
    let context = SourceContext::new(repository.path(), &git_runner, &curl);

    let first = resolve_sources(draft("git-file", "docs/contract.md")?, &context)?;
    assert_eq!(
        first
            .sources()
            .first()
            .ok_or("missing fixture element")?
            .fingerprint()
            .as_str(),
        expected_fingerprint(b"first")
    );

    fs::write(repository.path().join("docs/contract.md"), "second")?;
    let second = resolve_sources(draft("git-file", "docs/contract.md")?, &context)?;
    assert_eq!(
        second
            .sources()
            .first()
            .ok_or("missing fixture element")?
            .fingerprint()
            .as_str(),
        expected_fingerprint(b"second")
    );
    assert_ne!(
        first
            .sources()
            .first()
            .ok_or("missing fixture element")?
            .fingerprint(),
        second
            .sources()
            .first()
            .ok_or("missing fixture element")?
            .fingerprint()
    );
    Ok(())
}

#[test]
fn passes_the_resolved_git_path_to_the_tracking_probe_as_separate_argv()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let repository = tempfile::tempdir()?;
    fs::create_dir(repository.path().join("docs"))?;
    fs::write(repository.path().join("docs/contract.md"), BODY)?;
    let git_runner = FakeProcessRunner::with_responses([
        FakeResponse::success(format!("{}\n", repository.path().display())),
        FakeResponse::success(b"docs/contract.md\n".to_vec()),
    ]);
    let curl = FakeProcessRunner::default();
    let context = SourceContext::new(repository.path(), &git_runner, &curl);

    assert_eq!(
        verdict(resolve_sources(
            draft("git-file", "docs/contract.md")?,
            &context
        ))?,
        "valid"
    );

    assert_eq!(
        git_runner
            .calls()
            .get(1)
            .ok_or("missing fixture element")?
            .arguments,
        [
            OsString::from("-C"),
            repository.path().canonicalize()?.as_os_str().to_owned(),
            OsString::from("--literal-pathspecs"),
            OsString::from("ls-files"),
            OsString::from("--error-unmatch"),
            OsString::from("--full-name"),
            OsString::from("--"),
            OsString::from("docs/contract.md"),
        ]
    );
    Ok(())
}

#[test]
fn refuses_every_named_local_source_bypass_without_disclosing_the_locator()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let repository = tempfile::tempdir()?;
    fs::write(repository.path().join("untracked"), BODY)?;
    fs::write(repository.path().join("target"), BODY)?;
    symlink(
        repository.path().join("target"),
        repository.path().join("link"),
    )?;
    let missing = repository.path().join("sensitive-missing-file");
    let absolute_git_file = repository.path().join("target");
    let local_relative = "sensitive-relative-file";
    let cases = [
        ("untracked git file", "git-file", "untracked", "invalid"),
        (
            "parent traversal",
            "git-file",
            "../sensitive-file",
            "invalid",
        ),
        (
            "absolute git file",
            "git-file",
            absolute_git_file.to_str().ok_or("missing fixture value")?,
            "invalid",
        ),
        ("final symlink", "git-file", "link", "invalid"),
        (
            "relative local file",
            "local-file",
            local_relative,
            "invalid",
        ),
        (
            "missing local file",
            "local-file",
            missing.to_str().ok_or("missing fixture value")?,
            "invalid",
        ),
    ];

    for (bypass, kind, locator, expected) in cases {
        let responses = if bypass == "untracked git file" {
            vec![
                FakeResponse::success(format!("{}\n", repository.path().display())),
                FakeResponse::failure(1, Vec::new()),
            ]
        } else {
            vec![FakeResponse::failure(1, Vec::new())]
        };
        let git_runner = FakeProcessRunner::with_responses(responses);
        let curl = FakeProcessRunner::default();
        let context = SourceContext::new(repository.path(), &git_runner, &curl);
        let result = resolve_sources(draft(kind, locator)?, &context);
        let diagnostic = result
            .as_ref()
            .err()
            .ok_or("expected operation failure")?
            .to_string();
        assert_eq!(verdict(result)?, expected, "{bypass}");
        assert!(!diagnostic.contains(locator), "{bypass}");
    }
    Ok(())
}

#[test]
fn reports_unavailable_for_unreadable_oversized_or_inconclusive_local_sources()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let repository = tempfile::tempdir()?;
    let unreadable = repository.path().join("sensitive-unreadable");
    fs::write(&unreadable, BODY)?;
    fs::set_permissions(&unreadable, fs::Permissions::from_mode(0o000))?;
    let oversized = repository.path().join("sensitive-oversized");
    fs::write(&oversized, vec![b'x'; 1024 * 1024 + 1])?;
    let paths = [unreadable.as_path(), oversized.as_path()];

    for path in paths {
        let git_runner = FakeProcessRunner::default();
        let curl = FakeProcessRunner::default();
        let context = SourceContext::new(repository.path(), &git_runner, &curl);
        let result = resolve_sources(
            draft("local-file", path.to_str().ok_or("missing fixture value")?)?,
            &context,
        );
        let diagnostic = result
            .as_ref()
            .err()
            .ok_or("expected operation failure")?
            .to_string();
        assert_eq!(verdict(result)?, "unavailable");
        assert!(!diagnostic.contains(path.to_str().ok_or("missing fixture value")?));
    }
    fs::set_permissions(&unreadable, fs::Permissions::from_mode(0o600))?;

    for response in [
        FakeResponse::success(Vec::new()),
        FakeResponse::success(vec![0xff]),
        FakeResponse::failure(2, Vec::new()),
        FakeResponse::missing(),
    ] {
        let tracked = repository.path().join("tracked");
        fs::write(&tracked, BODY)?;
        let git_runner = FakeProcessRunner::with_responses([
            FakeResponse::success(format!("{}\n", repository.path().display())),
            response,
        ]);
        let curl = FakeProcessRunner::default();
        let context = SourceContext::new(repository.path(), &git_runner, &curl);
        assert_eq!(
            verdict(resolve_sources(draft("git-file", "tracked")?, &context))?,
            "unavailable"
        );
    }
    Ok(())
}

#[test]
fn fingerprints_an_absolute_regular_local_file()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let repository = tempfile::tempdir()?;
    let local = repository.path().join("local-proof");
    fs::write(&local, BODY)?;
    let git_runner = FakeProcessRunner::default();
    let curl = FakeProcessRunner::default();
    let context = SourceContext::new(repository.path(), &git_runner, &curl);

    let resolved = resolve_sources(
        draft("local-file", local.to_str().ok_or("missing fixture value")?)?,
        &context,
    )?;

    assert_eq!(
        resolved
            .sources()
            .first()
            .ok_or("missing fixture element")?
            .fingerprint()
            .as_str(),
        expected_fingerprint(BODY)
    );
    Ok(())
}

#[test]
fn refuses_every_named_initial_url_bypass_without_invoking_curl()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let repository = tempfile::tempdir()?;
    let cases = [
        ("HTTP scheme", "http://sensitive.example.test/proof"),
        ("missing host", "https://"),
        ("IPv4 literal", "https://192.0.2.10/proof"),
        ("IPv6 literal", "https://[2001:db8::1]/proof"),
        ("fragment", "https://sensitive.example.test/proof#private"),
    ];

    for (bypass, locator) in cases {
        let git_runner = FakeProcessRunner::default();
        let curl = FakeProcessRunner::default();
        let context = SourceContext::new(repository.path(), &git_runner, &curl);
        let result = resolve_sources(official_url_draft(locator)?, &context);
        let diagnostic = result
            .as_ref()
            .err()
            .ok_or("expected operation failure")?
            .to_string();
        assert_eq!(verdict(result)?, "invalid", "{bypass}");
        assert!(!diagnostic.contains(locator), "{bypass}");
        assert!(curl.calls().is_empty(), "{bypass}");
    }
    Ok(())
}

#[test]
fn refuses_an_official_url_without_a_user_decision_before_any_side_effect()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let repository = tempfile::tempdir()?;
    let temporary = tempfile::tempdir()?;
    let locator = "https://sensitive.example.test/proof";
    let git_runner = FakeProcessRunner::default();
    let curl =
        FakeProcessRunner::with_responses(
            [FakeResponse::success(SUCCESS_METADATA).with_body(BODY)],
        );
    let context = SourceContext::new(repository.path(), &git_runner, &curl)
        .with_temporary_directory(temporary.path());

    let result = resolve_sources(draft("official-url", locator)?, &context);
    let diagnostic = result
        .as_ref()
        .err()
        .ok_or("expected operation failure")?
        .to_string();

    assert_eq!(verdict(result)?, "invalid");
    assert!(!diagnostic.contains(locator));
    assert!(curl.calls().is_empty());
    assert!(fs::read_dir(temporary.path())?.next().is_none());
    Ok(())
}

#[test]
fn resolves_an_official_url_when_the_proof_contains_a_user_decision()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let repository = tempfile::tempdir()?;
    let locator = "https://docs.example.test/start";
    let git_runner = FakeProcessRunner::default();
    let curl =
        FakeProcessRunner::with_responses(
            [FakeResponse::success(SUCCESS_METADATA).with_body(BODY)],
        );
    let context = SourceContext::new(repository.path(), &git_runner, &curl);

    let resolved = resolve_sources(official_url_draft(locator)?, &context)?;

    assert_eq!(resolved.sources().len(), 2);
    assert_eq!(
        resolved
            .sources()
            .first()
            .ok_or("missing fixture element")?
            .kind(),
        agent_memory::SourceKind::OfficialUrl
    );
    assert_eq!(
        resolved
            .sources()
            .get(1)
            .ok_or("missing fixture element")?
            .kind(),
        agent_memory::SourceKind::UserDecision
    );
    Ok(())
}

#[test]
fn credential_url_bypasses_are_refused_before_source_resolution()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for (bypass, locator) in [
        ("username", "https://reader@sensitive.example.test/proof"),
        (
            "username and password",
            "https://reader:secret@sensitive.example.test/proof",
        ),
    ] {
        let automated = "  automated:\n    kind: source-fingerprint\n    expected: all-proof-sources-unchanged\n";
        let yaml = format!(
            "schema_version: 1\nkind: invariant\nstatement: A durable invariant remains independently useful.\nscope: project\nretrieval_terms:\n  - durable invariant\nproof:\n  summary: The source establishes the invariant.\n  sources:\n    - kind: official-url\n      locator: {}\noracle:\n{automated}  human_fallback:\n    question: Does the evidence still establish the invariant?\n    valid_when: The evidence remains observable.\n  outcomes:\n    valid: The evidence is unchanged.\n    invalidated: The evidence no longer establishes the invariant.\n",
            serde_json::to_string(locator)?
        );
        let parsed = parse_draft(yaml.as_bytes())?;
        let error = validate_draft(parsed, AdmissionAuthorization::ExplicitRequest)
            .err()
            .ok_or("expected operation failure")?;
        assert_eq!(error.code(), "sensitive_content", "{bypass}");
        assert!(!error.to_string().contains(locator), "{bypass}");
    }
    Ok(())
}

#[test]
fn invokes_curl_with_the_closed_https_redirect_time_and_size_policy()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let repository = tempfile::tempdir()?;
    let temporary = tempfile::tempdir()?;
    let locator = "https://docs.example.test/start";
    let git_runner = FakeProcessRunner::default();
    let curl =
        FakeProcessRunner::with_responses(
            [FakeResponse::success(SUCCESS_METADATA).with_body(BODY)],
        );
    let context = SourceContext::new(repository.path(), &git_runner, &curl)
        .with_temporary_directory(temporary.path());

    let resolved = resolve_sources(official_url_draft(locator)?, &context)?;

    assert_eq!(
        resolved
            .sources()
            .first()
            .ok_or("missing fixture element")?
            .fingerprint()
            .as_str(),
        expected_fingerprint(BODY)
    );
    let calls = curl.calls();
    assert_eq!(calls.len(), 1);
    let arguments = &calls.first().ok_or("missing fixture element")?.arguments;
    assert_eq!(arguments.first(), Some(&OsString::from("--disable")));
    for pair in [
        ["--max-redirs", "5"],
        ["--proto", "=https"],
        ["--proto-redir", "=https"],
        ["--connect-timeout", "5"],
        ["--max-time", "15"],
        ["--max-filesize", "1048576"],
    ] {
        assert!(
            arguments
                .windows(2)
                .any(|window| window == pair.map(OsString::from))
        );
    }
    for flag in ["--silent", "--show-error", "--fail-with-body", "--location"] {
        assert!(arguments.contains(&OsString::from(flag)));
    }
    assert_eq!(arguments.last(), Some(&OsString::from(locator)));
    let output_files = curl.output_files();
    assert_eq!(
        output_files.first().ok_or("missing fixture element")?.1,
        0o600
    );
    assert!(
        !output_files
            .first()
            .ok_or("missing fixture element")?
            .0
            .exists()
    );
    assert!(fs::read_dir(temporary.path())?.next().is_none());
    Ok(())
}

struct RemainingProcessRunner {
    inner: FakeProcessRunner,
    remaining: Duration,
}

impl ProcessRunner for RemainingProcessRunner {
    fn run(
        &self,
        program: &OsStr,
        arguments: &[OsString],
        current_directory: Option<&Path>,
    ) -> io::Result<ProcessOutput> {
        self.inner.run(program, arguments, current_directory)
    }

    fn remaining_time(&self) -> Option<Duration> {
        Some(self.remaining)
    }
}

#[test]
fn caps_curl_to_the_remaining_shared_deadline()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let repository = tempfile::tempdir()?;
    let temporary = tempfile::tempdir()?;
    let git_runner = FakeProcessRunner::default();
    let curl = RemainingProcessRunner {
        inner: FakeProcessRunner::with_responses([
            FakeResponse::success(SUCCESS_METADATA).with_body(BODY)
        ]),
        remaining: Duration::from_millis(2750),
    };
    let context = SourceContext::new(repository.path(), &git_runner, &curl)
        .with_temporary_directory(temporary.path());

    resolve_sources(
        official_url_draft("https://docs.example.test/start")?,
        &context,
    )?;

    let calls = curl.inner.calls();
    let arguments = &calls.first().ok_or("missing fixture element")?.arguments;
    assert!(
        arguments
            .windows(2)
            .any(|window| { window == [OsString::from("--max-time"), OsString::from("2.750")] })
    );
    Ok(())
}

#[test]
fn accepts_an_https_redirect_and_refuses_an_http_redirect()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let repository = tempfile::tempdir()?;
    let cases = [
        (SUCCESS_METADATA, "valid"),
        (
            "200\nhttp://sensitive.example.test/final\n203.0.113.10\n",
            "invalid",
        ),
    ];

    for (metadata, expected) in cases {
        let git_runner = FakeProcessRunner::default();
        let curl =
            FakeProcessRunner::with_responses([FakeResponse::success(metadata).with_body(BODY)]);
        let context = SourceContext::new(repository.path(), &git_runner, &curl);
        let result = resolve_sources(
            official_url_draft("https://docs.example.test/start")?,
            &context,
        );
        assert_eq!(verdict(result)?, expected);
    }
    Ok(())
}

#[test]
fn maps_http_and_transport_failures_without_disclosing_the_locator()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let repository = tempfile::tempdir()?;
    let locator = "https://sensitive.example.test/private-proof";
    let cases = [
        (
            "404",
            FakeResponse::failure(
                22,
                "404\nhttps://sensitive.example.test/private-proof\n203.0.113.10\n",
            ),
            "invalid",
        ),
        (
            "410",
            FakeResponse::failure(
                22,
                "410\nhttps://sensitive.example.test/private-proof\n203.0.113.10\n",
            ),
            "invalid",
        ),
        (
            "429",
            FakeResponse::failure(
                22,
                "429\nhttps://sensitive.example.test/private-proof\n203.0.113.10\n",
            ),
            "unavailable",
        ),
        (
            "5xx",
            FakeResponse::failure(
                22,
                "503\nhttps://sensitive.example.test/private-proof\n203.0.113.10\n",
            ),
            "unavailable",
        ),
        ("DNS", FakeResponse::failure(6, Vec::new()), "unavailable"),
        (
            "timeout",
            FakeResponse::failure(28, Vec::new()),
            "unavailable",
        ),
        (
            "TLS handshake",
            FakeResponse::failure(35, Vec::new()),
            "unavailable",
        ),
        (
            "TLS certificate",
            FakeResponse::failure(60, Vec::new()),
            "unavailable",
        ),
        (
            "maximum body",
            FakeResponse::failure(63, Vec::new()),
            "unavailable",
        ),
        ("curl absent", FakeResponse::missing(), "unavailable"),
    ];

    for (failure, response, expected) in cases {
        let temporary = tempfile::tempdir()?;
        let git_runner = FakeProcessRunner::default();
        let curl = FakeProcessRunner::with_responses([response]);
        let context = SourceContext::new(repository.path(), &git_runner, &curl)
            .with_temporary_directory(temporary.path());
        let result = resolve_sources(official_url_draft(locator)?, &context);
        let diagnostic = result
            .as_ref()
            .err()
            .ok_or("expected operation failure")?
            .to_string();
        assert_eq!(verdict(result)?, expected, "{failure}");
        assert!(!diagnostic.contains(locator), "{failure}");
        assert!(
            fs::read_dir(temporary.path())?.next().is_none(),
            "{failure}"
        );
    }
    Ok(())
}

#[test]
fn fails_closed_on_malformed_curl_metadata_or_oversized_written_body()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let repository = tempfile::tempdir()?;
    let cases = [
        FakeResponse::success(Vec::new()).with_body(BODY),
        FakeResponse::success(b"200\nhttps://docs.example.test/final\nnot-an-ip\n".to_vec())
            .with_body(BODY),
        FakeResponse::success(b"twenty\nhttps://docs.example.test/final\n203.0.113.10\n".to_vec())
            .with_body(BODY),
        FakeResponse::success(
            b"200\nhttps://docs.example.test/final\n203.0.113.10\nextra\n".to_vec(),
        )
        .with_body(BODY),
        FakeResponse::success(SUCCESS_METADATA).with_body(vec![b'x'; 1024 * 1024 + 1]),
    ];

    for response in cases {
        let git_runner = FakeProcessRunner::default();
        let curl = FakeProcessRunner::with_responses([response]);
        let context = SourceContext::new(repository.path(), &git_runner, &curl);
        assert_eq!(
            verdict(resolve_sources(
                official_url_draft("https://docs.example.test/start")?,
                &context
            ))?,
            "unavailable"
        );
    }
    Ok(())
}

#[test]
fn fingerprints_a_user_decision_without_a_process_or_transcript()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let repository = tempfile::tempdir()?;
    let locator = "decision:official-domain:docs.example.test";
    let git_runner = FakeProcessRunner::default();
    let curl = FakeProcessRunner::default();
    let context = SourceContext::new(repository.path(), &git_runner, &curl);

    let resolved = resolve_sources(draft("user-decision", locator)?, &context)?;

    assert_eq!(
        resolved
            .sources()
            .first()
            .ok_or("missing fixture element")?
            .fingerprint()
            .as_str(),
        expected_fingerprint(locator.as_bytes())
    );
    assert!(git_runner.calls().is_empty());
    assert!(curl.calls().is_empty());
    Ok(())
}

#[test]
fn user_scope_supports_non_git_source_kinds() -> Result<(), Box<dyn std::error::Error + Send + Sync>>
{
    let repository = tempfile::tempdir()?;
    let local = repository.path().join("local-proof");
    fs::write(&local, BODY)?;
    let git_runner = FakeProcessRunner::default();
    let curl =
        FakeProcessRunner::with_responses(
            [FakeResponse::success(SUCCESS_METADATA).with_body(BODY)],
        );
    let context = SourceContext::new(repository.path(), &git_runner, &curl);
    let official = [
        ("official-url", "https://docs.example.test/start"),
        (
            "user-decision",
            "The user designated this proof as official.",
        ),
    ];

    for sources in [
        vec![("local-file", local.to_str().ok_or("missing fixture value")?)],
        vec![("user-decision", "decision:user-proof")],
        official.to_vec(),
    ] {
        resolve_sources(draft_with_scope_sources("user", &sources)?, &context)?;
    }
    Ok(())
}

#[test]
fn resolved_draft_can_recheck_sources_before_admission_commit()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let repository = tempfile::tempdir()?;
    let local = repository.path().join("proof");
    fs::write(&local, b"before")?;
    let git_runner = FakeProcessRunner::default();
    let curl = FakeProcessRunner::default();
    let context = SourceContext::new(repository.path(), &git_runner, &curl);
    let resolved = resolve_sources(
        draft("local-file", local.to_str().ok_or("missing fixture value")?)?,
        &context,
    )?;
    fs::write(&local, b"after")?;

    let error = resolved
        .recheck_sources(&context)
        .err()
        .ok_or("expected operation failure")?;

    assert_eq!(error.code(), "source_changed");
    assert!(
        !error
            .to_string()
            .contains(local.to_str().ok_or("missing fixture value")?)
    );
    Ok(())
}
