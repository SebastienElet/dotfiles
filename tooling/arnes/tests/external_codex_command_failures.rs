#[path = "support/codex.rs"]
pub mod codex_support;
#[path = "support/skills.rs"]
pub mod skill_support;
pub mod support;

use codex_support::{install_script, marketplace, plugin};
use serde_json::json;
use skill_support::{MANIFEST, configured_fixture, run};
use std::time::{Duration, Instant};

fn fixture() -> support::Fixture {
    let fixture = configured_fixture();
    let manifest = MANIFEST.replacen(
        "resources:",
        "external:\n  roots: []\n  plugins:\n    - { agent: codex, scope: user, id: demo@marketplace }\n  skills: []\nresources:",
        1,
    );
    fixture.write_home(".arnes.yaml", &manifest);
    fixture.write_home(
        ".codex/config.toml",
        "[plugins.\"demo@marketplace\"]\nenabled = true\n",
    );
    fixture
}

fn diagnose(fixture: &support::Fixture) -> (i32, String, String) {
    run(
        fixture,
        &["doctor", "skills", "--agent", "codex", "--scope", "user"],
    )
}

#[test]
fn resolver_spawn_and_exit_failures_remain_visible() {
    let missing = fixture();
    let (code, stdout, _) = diagnose(&missing);
    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("Codex marketplace resolver could not be started"));

    let failed = fixture();
    install_script(
        &failed,
        "#!/bin/sh\nprintf 'resolver failed' >&2\nexit 23\n",
    );
    let (code, stdout, _) = diagnose(&failed);
    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("Codex marketplace resolver exited with status 23"));
}

#[test]
fn resolver_failure_is_visible_when_user_config_has_no_plugins() {
    let fixture = fixture();
    fixture.write_home(".codex/config.toml", "");
    install_script(&fixture, "#!/bin/sh\nexit 23\n");

    let (code, stdout, _) = diagnose(&fixture);

    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("UNSUPPORTED external codex user plugin resolution"));
    assert!(stdout.contains("resolver exited with status 23"));
}

#[test]
fn resolver_timeout_is_bounded_and_visible() {
    let fixture = fixture();
    install_script(&fixture, "#!/bin/sh\n/bin/sleep 10 &\nwait\n");
    assert_timeout(&fixture);
}

#[test]
fn resolver_descendant_cannot_hold_output_open_past_timeout() {
    let fixture = fixture();
    install_script(
        &fixture,
        "#!/bin/sh\nif [ \"$2\" = \"marketplace\" ]; then /bin/sleep 10 & printf '{\"marketplaces\":[]}\\n'; exit 0; fi\nprintf '{\"installed\":[]}\\n'\n",
    );
    assert_timeout(&fixture);
}

fn assert_timeout(fixture: &support::Fixture) {
    let started = Instant::now();
    let (code, stdout, _) = diagnose(fixture);
    assert_eq!(code, 0, "{stdout}");
    assert!(started.elapsed() < Duration::from_secs(7));
    assert!(stdout.contains("Codex marketplace resolver timed out"));
}

#[test]
fn oversized_resolver_output_is_bounded_and_visible() {
    let fixture = fixture();
    fixture.write_home(
        ".codex-test-marketplaces.json",
        &"x".repeat(1024 * 1024 + 1),
    );
    fixture.write_home(".codex-test-plugins.json", r#"{"installed":[]}"#);
    install_script(
        &fixture,
        "#!/bin/sh\nif [ \"$2\" = \"marketplace\" ]; then file=\"$HOME/.codex-test-marketplaces.json\"; else file=\"$HOME/.codex-test-plugins.json\"; fi\nwhile IFS= read -r line || [ -n \"$line\" ]; do printf '%s\\n' \"$line\"; done < \"$file\"\n",
    );

    let (code, stdout, _) = diagnose(&fixture);

    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("Codex marketplace resolver exceeded its output limit"));
}

#[test]
fn resolver_runs_outside_the_callers_project_context() {
    let fixture = fixture();
    let root = fixture.home().join(".codex/.tmp/plugins");
    let path = root.join("plugins/demo");
    fixture.write_home(
        ".codex/plugins/cache/marketplace/demo/revision/.codex-plugin/plugin.json",
        r#"{"name":"demo","version":"1.0.0"}"#,
    );
    fixture.write_home(
        ".codex-test-marketplaces.json",
        &json!({"marketplaces": [marketplace("marketplace", &root)]}).to_string(),
    );
    fixture.write_home(
        ".codex-test-plugins.json",
        &json!({"installed": [plugin("demo@marketplace", "marketplace", "revision", true, &path)], "available": []}).to_string(),
    );
    install_script(
        &fixture,
        "#!/bin/sh\n[ -f .codex-test-marketplaces.json ] || { printf 'project override leaked'; exit 0; }\nif [ \"$2\" = \"marketplace\" ]; then file=\"$HOME/.codex-test-marketplaces.json\"; else file=\"$HOME/.codex-test-plugins.json\"; fi\nwhile IFS= read -r line || [ -n \"$line\" ]; do printf '%s\\n' \"$line\"; done < \"$file\"\n",
    );

    let (code, stdout, _) = run(
        &fixture,
        &[
            "doctor",
            "skills",
            "--agent",
            "codex",
            "--scope",
            "user",
            "--verbose",
        ],
    );

    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("demo@marketplace@1.0.0"), "{stdout}");
}
