use super::support::*;
use agent_memory::{
    Index, OmissionEffect, RetrievalContext, RetrievalRequest, SourceContext, retrieve,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;

#[test]
fn user_git_yaml_is_omitted_before_cross_repository_proof_resolution()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let (root, store) = open_store(fixture.path())?;
    let repository_a = fixture.path().join("repository-a");
    let repository_b = fixture.path().join("repository-b");
    let proof = b"same repository-bound proof";
    for repository in [&repository_a, &repository_b] {
        fs::create_dir_all(repository.join("docs"))?;
        fs::write(repository.join("docs/proof.txt"), proof)?;
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
    )?)?
    .replace(&fingerprint('a'), &expected_fingerprint)
    .into_bytes();
    let path = write_user_entry(&root, 'a', &yaml)?;
    let legacy_index = legacy_index_bytes(&path)?;
    fs::write(root.join("index.json"), &legacy_index)?;
    assert_eq!(fs::read(root.join("index.json"))?, legacy_index);
    let metadata = fs::metadata(&path)?;
    let before = (
        fs::read(&path)?,
        metadata.permissions().mode() & 0o777,
        metadata.ino(),
    );
    let key = project_key(&repository_b)?;
    let loaded = Index::load_or_rebuild(&store)?;
    let selection = select(&store, &key, 5)?;
    let runner = FakeProcessRunner::with_responses([
        FakeResponse::success(format!("{}\n", repository_b.display())),
        FakeResponse::success("docs/proof.txt\n"),
    ]);
    let sources = SourceContext::new(&repository_b, &runner, &runner);
    let clock = FixedClock::at("2026-08-28T01:00:00Z")?;

    let report = retrieve(
        RetrievalRequest::new(&selection, &key, true),
        &RetrievalContext::new(&store, &clock, &sources, environment()),
    );

    let calls = runner.calls();
    assert!(calls.is_empty(), "{calls:?}");
    assert_eq!(
        (
            loaded.rebuilt,
            selection.selected.len(),
            selection.diagnostics.len(),
        ),
        (true, 0, 1)
    );
    assert_eq!(
        selection
            .diagnostics
            .first()
            .ok_or("missing fixture element")?
            .check,
        "source_invalid"
    );
    assert!(report.injected.is_empty());
    assert_eq!(report.omitted.len(), 1);
    let omission = report.omitted.first().ok_or("missing fixture element")?;
    assert_eq!(omission.code, "source_invalid");
    assert_eq!(omission.effect, OmissionEffect::NotApplied);
    assert!(omission.question.is_none());
    let index: serde_json::Value = serde_json::from_slice(&fs::read(root.join("index.json"))?)?;
    assert_eq!(
        index
            .get("schema_version")
            .ok_or("missing fixture element")?,
        2
    );
    assert_eq!(
        index.get("entries").ok_or("missing fixture element")?,
        &serde_json::json!([])
    );
    let metadata = fs::metadata(&path)?;
    assert_eq!(
        (
            fs::read(&path)?,
            metadata.permissions().mode() & 0o777,
            metadata.ino(),
        ),
        before
    );
    let rendered = format!("{report:?}");
    assert!(!rendered.contains("docs/proof.txt"));
    assert!(!rendered.contains(repository_a.to_str().ok_or("missing fixture value")?));
    assert!(!rendered.contains(repository_b.to_str().ok_or("missing fixture value")?));
    Ok(())
}

#[test]
fn user_non_git_sources_remain_indexed_and_retrievable()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let (root, store) = open_store(fixture.path())?;
    let yaml = entry_yaml(
        'b',
        "invariant",
        &[
            SourceFixture {
                kind: "local-file",
                locator: "/private/proof.txt",
                fingerprint: 'b',
            },
            SourceFixture {
                kind: "official-url",
                locator: "https://docs.example.test/proof",
                fingerprint: 'c',
            },
            SourceFixture {
                kind: "user-decision",
                locator: "decision:proof-accepted",
                fingerprint: 'd',
            },
        ],
    )?;
    write_user_entry(&root, 'b', &yaml)?;
    let key = project_key(fixture.path())?;
    let selection = select(&store, &key, 5)?;
    let resolver = FakeResolver::with_responses([valid('b'), valid('c'), valid('d')]);
    let clock = FixedClock::at("2026-08-28T01:00:00Z")?;

    let report = resolver.checked(|checked_resolver| {
        Ok(retrieve(
            RetrievalRequest::new(&selection, &key, true),
            &RetrievalContext::new(&store, &clock, checked_resolver, environment()),
        ))
    })?;

    assert_eq!(selection.selected.len(), 1);
    assert!(selection.diagnostics.is_empty());
    assert_eq!(report.injected.len(), 1);
    assert!(report.omitted.is_empty());
    Ok(())
}

fn legacy_index_bytes(yaml: &Path) -> FixtureResult<Vec<u8>> {
    let metadata = fs::metadata(yaml)?;
    let id = yaml
        .file_stem()
        .and_then(std::ffi::OsStr::to_str)
        .ok_or("missing fixture identifier")?;
    let path = format!("entries/user/{id}.yaml");
    let inventory = [LegacyInventoryItem {
        path: path.clone(),
        length: metadata.len(),
        modified_ns: modified_ns(&metadata),
    }];
    let inventory_digest = format!(
        "sha256:{:x}",
        Sha256::digest(serde_json::to_vec(&inventory)?)
    );
    let document = serde_json::json!({
        "schema_version": 1,
        "inventory_digest": inventory_digest,
        "entries": [{
            "id": id,
            "kind": "invariant",
            "status": "active",
            "scope": { "type": "user" },
            "retrieval_terms": ["durable memory"],
            "statement_tokens": ["a", "durable", "memory", "statement"],
            "summary": "Durable proof summary.",
            "path": path,
            "length": metadata.len(),
            "modified_ns": modified_ns(&metadata),
        }],
        "diagnostics": { "user": [], "projects": {} },
    });
    Ok(format!("{document:#}\n").into_bytes())
}

fn modified_ns(metadata: &fs::Metadata) -> i64 {
    metadata.mtime() * 1_000_000_000 + metadata.mtime_nsec()
}

#[derive(Serialize)]
struct LegacyInventoryItem {
    path: String,
    length: u64,
    modified_ns: i64,
}
