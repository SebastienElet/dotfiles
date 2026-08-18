use super::*;
use crate::manifest;

#[test]
fn unsupported_bindings_do_not_initialize_topology() {
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
    )
    .unwrap();
    let roots = Roots::new("/missing/repository", "/missing/home");

    let diagnostics = diagnose_with_tracker(
        &roots,
        &manifest,
        Some(Agent::Cursor),
        Some(Scope::Project),
        |_| panic!("topology initialized for an unsupported binding"),
    );

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].state, State::Unsupported);
}
