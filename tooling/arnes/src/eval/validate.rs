use super::{
    evidence::read_report,
    sources::{load_cases, load_trigger},
};
use std::{fs, path::Path, process::Command};

/// # Errors
/// Returns errors loading cases, enumerating tracked contracts with Git, or validating required activation contracts.
pub fn validate_evaluations(repository: &Path) -> Result<String, String> {
    let cases = load_cases(repository)?;
    let result = Command::new("git")
        .arg("-C")
        .arg(repository)
        .args([
            "ls-files",
            "-z",
            "harness/skills/*/evals/trigger-queries.json",
            ".agents/skills/*/evals/trigger-queries.json",
        ])
        .output()
        .map_err(|e| format!("Cannot enumerate tracked activation contracts: {e}"))?;
    if !result.status.success() {
        return Err("Cannot enumerate tracked activation contracts".into());
    }
    let output = String::from_utf8(result.stdout).map_err(|e| e.to_string())?;
    let paths = output
        .split('\0')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>();
    if !paths.contains(&"harness/skills/code-search/evals/trigger-queries.json") {
        return Err("Missing code-search activation contract".into());
    }
    for path in &paths {
        load_trigger(repository, path)
            .map_err(|e| format!("Invalid activation contract: {path}: {e}"))?;
    }
    Ok(format!(
        "{} behavioral cases; {} activation contracts valid",
        cases.len(),
        paths.len()
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
