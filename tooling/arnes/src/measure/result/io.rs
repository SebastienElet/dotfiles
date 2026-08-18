use super::super::MeasureError;
use super::super::store::{MAX_RECORD_BYTES, ManagedPath};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};

pub fn read_optional_json<T>(path: &ManagedPath, label: &str) -> Result<Option<T>, MeasureError>
where
    T: DeserializeOwned + Serialize,
{
    if path.exists()? {
        read_json(path, label).map(Some)
    } else {
        Ok(None)
    }
}

pub fn read_json<T>(path: &ManagedPath, label: &str) -> Result<T, MeasureError>
where
    T: DeserializeOwned + Serialize,
{
    let mut file = path.open_read()?;
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
    path: &ManagedPath,
    label: &str,
    visitor: impl FnMut(T) -> Result<(), MeasureError>,
) -> Result<(), MeasureError>
where
    T: DeserializeOwned + Serialize,
{
    if !path.exists()? {
        return Ok(());
    }
    let mut file = path.open_read()?;
    file.lock()?;
    visit_jsonl_typed_file(&mut file, label, visitor)
}

pub fn open_locked_jsonl(path: &ManagedPath) -> Result<Option<File>, MeasureError> {
    if !path.exists()? {
        return Ok(None);
    }
    let file = path.open_read()?;
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

#[cfg(test)]
mod tests {
    use super::visit_jsonl_typed;
    use crate::measure::store::ManagedPath;
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

        visit_jsonl_typed::<Record>(
            &ManagedPath::test_path(file.path()),
            "records.jsonl",
            |record| {
                count += 1;
                last = Some(record.sequence);
                Ok(())
            },
        )
        .unwrap();

        assert_eq!(count, 20_000);
        assert_eq!(last, Some(19_999));
    }
}
