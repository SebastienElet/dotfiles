use arnes::Roots;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

pub fn run() -> ExitCode {
    if let Err(error) = load() {
        let _ = crate::cli_output::write_error(format_args!("output-discipline: {error}"));
    }
    ExitCode::SUCCESS
}

fn load() -> Result<(), Box<dyn std::error::Error>> {
    let config = env::var_os("CLAUDE_CONFIG_DIR")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".claude")))
        .ok_or("HOME is required when CLAUDE_CONFIG_DIR is unset")?;
    let marker = config.join(".output-discipline-always");
    if !marker.try_exists()? {
        return Ok(());
    }
    let home = env::var_os("HOME")
        .map(PathBuf::from)
        .filter(|home| home.is_absolute())
        .ok_or("HOME must be an absolute path")?;
    let manifest = home.join(".arnes.yaml");
    let metadata = fs::symlink_metadata(&manifest)
        .map_err(|error| format!("deployed {} is unavailable: {error}", manifest.display()))?;
    if !metadata.file_type().is_symlink() {
        return Err("deployed ~/.arnes.yaml must be a symlink to the canonical repository".into());
    }
    let roots = Roots::from_environment()?;
    let source = roots
        .deployment_repository()
        .join("harness/skills/output-discipline/SKILL.md");
    let text =
        fs::read_to_string(&source).map_err(|error| format!("{}: {error}", source.display()))?;
    let body = strip_frontmatter(&text).trim_end_matches(['\r', '\n']);
    crate::cli_output::write_output(&format!(
        "OUTPUT DISCIPLINE ACTIVE (always-on). The ruleset below applies to every response.\n\n{body}"
    ))?;
    Ok(())
}

fn strip_frontmatter(text: &str) -> &str {
    let Some((opening, rest)) = text.split_once('\n') else {
        return text;
    };
    if !delimiter(opening) {
        return text;
    }
    let mut lines = rest.split_inclusive('\n');
    let Some(first) = lines.next() else {
        return text;
    };
    let mut offset = opening.len() + 1 + first.len();
    for line in lines {
        offset += line.len();
        if delimiter(line.strip_suffix('\n').unwrap_or(line)) {
            return text.get(offset..).unwrap_or(text);
        }
    }
    text
}

fn delimiter(line: &str) -> bool {
    line.strip_suffix('\r')
        .unwrap_or(line)
        .strip_prefix("---")
        .is_some_and(|suffix| {
            suffix.chars().all(|character| {
                character.is_whitespace() && character != '\r' && character != '\n'
            })
        })
}
