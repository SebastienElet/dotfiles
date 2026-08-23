use super::super::super::MeasureError;
use super::super::super::store::ManagedPath;
use super::super::io::{open_locked_jsonl, visit_jsonl_typed_file};
use super::{ResultRecord, StoredEvent, invalid_event};
use serde::Serialize;
use std::fs::File;

pub struct EventHistory {
    last_event: Option<String>,
    latest_result: Option<ResultRecord>,
    previous_result: Option<ResultRecord>,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResultState {
    Pending,
    Recorded,
    Missing,
    Lagging,
}

impl ResultState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Recorded => "recorded",
            Self::Missing => "missing",
            Self::Lagging => "lagging",
        }
    }
}

pub fn read_events_with<T>(
    path: &ManagedPath,
    run_id: &str,
    read_result: impl FnOnce() -> Result<T, MeasureError>,
) -> Result<(EventHistory, T), MeasureError> {
    let Some(mut file) = open_locked_jsonl(path)? else {
        return Err(MeasureError::new(
            "managed events.jsonl has an invalid record",
        ));
    };
    let history = read_events_file(&mut file, run_id)?;
    let result = read_result()?;
    Ok((history, result))
}

pub fn read_events_file(file: &mut File, run_id: &str) -> Result<EventHistory, MeasureError> {
    let mut history = EventHistory::new();
    visit_jsonl_typed_file::<StoredEvent>(file, "events.jsonl", |event| {
        history.push(event, run_id)
    })?;
    history.finish()
}

pub fn latest_result(history: &EventHistory) -> Option<&ResultRecord> {
    history.latest_result.as_ref()
}

fn previous_result(history: &EventHistory) -> Option<&ResultRecord> {
    history.previous_result.as_ref()
}

pub fn result_state(
    history: &EventHistory,
    result: Option<&ResultRecord>,
) -> Result<ResultState, MeasureError> {
    match (latest_result(history), result) {
        (None, None) => Ok(ResultState::Pending),
        (Some(latest), Some(result)) if latest == result => Ok(ResultState::Recorded),
        (Some(_), None) => Ok(ResultState::Missing),
        (Some(_), Some(result)) if previous_result(history) == Some(result) => {
            Ok(ResultState::Lagging)
        }
        _ => Err(MeasureError::new(
            "managed result.json diverges from result_recorded history",
        )),
    }
}

impl EventHistory {
    fn new() -> Self {
        Self {
            last_event: None,
            latest_result: None,
            previous_result: None,
        }
    }

    fn push(&mut self, event: StoredEvent, run_id: &str) -> Result<(), MeasureError> {
        if invalid_event(&event, run_id) {
            return Err(MeasureError::new(
                "managed events.jsonl has an invalid record",
            ));
        }
        if let Some(result) = event.result {
            self.push_result(result)?;
        }
        self.last_event = Some(event.event);
        Ok(())
    }

    fn push_result(&mut self, result: ResultRecord) -> Result<(), MeasureError> {
        let expected = match &self.latest_result {
            None => 1,
            Some(latest) => latest.revision.checked_add(1).ok_or_else(revision_error)?,
        };
        if result.revision != expected {
            return Err(revision_error());
        }
        self.previous_result = self.latest_result.take();
        self.latest_result = Some(result);
        Ok(())
    }

    fn finish(self) -> Result<Self, MeasureError> {
        if self.last_event.is_none() {
            return Err(MeasureError::new(
                "managed events.jsonl has an invalid record",
            ));
        }
        Ok(self)
    }

    pub fn last_event(&self) -> Option<&str> {
        self.last_event.as_deref()
    }
}

fn revision_error() -> MeasureError {
    MeasureError::new("result revisions must be unique and continuous")
}

#[cfg(test)]
mod tests {
    use super::read_events_with;
    use crate::measure::store::ManagedPath;
    use serde_json::json;
    use std::fs::OpenOptions;
    use std::io::Write;

    #[test]
    fn keeps_the_event_lock_while_reading_the_result_snapshot() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("events.jsonl");
        let event_id = "a".repeat(64);
        let mut file = std::fs::File::create(&path).unwrap();
        writeln!(
            file,
            "{}",
            json!({
                "timestamp_ms": 1,
                "event_id": event_id,
                "event": "prompt.submit",
                "native_event": "UserPromptSubmit",
                "artifact": "artifacts/hooks/event.json",
                "native_ids": {}
            })
        )
        .unwrap();

        let (_, result) = read_events_with(&ManagedPath::test_path(&path), &"b".repeat(64), || {
            let other = OpenOptions::new().read(true).write(true).open(&path)?;
            assert!(other.try_lock().is_err());
            Ok(7)
        })
        .unwrap();

        assert_eq!(result, 7);
    }
}
