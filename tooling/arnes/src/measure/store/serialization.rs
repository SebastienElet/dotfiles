use super::super::MeasureError;
use super::validation::validate_jsonl;
use super::{MAX_RECORD_BYTES, open_private_append, open_private_new, temporary_path};
use serde::Serialize;
use std::fs;
use std::io::Write;
use std::path::Path;

pub fn append_jsonl<T: Serialize>(path: &Path, value: &T) -> Result<(), MeasureError> {
    let bytes = jsonl_bytes(value)?;
    append_jsonl_bytes(path, &bytes)
}

pub fn append_jsonl_bytes(path: &Path, bytes: &[u8]) -> Result<(), MeasureError> {
    ensure_record_size(bytes)?;
    let mut file = open_private_append(path)?;
    file.lock()?;
    validate_jsonl(&mut file)?;
    file.write_all(bytes)?;
    file.sync_data()?;
    Ok(())
}

pub fn jsonl_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, MeasureError> {
    let mut bytes = serde_json::to_vec(value)?;
    bytes.push(b'\n');
    ensure_record_size(&bytes)?;
    Ok(bytes)
}

pub fn json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, MeasureError> {
    let bytes = serde_json::to_vec_pretty(value)?;
    ensure_record_size(&bytes)?;
    Ok(bytes)
}

pub fn write_json_atomic<T: Serialize>(path: &Path, value: &T) -> Result<(), MeasureError> {
    let bytes = json_bytes(value)?;
    let temporary = temporary_path(path);
    let mut file = open_private_new(&temporary)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    fs::rename(&temporary, path)?;
    Ok(())
}

fn ensure_record_size(bytes: &[u8]) -> Result<(), MeasureError> {
    if bytes.len() > MAX_RECORD_BYTES {
        Err(MeasureError::new("serialized record exceeds 1100000 bytes"))
    } else {
        Ok(())
    }
}
