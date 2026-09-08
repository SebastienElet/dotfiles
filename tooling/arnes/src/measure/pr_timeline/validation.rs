use super::super::MeasureError;
use super::model::{PrEventRecord, PrIdentity, PrVerdictArgs};

pub(super) fn input(args: &PrVerdictArgs) -> Result<(), MeasureError> {
    identity(&args.pr)?;
    head_sha(&args.head_sha)
}

pub(super) fn record(record: &PrEventRecord) -> Result<(), MeasureError> {
    identity(&record.pr)?;
    head_sha(&record.head_sha)?;
    let valid_agent = record
        .agent
        .as_deref()
        .is_none_or(|agent| matches!(agent, "codex" | "claude-code" | "cursor"));
    if record.schema_version != 1
        || record.timestamp_ms == 0
        || !valid_agent
        || record.operating_system.is_empty()
        || record.architecture.is_empty()
    {
        return Err(MeasureError::new(
            "managed PR event has invalid metadata or schema version",
        ));
    }
    Ok(())
}

fn identity(pr: &PrIdentity) -> Result<(), MeasureError> {
    let valid_host = (1..=253).contains(&pr.forge.len())
        && pr.forge.split('.').all(|label| {
            (1..=63).contains(&label.len())
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        });
    let valid_repository = (1..=255).contains(&pr.repository.len())
        && pr.repository.contains('/')
        && !pr.repository.ends_with(".git")
        && pr.repository.split('/').all(|segment| {
            !segment.is_empty()
                && !matches!(segment, "." | "..")
                && segment
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        });
    if !valid_host || !valid_repository || pr.pr_id == 0 {
        return Err(MeasureError::new(
            "PR identity requires a lowercase forge hostname, canonical repository path and positive PR id",
        ));
    }
    Ok(())
}

fn head_sha(value: &str) -> Result<(), MeasureError> {
    if matches!(value.len(), 40 | 64)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(())
    } else {
        Err(MeasureError::new(
            "head SHA must be 40 or 64 lowercase hexadecimal characters",
        ))
    }
}
