use crate::support::Fixture;
use serde_json::{Value, json};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

pub fn install(fixture: &Fixture, marketplaces: Value, plugins: Value) {
    fixture.write_home(
        ".codex-test-marketplaces.json",
        &serde_json::to_string(&marketplaces).unwrap(),
    );
    fixture.write_home(
        ".codex-test-plugins.json",
        &serde_json::to_string(&plugins).unwrap(),
    );
    install_script(
        fixture,
        "#!/bin/sh\nif [ \"$1 $2 $3 $4\" = \"plugin marketplace list --json\" ]; then file=\"$HOME/.codex-test-marketplaces.json\"; elif [ \"$1 $2 $3\" = \"plugin list --json\" ]; then file=\"$HOME/.codex-test-plugins.json\"; else exit 64; fi\nwhile IFS= read -r line || [ -n \"$line\" ]; do printf '%s\\n' \"$line\"; done < \"$file\"\n",
    );
}

pub fn install_script(fixture: &Fixture, script: &str) {
    let path = fixture.home().join("bin/codex");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, script).unwrap();
    let mut permissions = fs::metadata(&path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
}

pub fn marketplace(name: &str, root: &Path) -> Value {
    json!({"name": name, "root": root})
}

pub fn plugin(
    id: &str,
    marketplace_name: &str,
    artifact: &str,
    enabled: bool,
    path: &Path,
) -> Value {
    let name = id.split_once('@').map_or(id, |(name, _)| name);
    json!({
        "pluginId": id,
        "name": name,
        "marketplaceName": marketplace_name,
        "version": artifact,
        "installed": true,
        "enabled": enabled,
        "source": {"source": "local", "path": path}
    })
}
