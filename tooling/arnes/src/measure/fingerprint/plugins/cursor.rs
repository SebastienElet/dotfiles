use super::{PluginSelection, manifest as plugin_manifest, safe_label, within};
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
    let manifest_file = plugin_manifest::read(&home.join(".arnes.yaml"))?;
    if let Some(marker) = manifest_file.marker {
        selection.markers.push(format!("cursor:manifest:{marker}"));
        selection
            .limitations
            .push("oversized plugin manifest uses size and boundary windows".to_owned());
    }
    let Some(contents) = manifest_file.contents else {
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
        add_local(&mut selection, &local, id);
    }
    Ok(selection)
}

fn add_local(selection: &mut PluginSelection, local: &Path, id: &str) {
    let path = local.join(id);
    if within(&path, local) {
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
