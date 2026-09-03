#[path = "support/codex.rs"]
pub mod codex_support;
#[path = "support/skills.rs"]
pub mod skill_support;
pub mod support;

use serde_json::json;
use skill_support::run;
use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use support::Fixture;

fn fixture(config: &str) -> Fixture {
    let fixture = Fixture::new();
    fixture.write_home(
        ".arnes.yaml",
        "version: 1\nagents:\n  - { id: codex, scopes: [user] }\nskills: []\nresources: []\nexternal:\n  roots:\n    - { agent: codex, scope: user, origin: system, location: { root: home, path: .codex/skills/.system } }\n  plugins: []\n  skills:\n    - { agent: codex, scope: user, origin: system, slug: folder }\n",
    );
    fixture.write_home(
        ".codex/skills/.system/folder/SKILL.md",
        "---\nname: actual-name\ndescription: Example\n---\nSkill\n",
    );
    let path = fixture.home().join(".codex/skills/.system/folder/SKILL.md");
    fixture.write_home(
        ".codex/config.toml",
        &config.replace("SKILL_PATH", path.to_str().unwrap()),
    );
    codex_support::install(
        &fixture,
        json!({"marketplaces": []}),
        json!({"installed": [], "available": []}),
    );
    fixture
}

fn doctor(fixture: &Fixture) -> (i32, String, String) {
    run(
        fixture,
        &[
            "doctor", "skills", "--agent", "codex", "--scope", "user", "-v",
        ],
    )
}

#[test]
fn name_only_override_preserves_unrelated_path_overrides() {
    let fixture = fixture(
        "[[skills.config]]\nname = 'review'\nenabled = false\n[[skills.config]]\npath = 'SKILL_PATH'\nenabled = false\n",
    );
    let (code, stdout, _) = doctor(&fixture);

    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("folder · disabled · healthy · allowed"));
    assert!(!stdout.contains("plugin configuration"), "{stdout}");
}

#[test]
fn name_override_matches_metadata_instead_of_directory_name() {
    for (name, expected) in [(" actual-name ", "disabled"), ("folder", "enabled")] {
        let fixture = fixture(&format!(
            "[[skills.config]]\nname = '{name}'\nenabled = false\n"
        ));
        let (code, stdout, _) = doctor(&fixture);

        assert_eq!(code, 0, "{stdout}");
        assert!(stdout.contains(&format!("folder · {expected} · healthy · allowed")));
    }
}

#[test]
fn name_matching_accepts_crlf_and_whitespace_around_delimiters() {
    let fixture = fixture("[[skills.config]]\nname = 'actual-name'\nenabled = false\n");
    fixture.write_home(
        ".codex/skills/.system/folder/SKILL.md",
        " --- \r\nname: actual-name\r\ndescription: Example\r\n --- \r\nSkill\r\n",
    );
    let (code, stdout, _) = doctor(&fixture);

    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("folder · disabled · healthy · allowed"));
}

#[test]
fn name_matching_uses_normalized_metadata_or_the_folder_fallback() {
    for (metadata, name, folder) in [
        ("name: ' actual   name '", "actual name", "folder"),
        ("name: ' '", "folder", "folder"),
        ("", "folder", "folder"),
        ("", "actual name", "actual   name"),
        ("", "skill", " "),
    ] {
        let fixture = fixture(&format!(
            "[[skills.config]]\nname = '{name}'\nenabled = false\n"
        ));
        fixture.write_home(
            ".codex/skills/.system/folder/SKILL.md",
            &format!("---\n{metadata}\ndescription: Example\n---\n"),
        );
        let path = fixture.home().join(".codex/skills/.system/folder");
        fs::rename(&path, path.with_file_name(folder)).unwrap();
        let (code, stdout, _) = doctor(&fixture);

        assert_eq!(code, 0, "{stdout}");
        assert!(stdout.contains(&format!("{folder} · disabled · healthy")));
    }
}

#[test]
fn later_matching_name_or_path_override_wins() {
    for (first, second) in [
        ("name = 'actual-name'", "path = 'SKILL_PATH'"),
        ("path = 'SKILL_PATH'", "name = 'actual-name'"),
        ("name = 'actual-name'", "name = 'actual-name'"),
        ("path = 'SKILL_PATH'", "path = 'SKILL_PATH'"),
    ] {
        for (enabled, expected) in [(true, "enabled"), (false, "disabled")] {
            let fixture = fixture(&format!(
                "[[skills.config]]\n{first}\nenabled = {}\n[[skills.config]]\n{second}\nenabled = {enabled}\n",
                !enabled
            ));
            let (code, stdout, _) = doctor(&fixture);

            assert_eq!(code, 0, "{stdout}");
            assert!(stdout.contains(&format!("folder · {expected} · healthy · allowed")));
        }
    }
}

