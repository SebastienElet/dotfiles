use super::super::MeasureError;
use super::super::store::validation::{ensure_regular_file, ensure_single_link};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

const MAX_RECORD_BYTES: usize = 1_100_000;

pub fn read_optional_json<T>(path: &Path, label: &str) -> Result<Option<T>, MeasureError>
where
    T: DeserializeOwned + Serialize,
{
    match std::fs::symlink_metadata(path) {
        Ok(_) => read_json(path, label).map(Some),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

pub fn read_json<T>(path: &Path, label: &str) -> Result<T, MeasureError>
where
    T: DeserializeOwned + Serialize,
{
    ensure_regular_file(path)?;
    let mut file = open_read(path)?;
    let mut bytes = Vec::new();
    file.by_ref()
        .take((MAX_RECORD_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.len() > MAX_RECORD_BYTES {
        return Err(MeasureError::new(format!("managed {label} is oversized")));
    }
    let record: T = serde_json::from_slice(&bytes)
        .map_err(|error| MeasureError::new(format!("managed {label} is malformed: {error}")))?;
    let raw: Value = serde_json::from_slice(&bytes)?;
    if serde_json::to_value(&record)? != raw {
        return Err(MeasureError::new(format!(
            "managed {label} does not exactly match its schema"
        )));
    }
    Ok(record)
}

pub fn read_jsonl_typed<T>(path: &Path, label: &str) -> Result<Vec<T>, MeasureError>
where
    T: DeserializeOwned + Serialize,
{
    parse_typed(read_lines(path, label)?, label)
}

pub fn read_jsonl_typed_file<T>(file: &mut File, label: &str) -> Result<Vec<T>, MeasureError>
where
    T: DeserializeOwned + Serialize,
{
    parse_typed(read_lines_file(file, label)?, label)
}

fn parse_typed<T>(lines: Vec<Vec<u8>>, label: &str) -> Result<Vec<T>, MeasureError>
where
    T: DeserializeOwned + Serialize,
{
    lines
        .into_iter()
        .map(|line| {
            let record: T = serde_json::from_slice(&line)
                .map_err(|_| MeasureError::new(format!("managed {label} has an invalid record")))?;
            let raw: Value = serde_json::from_slice(&line)?;
            if serde_json::to_value(&record)? != raw {
                return Err(MeasureError::new(format!(
                    "managed {label} has an invalid record"
                )));
            }
            Ok(record)
        })
        .collect()
}

fn read_lines(path: &Path, label: &str) -> Result<Vec<Vec<u8>>, MeasureError> {
    match std::fs::symlink_metadata(path) {
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.into()),
    }
    ensure_regular_file(path)?;
    let mut file = open_read(path)?;
    file.lock()?;
    read_lines_file(&mut file, label)
}

fn read_lines_file(file: &mut File, label: &str) -> Result<Vec<Vec<u8>>, MeasureError> {
    file.seek(SeekFrom::Start(0))?;
    let mut reader = BufReader::new(file);
    let mut records = Vec::new();
    loop {
        let mut line = Vec::new();
        let read = reader
            .by_ref()
            .take((MAX_RECORD_BYTES + 1) as u64)
            .read_until(b'\n', &mut line)?;
        if read == 0 {
            break;
        }
        if line.len() > MAX_RECORD_BYTES || !line.ends_with(b"\n") {
            return Err(MeasureError::new(format!(
                "managed {label} is truncated or oversized"
            )));
        }
        line.pop();
        records.push(line);
    }
    Ok(records)
}

fn open_read(path: &Path) -> Result<File, MeasureError> {
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32)
        .open(path)?;
    ensure_single_link(&file.metadata()?, path)?;
    Ok(file)
}
