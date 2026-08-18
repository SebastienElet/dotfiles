#[path = "support/skills.rs"]
pub mod skill_support;
pub mod support;

use skill_support::{MANIFEST, configured_fixture, run};
use std::os::unix::fs::symlink;

fn manifest(allow: bool) -> String {
    let skills = if allow {
        "    - { agent: codex, scope: user, origin: managed, slug: ponytail }\n"
    } else {
        ""
    };
    MANIFEST.replacen(
        "resources:",
        &format!("external:\n  roots: []\n  plugins: []\n  skills:\n{skills}resources:"),
        1,
    )
}

#[test]
fn allowed_external_managed_skill_is_healthy_and_not_adopted() {
    let fixture = configured_fixture();
    fixture.write_home(".arnes.yaml", &manifest(true));
    fixture.write_home(
        ".agents/skills/ponytail/SKILL.md",
        "[missing](references/missing.md)\n",
    );

    let (code, stdout, _) = run(
        &fixture,
        &[
            "doctor", "skills", "--agent", "codex", "--scope", "user", "-v",
        ],
    );

    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("codex user external skills"));
    assert!(stdout.contains("ponytail · managed · enabled · healthy · allowed"));
    assert!(!stdout.contains("local resource"));
}

#[test]
fn unexpected_external_managed_skill_is_policy_drift() {
    let fixture = configured_fixture();
    fixture.write_home(".arnes.yaml", &manifest(false));
    fixture.write_home(".agents/skills/ponytail/SKILL.md", "ponytail\n");

    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "codex", "--scope", "user"],
    );

    assert_eq!(code, 1, "{stdout}");
    assert!(stdout.contains("· unexpected"));
}

#[test]
fn absent_allowed_external_managed_skill_does_not_drift() {
    let fixture = configured_fixture();
    fixture.write_home(".arnes.yaml", &manifest(true));

    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "codex", "--scope", "user"],
    );

    assert_eq!(code, 0, "{stdout}");
    assert!(!stdout.contains("skill ponytail"));
}

#[test]
fn broken_allowed_external_managed_skill_is_topological_error() {
    let fixture = configured_fixture();
    fixture.write_home(".arnes.yaml", &manifest(true));
    symlink("missing", fixture.home().join(".agents/skills/ponytail")).unwrap();

    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "codex", "--scope", "user"],
    );

    assert_eq!(code, 2, "{stdout}");
    assert!(stdout.contains("· broken · allowed"));
}
