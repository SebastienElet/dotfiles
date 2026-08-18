use super::super::MeasureError;
use super::super::store::MAX_RECORD_BYTES;
use super::super::store::validation::{ensure_regular_file, ensure_single_link};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

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

pub fn visit_jsonl_typed<T>(
    path: &Path,
    label: &str,
    visitor: impl FnMut(T) -> Result<(), MeasureError>,
) -> Result<(), MeasureError>
where
    T: DeserializeOwned + Serialize,
{
    match std::fs::symlink_metadata(path) {
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    }
    ensure_regular_file(path)?;
    let mut file = open_read(path)?;
    file.lock()?;
    visit_jsonl_typed_file(&mut file, label, visitor)
}

pub fn open_locked_jsonl(path: &Path) -> Result<Option<File>, MeasureError> {
    match std::fs::symlink_metadata(path) {
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    }
    ensure_regular_file(path)?;
    let file = open_read(path)?;
    file.lock()?;
    Ok(Some(file))
}

pub fn visit_jsonl_typed_file<T>(
    file: &mut File,
    label: &str,
    mut visitor: impl FnMut(T) -> Result<(), MeasureError>,
) -> Result<(), MeasureError>
where
    T: DeserializeOwned + Serialize,
{
    file.seek(SeekFrom::Start(0))?;
    let mut reader = BufReader::new(file);
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
        visitor(parse_typed(&line, label)?)?;
    }
    Ok(())
}

fn parse_typed<T>(line: &[u8], label: &str) -> Result<T, MeasureError>
where
    T: DeserializeOwned + Serialize,
{
    let record: T = serde_json::from_slice(line)
        .map_err(|_| MeasureError::new(format!("managed {label} has an invalid record")))?;
    let raw: Value = serde_json::from_slice(line)?;
    if serde_json::to_value(&record)? != raw {
        return Err(MeasureError::new(format!(
            "managed {label} has an invalid record"
        )));
    }
    Ok(record)
}

fn open_read(path: &Path) -> Result<File, MeasureError> {
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32)
        .open(path)?;
    ensure_single_link(&file.metadata()?, path)?;
    Ok(file)
}

#[cfg(test)]
mod tests {
    use super::visit_jsonl_typed;
    use serde::{Deserialize, Serialize};
    use std::io::Write;

    #[derive(Deserialize, Serialize)]
    struct Record {
        sequence: usize,
    }

    #[test]
    fn visits_a_long_journal_line_by_line() {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        for sequence in 0..20_000 {
            writeln!(file, "{{\"sequence\":{sequence}}}").unwrap();
        }
        let mut count = 0;
        let mut last = None;

        visit_jsonl_typed::<Record>(file.path(), "records.jsonl", |record| {
            count += 1;
            last = Some(record.sequence);
            Ok(())
        })
        .unwrap();

        assert_eq!(count, 20_000);
        assert_eq!(last, Some(19_999));
    }
}
