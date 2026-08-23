mod support;

use support::Fixture;

fn fixture() -> Fixture {
    let fixture = Fixture::new();
    fixture.write_home(
        ".arnes.yaml",
        "version: 1
agents:
  - id: claude
    scopes: [user]
prompts:
  - id: deploy
    source: { root: repository, path: deploy.md }
    includes: []
    variables: []
    projections:
      - agent: claude
        scope: user
        representation: rendered
        destination: { root: home, path: .claude/commands/deploy.md }
commands:
  - name: deploy
    description: Deploy safely
    prompt: deploy
    bindings:
      - { agent: claude, scope: user }
resources: []
",
    );
    let contents = "---\ndescription: Deploy safely\n---\nDeploy now\n";
    fixture.write_repository("deploy.md", contents);
    fixture.write_home(".claude/commands/deploy.md", contents);
    fixture
}

#[test]
fn commands_hide_healthy_details_until_verbose() {
    let fixture = fixture();
    assert!(fixture.home().is_dir());
    assert!(fixture.repository().is_dir());
    let before = fixture.snapshot();
    let normal = fixture.command(["doctor", "commands", "--agent", "claude"]);
    let verbose = fixture.command(["doctor", "commands", "--agent", "claude", "-v"]);
    let normal = String::from_utf8(normal.stdout).unwrap();
    let verbose = String::from_utf8(verbose.stdout).unwrap();

    assert!(normal.starts_with("Commands · user scope · claude agent\n✓ 1 healthy\n"));
    assert!(!normal.contains("deploy · current"));
    assert!(verbose.contains("healthy     deploy · current"));
    assert_eq!(fixture.snapshot(), before);
}
