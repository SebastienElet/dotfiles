use super::*;
use crate::manifest;

#[test]
fn unsupported_bindings_do_not_initialize_topology() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = manifest::parse(
        "version: 1
agents:
  - id: cursor
    scopes: [project]
prompts:
  - id: deploy
    source: { root: repository, path: prompt.md }
    includes: []
    variables: []
    projections: []
commands:
  - name: deploy
    description: Deploy safely
    prompt: deploy
    bindings:
      - { agent: cursor, scope: project }
resources: []
",
    )?;
    let roots = Roots::new("/missing/repository", "/missing/home");

    let mut initialized = false;
    let diagnostics = diagnose_with_tracker(
        &roots,
        &manifest,
        Some(Agent::Cursor),
        Some(Scope::Project),
        |scopes| {
            initialized = true;
            ProjectionTracker::new_for_scopes(&roots, &manifest, scopes)
        },
    );

    assert_eq!(diagnostics.len(), 1);
    assert!(!initialized);
    assert_eq!(
        diagnostics.first().ok_or("missing diagnostic")?.state,
        State::Unsupported
    );
    Ok(())
}
