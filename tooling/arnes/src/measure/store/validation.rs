use super::super::MeasureError;
use super::super::model::RunRecord;
use super::MAX_RECORD_BYTES;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::Path;

pub fn ensure_regular_or_missing(path: &Path) -> Result<(), MeasureError> {
    match fs::symlink_metadata(path) {
        Ok(_) => ensure_regular_file(path),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

pub fn ensure_regular_file(path: &Path) -> Result<(), MeasureError> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.nlink() != 1 {
        return Err(MeasureError::new(format!(
            "managed path is not a single-link regular file: {}",
            path.display()
        )));
    }
    Ok(())
}

pub fn ensure_single_link(metadata: &fs::Metadata, path: &Path) -> Result<(), MeasureError> {
    if metadata.nlink() != 1 {
        return Err(MeasureError::new(format!(
            "managed file has multiple hardlinks: {}",
            path.display()
        )));
    }
    Ok(())
}

pub fn validate_jsonl(file: &mut File) -> Result<(), MeasureError> {
    let length = file.metadata()?.len();
    if length == 0 {
        return Ok(());
    }
    let window = length.min((MAX_RECORD_BYTES + 1) as u64) as usize;
    file.seek(SeekFrom::End(-(window as i64)))?;
    let mut tail = vec![0; window];
    file.read_exact(&mut tail)?;
    let content = tail
        .strip_suffix(b"\n")
        .ok_or_else(|| MeasureError::new("managed JSONL file is truncated or oversized"))?;
    let start = content
        .iter()
        .rposition(|byte| *byte == b'\n')
        .map_or(0, |index| index + 1);
    if start == 0 && length > window as u64 || content.len() - start >= MAX_RECORD_BYTES {
        return Err(MeasureError::new(
            "managed JSONL file is truncated or oversized",
        ));
    }
    serde_json::from_slice::<serde_json::Value>(&content[start..])?;
    Ok(())
}

pub fn read_run(path: &Path) -> Result<RunRecord, MeasureError> {
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32)
        .open(path)?;
    ensure_single_link(&file.metadata()?, path)?;
    let mut bytes = Vec::new();
    file.by_ref().take(1_048_577).read_to_end(&mut bytes)?;
    if bytes.len() > 1_048_576 {
        return Err(MeasureError::new("managed run.json is oversized"));
    }
    let record: RunRecord = serde_json::from_slice(&bytes).map_err(|error| {
        MeasureError::new(format!("managed run.json has an invalid schema: {error}"))
    })?;
    let raw: serde_json::Value = serde_json::from_slice(&bytes)?;
    if serde_json::to_value(&record)? != raw {
        return Err(MeasureError::new(
            "managed run.json does not exactly match its schema",
        ));
    }
    Ok(record)
}

pub fn validate_run(
    current: &RunRecord,
    agent: &str,
    session: &str,
    run_id: &str,
) -> Result<(), MeasureError> {
    for (key, matches) in [
        ("schema_version", current.schema_version == 1),
        ("run_id", current.run_id == run_id),
        ("agent", current.agent == agent),
        ("session_id", current.session_id == session),
    ] {
        if !matches {
            return Err(MeasureError::new(format!(
                "managed run.json has an unexpected {key}"
            )));
        }
    }
    Ok(())
}
