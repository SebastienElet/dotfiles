use super::command;
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SelectedPlugin {
    pub plugin_id: String,
    pub name: String,
    pub marketplace_name: String,
    pub version: Option<String>,
    pub installed: bool,
    pub enabled: bool,
}

pub(super) struct ResolverState {
    pub marketplaces: Vec<Marketplace>,
    pub plugins: Vec<SelectedPlugin>,
}

#[derive(Deserialize)]
pub(super) struct Marketplace {
    pub name: String,
}

#[derive(Deserialize)]
struct MarketplaceList {
    marketplaces: Vec<Marketplace>,
}

#[derive(Deserialize)]
struct PluginList {
    installed: Vec<SelectedPlugin>,
}

pub(super) fn load(home: &Path) -> Result<ResolverState, String> {
    let marketplaces = command::run(
        home,
        &["plugin", "marketplace", "list", "--json"],
        "marketplace",
    )?;
    let marketplaces: MarketplaceList = serde_json::from_slice(&marketplaces)
        .map_err(|_| "Codex marketplace resolver returned invalid JSON".to_owned())?;
    let plugins = command::run(home, &["plugin", "list", "--json"], "plugin")?;
    let plugins: PluginList = serde_json::from_slice(&plugins)
        .map_err(|_| "Codex plugin resolver returned invalid JSON".to_owned())?;
    Ok(ResolverState {
        marketplaces: marketplaces.marketplaces,
        plugins: plugins.installed,
    })
}
