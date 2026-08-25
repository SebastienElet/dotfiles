use super::sources::{Category, Source};
use super::{ExportError, Metadata, sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

const FORMAT_VERSION: u8 = 1;
const MAX_SKILL_LINES: usize = 2_500;

pub(super) fn render_snapshot(sources: &[Source], metadata: &Metadata) -> BTreeMap<String, String> {
    let mut grouped = BTreeMap::<Category, Vec<&Source>>::new();
    for source in sources {
        grouped.entry(source.category).or_default().push(source);
    }
    let mut bundles = BTreeMap::new();
    let mut source_bundles = BTreeMap::new();
    for (category, category_sources) in grouped {
        for (name, selected) in category_bundles(category, category_sources) {
            for source in &selected {
                source_bundles.insert(source.path.clone(), name.clone());
            }
            bundles.insert(name, render_bundle(category, &selected));
        }
    }
    let manifest = render_manifest(sources, &source_bundles, metadata);
    bundles.insert("00-MANIFEST.md".to_owned(), manifest);
    bundles
}

fn category_bundles(category: Category, sources: Vec<&Source>) -> Vec<(String, Vec<&Source>)> {
    if category != Category::Skills {
        return vec![(bundle_name(category, 1), sources)];
    }
    let mut partitions = vec![Vec::new()];
    let mut lines = 0;
    for skill in skill_groups(sources) {
        let skill_lines = skill
            .iter()
            .map(|source| line_count(&source.contents))
            .sum::<usize>();
        if lines > 0 && lines + skill_lines > MAX_SKILL_LINES {
            partitions.push(Vec::new());
            lines = 0;
        }
        partitions.last_mut().unwrap().extend(skill);
        lines += skill_lines;
    }
    partitions
        .into_iter()
        .enumerate()
        .map(|(index, sources)| (bundle_name(category, index + 1), sources))
        .collect()
}

fn skill_groups(sources: Vec<&Source>) -> Vec<Vec<&Source>> {
    let mut groups = Vec::<Vec<&Source>>::new();
    let mut current_slug = "";
    for source in sources {
        let slug = source.path.split('/').nth(2).unwrap_or("");
        if slug != current_slug {
            groups.push(Vec::new());
            current_slug = slug;
        }
        groups.last_mut().unwrap().push(source);
    }
    groups
}

fn bundle_name(category: Category, index: usize) -> String {
    match category {
        Category::Instructions => "10-INSTRUCTIONS.md".to_owned(),
        Category::CommandsRules => "20-COMMANDS-RULES.md".to_owned(),
        Category::Skills => format!("{}-SKILLS-{index:02}.md", 29 + index),
        Category::HooksRouting => "40-HOOKS-ROUTING.md".to_owned(),
        Category::Services => "50-SERVICES-CONFIG.md".to_owned(),
    }
}

fn render_bundle(category: Category, sources: &[&Source]) -> String {
    let title = match category {
        Category::Instructions => "User harness instructions",
        Category::CommandsRules => "Commands and rules",
        Category::Skills => "User skills",
        Category::HooksRouting => "Hooks and routing",
        Category::Services => "Services and configuration",
    };
    let mut output = format!(
        "# {title}\n\n> Generated artifact. Do not edit; regenerate with `arnes export`.\n\n"
    );
    for source in sources {
        output.push_str(&format!("# FILE: {}\n\n", source.path));
        output.push_str(&source.contents);
        if !source.contents.ends_with('\n') {
            output.push('\n');
        }
        output.push_str(&format!("\n# END FILE: {}\n\n", source.path));
    }
    output
}

fn render_manifest(
    sources: &[Source],
    source_bundles: &BTreeMap<String, String>,
    metadata: &Metadata,
) -> String {
    let mut output = format!(
        "# Harness project export manifest\n\n> Generated artifact. This snapshot is derived and disposable. Do not edit it manually.\n\n- Format version: `{FORMAT_VERSION}`\n- Generated at (Unix UTC): `{}`\n- Git commit at generation (informational): `{}`\n- Repository state at generation (informational): `{}`\n- Metadata SHA256: `{}`\n\n| Source | Kind | Bundle | SHA256 | Bytes | Lines |\n| --- | --- | --- | --- | ---: | ---: |\n",
        metadata.generated_at,
        metadata.commit,
        metadata.repository_state,
        metadata_sha256(metadata)
    );
    for source in sources {
        output.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} |\n",
            source.path,
            source.kind,
            source_bundles[&source.path],
            sha256(&source.contents),
            source.contents.len(),
            line_count(&source.contents)
        ));
    }
    output
}

fn line_count(contents: &str) -> usize {
    contents.lines().count()
}

pub(super) fn read_metadata(path: &Path) -> Result<Metadata, ExportError> {
    let input = fs::read_to_string(path).map_err(|error| {
        ExportError::new(format!(
            "export does not exist or manifest is unreadable: {error}"
        ))
    })?;
    let generated_at = field(&input, "Generated at (Unix UTC)")?
        .parse::<u64>()
        .map_err(|_| ExportError::new("manifest generation time is invalid"))?;
    let commit = field(&input, "Git commit at generation (informational)")?.to_owned();
    if commit != "unavailable"
        && (commit.is_empty() || !commit.bytes().all(|byte| byte.is_ascii_hexdigit()))
    {
        return Err(ExportError::new("manifest Git commit is invalid"));
    }
    let repository_state =
        field(&input, "Repository state at generation (informational)")?.to_owned();
    if !matches!(repository_state.as_str(), "clean" | "dirty") {
        return Err(ExportError::new("manifest repository state is invalid"));
    }
    let metadata = Metadata {
        commit,
        generated_at,
        repository_state,
    };
    let recorded_hash = field(&input, "Metadata SHA256")?;
    if recorded_hash != metadata_sha256(&metadata) {
        return Err(ExportError::new("manifest metadata integrity check failed"));
    }
    Ok(metadata)
}

fn metadata_sha256(metadata: &Metadata) -> String {
    sha256(&format!(
        "{}\n{}\n{}\n",
        metadata.generated_at, metadata.commit, metadata.repository_state
    ))
}

fn field<'a>(input: &'a str, name: &str) -> Result<&'a str, ExportError> {
    let prefix = format!("- {name}: `");
    let line = input
        .lines()
        .find(|line| line.starts_with(&prefix))
        .ok_or_else(|| ExportError::new(format!("manifest field {name} is missing")))?;
    line.strip_prefix(&prefix)
        .and_then(|value| value.strip_suffix('`'))
        .ok_or_else(|| ExportError::new(format!("manifest field {name} is malformed")))
}
