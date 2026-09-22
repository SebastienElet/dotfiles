mod support;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use support::Fixture;

#[test]
fn codex_inventory_never_executes_a_resolver_from_direct_or_aggregate_doctor()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for config in [
        None,
        Some(""),
        Some("[plugins.\"demo@marketplace\"]\nenabled = true\n"),
    ] {
        let fixture = Fixture::new()?;
        fixture.write_home(
            ".arnes.yaml",
            "version: 1\nagents:\n  - { id: codex, scopes: [user, project] }\nresources: []\nexternal:\n  plugins:\n    - { agent: codex, scope: user, id: demo@marketplace }\n",
        )?;
        if let Some(config) = config {
            fixture.write_home(".codex/config.toml", config)?;
        }
        fixture.write_home(
            "bin/codex",
            "#!/bin/sh\nprintf invoked >> \"$HOME/../resolver-invoked\"\nprintf temporary > \"$HOME/transient\"\n/bin/rm \"$HOME/transient\"\nprintf '{\"marketplaces\":[],\"installed\":[]}\\n'\n",
        )?;
        fs::set_permissions(
            fixture.home().join("bin/codex"),
            fs::Permissions::from_mode(0o755),
        )?;
        let before = fixture.snapshot()?;
        let witness = fixture
            .repository()
            .parent()
            .ok_or("fixture repository has no parent")?
            .join("resolver-invoked");
        for selector in [vec!["doctor", "skills"], vec!["doctor"]] {
            for format in ["human", "json"] {
                let args = selector
                    .iter()
                    .copied()
                    .chain(["--agent", "codex", "--scope", "user", "--format", format]);
                let output = fixture.command(args)?;
                assert_eq!(fixture.snapshot()?, before);
                assert!(
                    !witness.exists(),
                    "Doctor executed the external Codex resolver: {selector:?} {format} {config:?}"
                );
                assert!(output.stderr.is_empty());
                assert_ne!(output.status.code(), Some(2));
                let stdout = String::from_utf8(output.stdout)?;
                assert!(stdout.contains("read-only"), "{stdout}");
                assert!(stdout.to_lowercase().contains("unsupported"), "{stdout}");
            }
        }
    }
    Ok(())
}
