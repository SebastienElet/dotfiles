use crate::support::Fixture;
use serde_json::{Value, json};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
/// # Errors
/// Returns an error if the fixture resolver script or serialized responses cannot be written.
pub fn install(
    fixture: &Fixture,
    marketplaces: &Value,
    plugins: &Value,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    fixture.write_home(
        ".codex-test-marketplaces.json",
        &serde_json::to_string(&marketplaces)?,
    )?;
    fixture.write_home(
        ".codex-test-plugins.json",
        &serde_json::to_string(&plugins)?,
    )?;
    install_script(
        fixture,
        "#!/bin/sh\nif [ \"$1 $2 $3 $4\" = \"plugin marketplace list --json\" ]; then file=\"$HOME/.codex-test-marketplaces.json\"; elif [ \"$1 $2 $3\" = \"plugin list --json\" ]; then file=\"$HOME/.codex-test-plugins.json\"; else exit 64; fi\nwhile IFS= read -r line || [ -n \"$line\" ]; do printf '%s\\n' \"$line\"; done < \"$file\"\n",
    )?;
    Ok(())
}
/// # Errors
/// Returns an error if the fixture resolver script cannot be written or made executable.
pub fn install_script(
    fixture: &Fixture,
    script: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let path = fixture.home().join("bin/codex");
    fs::create_dir_all(path.parent().ok_or("fixture path has no parent")?)?;
    fs::write(&path, script)?;
    let mut permissions = fs::metadata(&path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)?;
    Ok(())
}
#[must_use]
pub fn marketplace(name: &str, root: &Path) -> Value {
    json ! ({ "name" : name , "root" : root })
}
#[must_use]
pub fn plugin(
    id: &str,
    marketplace_name: &str,
    artifact: &str,
    enabled: bool,
    path: &Path,
) -> Value {
    let name = id.split_once('@').map_or(id, |(name, _)| name);
    json ! ({ "pluginId" : id , "name" : name , "marketplaceName" : marketplace_name , "version" : artifact , "installed" : true , "enabled" : enabled , "source" : { "source" : "local" , "path" : path } })
}