#[test]
fn ambiguous_or_empty_selectors_do_not_disable_skills() {
    for selector in [
        "",
        "name = ' '",
        "name = 'actual-name'\npath = 'SKILL_PATH'",
    ] {
        let fixture = fixture(&format!("[[skills.config]]\n{selector}\nenabled = false\n"));
        let (code, stdout, _) = doctor(&fixture);

        assert_eq!(code, 0, "{stdout}");
        assert!(stdout.contains("folder · enabled · healthy · allowed"));
    }
}

#[test]
fn syntax_and_schema_failures_are_distinct_and_redacted() {
    for (config, detail) in [
        ("[[skills.config]", "config is invalid TOML"),
        (
            "[[skills.config]]\nname = 'private-sentinel'\nenabled = 'invalid'\n",
            "config has invalid plugin or skill settings",
        ),
        (
            "[[skills.config]]\nname = 42\nenabled = false\n",
            "config has invalid plugin or skill settings",
        ),
        (
            "[[skills.config]]\nname = 'private-sentinel'\n",
            "config has invalid plugin or skill settings",
        ),
    ] {
        let fixture = fixture(config);
        let (code, stdout, stderr) = doctor(&fixture);

        assert_eq!(code, 2, "{stdout}");
        assert!(stdout.contains(detail), "{stdout}");
        assert!(stdout.contains("folder · unknown · healthy · allowed"));
        assert!(!stdout.contains("private-sentinel"));
        assert!(!stderr.contains("private-sentinel"));
    }
}

#[test]
fn unreadable_or_invalid_metadata_keeps_name_exposure_unknown() {
    for contents in [
        "no metadata",
        "---\nname: [invalid]\n---\n",
        "---\nname: actual-name\n---not-a-delimiter\n",
    ] {
        let fixture = fixture("[[skills.config]]\nname = 'actual-name'\nenabled = false\n");
        fixture.write_home(".codex/skills/.system/folder/SKILL.md", contents);
        let (code, stdout, _) = doctor(&fixture);

        assert_eq!(code, 0, "{stdout}");
        assert!(stdout.contains("UNSUPPORTED folder · unknown · healthy · allowed"));
    }

    let fixture = fixture("[[skills.config]]\nname = 'actual-name'\nenabled = false\n");
    let path = fixture.home().join(".codex/skills/.system/folder/SKILL.md");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();
    let output = fixture.command(["doctor", "skills", "--agent", "codex", "-v"]);
    fs::set_permissions(path, fs::Permissions::from_mode(0o644)).unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("UNSUPPORTED folder · unknown · healthy · allowed"));
}

#[test]
fn non_regular_metadata_does_not_block_name_matching() {
    let fixture = fixture("[[skills.config]]\nname = 'actual-name'\nenabled = false\n");
    let path = fixture.home().join(".codex/skills/.system/folder/SKILL.md");
    fs::remove_file(&path).unwrap();
    assert!(Command::new("mkfifo").arg(path).status().unwrap().success());
    let mut child = Command::new(env!("CARGO_BIN_EXE_arnes"))
        .args(["doctor", "skills", "--agent", "codex", "-v"])
        .current_dir(fixture.repository())
        .env_clear()
        .env("HOME", fixture.home())
        .env("PATH", fixture.home().join("bin"))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let start = Instant::now();
    while child.try_wait().unwrap().is_none() {
        if start.elapsed() > Duration::from_secs(2) {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("skill doctor blocked on non-regular metadata");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(stdout.contains("folder · unknown · broken · allowed"));
}

#[test]
fn name_matching_does_not_read_a_skill_outside_the_declared_root() {
    let fixture = fixture("[[skills.config]]\nname = 'actual-name'\nenabled = false\n");
    fixture.write_home("outside/SKILL.md", "---\nname: actual-name\n---\n");
    symlink(
        fixture.home().join("outside"),
        fixture.home().join(".codex/skills/.system/escape"),
    )
    .unwrap();
    let (code, stdout, _) = doctor(&fixture);

    assert_eq!(code, 2, "{stdout}");
    assert!(stdout.contains("folder · disabled · healthy · allowed"));
    assert!(stdout.contains("escape · unknown · broken · unexpected"));
}
