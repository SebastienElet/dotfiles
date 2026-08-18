use super::super::MeasureError;
use super::super::model::RunRecord;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Read};
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

pub fn validate_jsonl(file: &File) -> Result<(), MeasureError> {
    let mut reader = BufReader::new(file);
    let mut line = Vec::new();
    while reader.read_until(b'\n', &mut line)? != 0 {
        if line.last() != Some(&b'\n') || line.len() > 1_100_000 {
            return Err(MeasureError::new(
                "managed JSONL file is truncated or oversized",
            ));
        }
        serde_json::from_slice::<serde_json::Value>(&line[..line.len() - 1])?;
        line.clear();
    }
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

pub fn validate_run(current: &RunRecord, expected: &RunRecord) -> Result<(), MeasureError> {
    for (key, matches) in [
        (
            "schema_version",
            current.schema_version == expected.schema_version,
        ),
        ("run_id", current.run_id == expected.run_id),
        ("agent", current.agent == expected.agent),
        ("session_id", current.session_id == expected.session_id),
    ] {
        if !matches {
            return Err(MeasureError::new(format!(
                "managed run.json has an unexpected {key}"
            )));
        }
    }
    Ok(())
}
