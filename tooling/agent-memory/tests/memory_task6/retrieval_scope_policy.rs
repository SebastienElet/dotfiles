use super::support::*;
use agent_memory::{OmissionEffect, RetrievalContext, RetrievalRequest, SourceContext, retrieve};
use sha2::{Digest, Sha256};
use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};

#[test]
fn user_git_yaml_is_omitted_before_cross_repository_proof_resolution() {
    let fixture = tempfile::tempdir().unwrap();
    let (root, store) = open_store(fixture.path());
    let repository_a = fixture.path().join("repository-a");
    let repository_b = fixture.path().join("repository-b");
    let proof = b"same repository-bound proof";
    for repository in [&repository_a, &repository_b] {
        fs::create_dir_all(repository.join("docs")).unwrap();
        fs::write(repository.join("docs/proof.txt"), proof).unwrap();
    }
    let expected_fingerprint = format!("sha256:{:x}", Sha256::digest(proof));
    let yaml = String::from_utf8(entry_yaml(
        'a',
        "invariant",
        &[SourceFixture {
            kind: "git-file",
            locator: "docs/proof.txt",
            fingerprint: 'a',
        }],
    ))
    .unwrap()
    .replace(&fingerprint('a'), &expected_fingerprint)
    .into_bytes();
    let path = write_user_entry(&root, 'a', &yaml);
    let metadata = fs::metadata(&path).unwrap();
    let before = (
        fs::read(&path).unwrap(),
        metadata.permissions().mode() & 0o777,
        metadata.ino(),
    );
    let key = project_key(&repository_b);
    let selection = select(&store, &key, 5);
    let runner = FakeProcessRunner::with_responses([
        FakeResponse::success(format!("{}\n", repository_b.display())),
        FakeResponse::success("docs/proof.txt\n"),
    ]);
    let sources = SourceContext::new(&repository_b, &runner, &runner);
    let clock = FixedClock::at("2026-08-28T01:00:00Z");

    let report = retrieve(
        RetrievalRequest::new(&selection, &key, true),
        RetrievalContext::new(&store, &clock, &sources, environment()),
    );

    let calls = runner.calls();
    assert!(calls.is_empty(), "{calls:?}");
    assert!(selection.selected.is_empty());
    assert_eq!(selection.diagnostics.len(), 1);
    assert_eq!(selection.diagnostics[0].check, "source_invalid");
    assert!(report.injected.is_empty());
    assert_eq!(report.omitted.len(), 1);
    assert_eq!(report.omitted[0].code, "source_invalid");
    assert_eq!(report.omitted[0].effect, OmissionEffect::NotApplied);
    assert!(report.omitted[0].question.is_none());
    let index: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("index.json")).unwrap()).unwrap();
    assert_eq!(index["entries"], serde_json::json!([]));
    let metadata = fs::metadata(&path).unwrap();
    assert_eq!(
        (
            fs::read(&path).unwrap(),
            metadata.permissions().mode() & 0o777,
            metadata.ino(),
        ),
        before
    );
    let rendered = format!("{report:?}");
    assert!(!rendered.contains("docs/proof.txt"));
    assert!(!rendered.contains(repository_a.to_str().unwrap()));
    assert!(!rendered.contains(repository_b.to_str().unwrap()));
}
