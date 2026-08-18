use super::super::MeasureError;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader};
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

pub fn validate_json(path: &Path) -> Result<serde_json::Value, MeasureError> {
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32)
        .open(path)?;
    ensure_single_link(&file.metadata()?, path)?;
    Ok(serde_json::from_reader(file)?)
}

pub fn validate_run(
    current: &serde_json::Value,
    expected: &serde_json::Value,
) -> Result<(), MeasureError> {
    let current = current
        .as_object()
        .ok_or_else(|| MeasureError::new("managed run.json must be an object"))?;
    let expected = expected
        .as_object()
        .ok_or_else(|| MeasureError::new("expected run record must be an object"))?;
    for key in ["schema_version", "run_id", "agent", "session_id"] {
        if current.get(key) != expected.get(key) {
            return Err(MeasureError::new(format!(
                "managed run.json has an unexpected {key}"
            )));
        }
    }
    if !current
        .get("started_at_ms")
        .is_some_and(serde_json::Value::is_u64)
        || !current
            .get("harness_fingerprint")
            .is_some_and(serde_json::Value::is_string)
    {
        return Err(MeasureError::new("managed run.json has an invalid schema"));
    }
    Ok(())
}
