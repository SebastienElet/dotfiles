use super::{
    evidence::read_report,
    sources::{load_cases, load_trigger, read_source},
};
use std::{fs, path::Path, process::Command};

/// # Errors
/// Returns errors loading cases, enumerating or reading tracked skills, finding numbered references, or validating activation contracts.
pub fn validate_evaluations(repository: &Path) -> Result<String, String> {
    let cases = load_cases(repository)?;
    let result = Command::new("git")
        .arg("-C")
        .arg(repository)
        .args([
            "ls-files",
            "-z",
            "--",
            "harness/skills/",
            ".agents/skills/*/evals/trigger-queries.json",
        ])
        .output()
        .map_err(|e| format!("Cannot enumerate tracked skills and activation contracts: {e}"))?;
    if !result.status.success() {
        return Err("Cannot enumerate tracked skills and activation contracts".into());
    }
    let output = String::from_utf8(result.stdout).map_err(|e| e.to_string())?;
    let paths = output
        .split('\0')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>();
    let skills = paths
        .iter()
        .filter(|path| path.starts_with("harness/skills/"))
        .collect::<Vec<_>>();
    if skills.is_empty() {
        return Err("Empty tracked skill selection".into());
    }
    for path in skills {
        let text = read_source(repository, path)
            .map_err(|e| format!("Cannot read tracked skill: {path}: {e}"))?;
        if text
            .as_bytes()
            .windows(2)
            .any(|pair| pair.first() == Some(&b'#') && pair.get(1).is_some_and(u8::is_ascii_digit))
        {
            return Err(format!(
                "Forbidden numbered PR/issue reference in skill: {path}"
            ));
        }
    }
    let triggers = paths
        .iter()
        .filter(|path| path.ends_with("/evals/trigger-queries.json"))
        .collect::<Vec<_>>();
    for path in &triggers {
        load_trigger(repository, path)
            .map_err(|e| format!("Invalid activation contract: {path}: {e}"))?;
    }
    Ok(format!(
        "{} behavioral cases; {} activation contracts valid",
        cases.len(),
        triggers.len()
    ))
}

/// # Errors
/// Returns errors enumerating the evidence directory or reading and validating any selected JSON report.
pub fn validate_evidence(repository: &Path) -> Result<String, String> {
    let directory = repository.join("harness/evals/evidence");
    let mut files = fs::read_dir(directory)
        .map_err(|e| e.to_string())?
        .map(|entry| entry.map(|e| e.path()).map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    files.retain(|path| path.extension().is_some_and(|ext| ext == "json"));
    files.sort();
    for file in &files {
        read_report(file).map_err(|e| format!("Invalid evidence: {}: {e}", file.display()))?;
    }
    Ok(format!(
        "{} historical reports valid; live evidence is optional",
        files.len()
    ))
}
