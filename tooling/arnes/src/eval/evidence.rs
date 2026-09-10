use super::{
    contracts::{Prompt, hash, nonempty},
    oracle::evaluate,
    report::{Environment, Report, ReportCase, Status},
    sources::fingerprint,
};
use std::{collections::HashSet, fs, io::Write, path::Path};

#[cfg(test)]
#[path = "evidence/tests.rs"]
mod tests;

/// # Errors
/// Rejects malformed report metadata, controls, case identities, run counts, fingerprints, or oracle results.
pub fn validate_report(report: &Report) -> Result<(), String> {
    if report.schema_version != 1
        || !(1..=10).contains(&report.run_count)
        || report.cases.is_empty()
        || report.limitations.is_empty()
    {
        return Err("Invalid report structure".into());
    }
    for text in [
        &report.agent,
        &report.agent_version,
        &report.model,
        &report.harness.variant,
    ]
    .into_iter()
    .chain(report.limitations.iter())
    {
        nonempty(text)?;
    }
    hash(&report.harness.git_revision, 40)?;
    for value in [
        &report.runner_revision,
        &report.harness.instruction_fingerprint,
        &report.harness.skill_fingerprint,
    ] {
        hash(value, 64)?;
    }
    validate_date(&report.date)?;
    let environment = match &report.environment {
        Environment::Rust {
            platform,
            architecture,
            runtime,
        } => [platform, architecture, runtime],
        Environment::Legacy {
            platform,
            architecture,
            bun,
        } => [platform, architecture, bun],
    };
    for value in environment {
        nonempty(value)?;
    }
    let controls = &report.controls;
    if controls.sandbox != "workspace-write"
        || controls.network
        || controls.tools != "shell-with-synthetic-cat-rg-fd-colgrep-v1"
        || !(1..=600).contains(&controls.timeout_seconds)
        || !["low", "medium", "high"].contains(&controls.reasoning_effort.as_str())
    {
        return Err("Invalid controls".into());
    }
    let mut identifiers = HashSet::new();
    for entry in &report.cases {
        if !identifiers.insert(&entry.definition.id) {
            return Err("Duplicate report case ID".into());
        }
        validate_entry(entry, report.run_count)?;
    }
    Ok(())
}

fn validate_entry(entry: &ReportCase, run_count: usize) -> Result<(), String> {
    entry.definition.validate()?;
    nonempty(&entry.prompt)?;
    hash(&entry.prompt_fingerprint, 64)?;
    hash(&entry.fixture_revision, 64)?;
    if entry.runs.len() != run_count {
        return Err("Run count mismatch".into());
    }
    if fingerprint(&entry.prompt) != entry.prompt_fingerprint {
        return Err("Prompt fingerprint mismatch".into());
    }
    if matches!(&entry.definition.prompt, Prompt::Inline{text} if text != &entry.prompt) {
        return Err("Inline prompt mismatch".into());
    }
    if entry.definition.sources.len() != entry.sources.len()
        || !entry
            .definition
            .sources
            .iter()
            .zip(&entry.sources)
            .all(|(a, b)| a.path == b.path && a.heading == b.heading)
    {
        return Err("Source reference mismatch".into());
    }
    for source in &entry.sources {
        hash(&source.fingerprint, 64)?;
    }
    for run in &entry.runs {
        if (run.status == Status::Invalid) != run.error.is_some() {
            return Err("Invalid status/error combination".into());
        }
        if run.status != Status::Invalid
            && evaluate(entry.definition.oracle, &run.observations) != run.status
        {
            return Err("Oracle verdict mismatch".into());
        }
        if run
            .duration_ms
            .is_some_and(|duration| !duration.is_finite() || duration < 0.0)
        {
            return Err("Invalid duration".into());
        }
        let integers = run
            .observations
            .iter()
            .map(|o| o.exit_code)
            .chain(run.tool_calls)
            .chain(
                run.tokens
                    .iter()
                    .flat_map(|t| [t.input, t.cached_input, t.output]),
            );
        if integers.into_iter().any(|n| n > 9_007_199_254_740_991) {
            return Err("Unsafe integer".into());
        }
    }
    Ok(())
}

fn validate_date(date: &str) -> Result<(), String> {
    let invalid = || "Invalid ISO datetime".to_string();
    let (day, time) = date
        .strip_suffix('Z')
        .and_then(|v| v.split_once('T'))
        .ok_or_else(invalid)?;
    let parts = day.split('-').collect::<Vec<_>>();
    let [year, month, day] = parts.as_slice() else {
        return Err(invalid());
    };
    if year.len() != 4 || month.len() != 2 || day.len() != 2 {
        return Err(invalid());
    }
    let parse = |s: &str| {
        if !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()) {
            s.parse::<u32>().map_err(|_| invalid())
        } else {
            Err(invalid())
        }
    };
    let year = parse(year)?;
    let month = parse(month)?;
    let day = parse(day)?;
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return Err(invalid()),
    };
    if day == 0 || day > max_day {
        return Err(invalid());
    }
    let parts = time.split(':').collect::<Vec<_>>();
    let (hour, minute, second) = match parts.as_slice() {
        [hour, minute] => (*hour, *minute, None),
        [hour, minute, second] => (*hour, *minute, Some(*second)),
        _ => return Err(invalid()),
    };
    if hour.len() != 2 || minute.len() != 2 || parse(hour)? > 23 || parse(minute)? > 59 {
        return Err(invalid());
    }
    if let Some(second) = second {
        let (seconds, fraction) = second
            .split_once('.')
            .map_or((second, None), |(s, f)| (s, Some(f)));
        if seconds.len() != 2
            || parse(seconds)? > 59
            || fraction.is_some_and(|f| f.is_empty() || !f.bytes().all(|b| b.is_ascii_digit()))
        {
            return Err(invalid());
        }
    }
    Ok(())
}

/// # Errors
/// Returns file-reading, JSON-decoding, or report-validation errors.
pub fn read_report(path: &Path) -> Result<Report, String> {
    let report = serde_json::from_str(&fs::read_to_string(path).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    validate_report(&report)?;
    Ok(report)
}

/// # Errors
/// Rejects an existing report, an inaccessible parent directory, or inability to create a temporary file.
pub fn assert_new_report(path: &Path) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Ok(_) => return Err(format!("Report already exists: {}", path.display())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.to_string()),
    }
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    if !fs::metadata(parent).map_err(|e| e.to_string())?.is_dir() {
        return Err("Report parent is not a directory".into());
    }
    tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    Ok(())
}

/// # Errors
/// Returns report-validation, filesystem, serialization, or atomic publication errors; an existing destination is never overwritten.
pub fn publish_report(path: &Path, report: &Report) -> Result<(), String> {
    validate_report(report)?;
    assert_new_report(path)?;
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let mut temporary = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    let bytes = format!(
        "{}\n",
        serde_json::to_string_pretty(report).map_err(|e| e.to_string())?
    );
    temporary
        .write_all(bytes.as_bytes())
        .map_err(|e| e.to_string())?;
    fs::hard_link(temporary.path(), path).map_err(|e| e.to_string())
}
