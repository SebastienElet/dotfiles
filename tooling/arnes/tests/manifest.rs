use arnes::manifest;
use std::fs;
use std::path::Path;

fn fixture(name: &str) -> String {
    fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/manifest")
            .join(name),
    )
    .unwrap()
}

fn error(name: &str) -> String {
    match manifest::parse(&fixture(name), Path::new("/fixture/home")) {
        Ok(_) => panic!("{name} unexpectedly passed validation"),
        Err(error) => error.to_string(),
    }
}

#[test]
fn valid_manifest_models_rooted_user_and_project_resources() {
    let manifest = manifest::parse(&fixture("valid.yaml"), Path::new("/fixture/home")).unwrap();

    assert_eq!(
        manifest.repository_root(),
        Path::new("/fixture/home/.dotfiles")
    );
    assert_eq!(
        manifest.resource_paths().collect::<Vec<_>>(),
        [
            (
                Path::new("/fixture/home/.dotfiles/harness/AGENTS.md").to_owned(),
                Path::new("/fixture/home/.claude/CLAUDE.md").to_owned(),
            ),
            (
                Path::new("/fixture/home/.dotfiles/harness/AGENTS.md").to_owned(),
                Path::new("/fixture/home/.dotfiles/CLAUDE.md").to_owned(),
            ),
        ]
    );
}

#[test]
fn unsupported_version_precedes_resource_validation() {
    assert_eq!(
        error("unsupported-version.yaml"),
        "version: unsupported version 2; expected 1"
    );
}

#[test]
fn required_fields_are_reported() {
    assert_eq!(
        error("missing-field.yaml"),
        "manifest: missing field `resources`"
    );
}

#[test]
fn duplicate_identifiers_are_rejected() {
    for (fixture, expected) in [
        (
            "duplicate-agent.yaml",
            "agents[1].id: duplicate agent identifier",
        ),
        (
            "duplicate-resource.yaml",
            "resources[1].id: duplicates resources[0].id",
        ),
        (
            "duplicate-scope.yaml",
            "agents[0].scopes[1]: duplicate scope identifier",
        ),
    ] {
        assert_eq!(error(fixture), expected);
    }
}

#[test]
fn duplicate_destinations_are_rejected() {
    for fixture in [
        "duplicate-destination.yaml",
        "resolved-duplicate-destination.yaml",
    ] {
        assert_eq!(
            error(fixture),
            "resources[1].destination: duplicates resources[0].destination"
        );
    }
}

#[test]
fn paths_cannot_escape_their_declared_roots() {
    for (fixture, field) in [
        ("absolute-path.yaml", "resources[0].source.path"),
        ("parent-path.yaml", "resources[0].destination.path"),
    ] {
        assert_eq!(
            error(fixture),
            format!("{field}: path must stay within its declared root")
        );
    }
}

#[test]
fn incompatible_declarations_are_rejected() {
    for (fixture, expected) in [
        (
            "undeclared-agent.yaml",
            "resources[0].agent: agent is not declared",
        ),
        (
            "undeclared-scope.yaml",
            "resources[0].scope: scope is not declared for this agent",
        ),
        (
            "home-source.yaml",
            "resources[0].source.root: resource sources must be repository-relative",
        ),
        (
            "wrong-destination-root.yaml",
            "resources[0].destination.root: destination root is incompatible with resource scope",
        ),
        (
            "resolved-source-destination.yaml",
            "resources[0].destination: source and destination must differ",
        ),
    ] {
        assert_eq!(error(fixture), expected);
    }
}

#[test]
fn secret_fields_are_rejected_without_exposing_values() {
    let error = error("secret.yaml");

    assert_eq!(
        error,
        "resources[0].api_token: secret values are not allowed"
    );
    assert!(!error.contains("super-secret-value"));
}

#[test]
fn inline_resource_contents_are_rejected_without_exposing_values() {
    let error = error("inline-content.yaml");

    assert_eq!(
        error,
        "resources[0].content: resources[0]: unknown field `content`, expected one of `id`, `kind`, `agent`, `scope`, `source`, `destination` at line 11 column 5"
    );
    assert!(!error.contains("inline instruction text"));
}
