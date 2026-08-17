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
    match manifest::parse(&fixture(name)) {
        Ok(_) => panic!("{name} unexpectedly passed validation"),
        Err(error) => error.to_string(),
    }
}

#[test]
fn valid_manifest_models_rooted_user_and_project_resources() {
    manifest::parse(&fixture("valid.yaml")).unwrap();
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
    assert_eq!(
        error("duplicate-destination.yaml"),
        "resources[1].destination: duplicates resources[0].destination"
    );
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
        "resources[0].content: resources[0]: unknown field `content`, expected one of `id`, `kind`, `agent`, `scope`, `layout`, `source`, `destination` at line 10 column 5"
    );
    assert!(!error.contains("inline instruction text"));
}

#[test]
fn skill_declarations_are_normalized_and_unambiguous() {
    for (fixture, expected) in [
        (
            "duplicate-skill.yaml",
            "skills[1].slug: duplicate skill identifier",
        ),
        (
            "duplicate-skill-installation.yaml",
            "skills[0].installations[1]: duplicate skill installation",
        ),
        (
            "empty-skill-installations.yaml",
            "skills[0].installations: at least one leaf installation is required",
        ),
        (
            "duplicate-skill-projection.yaml",
            "resources[1].agent: duplicates resources[0] skill projection",
        ),
        (
            "invalid-skill-slug.yaml",
            "skills[0].slug: must be one relative path component",
        ),
        (
            "aliased-skill-slug.yaml",
            "skills[1].slug: must be one relative path component",
        ),
        (
            "layout-on-instructions.yaml",
            "resources[0].layout: layout is only valid for skill projections",
        ),
        (
            "missing-skill-layout.yaml",
            "resources[0].layout: skill projection layout is required",
        ),
        (
            "skill-installation-on-root.yaml",
            "skills[0].installations[0]: has no leaves skill projection",
        ),
    ] {
        assert_eq!(error(fixture), expected);
    }
}
