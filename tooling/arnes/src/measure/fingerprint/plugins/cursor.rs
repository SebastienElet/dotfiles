use super::{PluginSelection, read_manifest, safe_label, within};
use crate::manifest::{self, Agent, Scope};
use crate::measure::MeasureError;
use crate::measure::fingerprint::SelectedRoot;
use std::path::Path;

pub fn selected(home: &Path) -> Result<PluginSelection, MeasureError> {
    let mut selection = PluginSelection {
        roots: Vec::new(),
        limitations: vec![
            "cursor marketplace plugin activation is not observable; marketplace caches excluded"
                .to_owned(),
        ],
        markers: Vec::new(),
    };
    let Some(contents) = read_manifest(&home.join(".arnes.yaml"))? else {
        selection.limitations.push(
            "cursor local plugin declarations are not observable; local plugin files excluded"
                .to_owned(),
        );
        return Ok(selection);
    };
    let manifest = match manifest::parse(&contents) {
        Ok(manifest) => manifest,
        Err(_) => {
            selection.limitations.push(
                "cursor local plugin declarations are invalid; local plugin files excluded"
                    .to_owned(),
            );
            return Ok(selection);
        }
    };
    let local = home.join(".cursor/plugins/local");
    for id in manifest.external_plugins(Agent::Cursor, Scope::User) {
        let path = local.join(id);
        if within(&path, &local) {
            selection.roots.push(SelectedRoot {
                label: Path::new("home/.cursor/plugins/active").join(safe_label(id)),
                path,
                bounded: true,
            });
        } else {
            selection
                .markers
                .push(format!("cursor:{id}:local-root-escape"));
            selection.limitations.push(
                "cursor declared local plugin root is unresolved; plugin files excluded".to_owned(),
            );
        }
    }
    Ok(selection)
}
