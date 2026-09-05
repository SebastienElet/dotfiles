use super::super::super::MeasureError;
use super::super::super::store::ManagedPath;
use super::super::io::{open_locked_jsonl, visit_jsonl_typed_file};
use super::{ResultRecord, StoredEvent, invalid_event};
use serde::Serialize;
use std::fs::File;

pub struct EventHistory {
    last_event: Option<String>,
    first_event_at_ms: Option<u64>,
    last_event_at_ms: Option<u64>,
    timestamps_consistent: bool,
    event_count: u64,
    tool_call_count: u64,
    latest_result: Option<ResultRecord>,
    previous_result: Option<ResultRecord>,
}

#[derive(Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResultState {
    Pending,
    OutcomeRecorded,
    Recorded,
    Missing,
    Lagging,
}

impl ResultState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::OutcomeRecorded => "outcome-recorded",
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

pub fn read_events_for_list_with<T>(
    path: &ManagedPath,
    run_id: &str,
    read_result: impl FnOnce() -> Result<T, MeasureError>,
) -> Result<(EventHistory, T), MeasureError> {
    match open_locked_jsonl(path)? {
        Some(mut file) => {
            let history = read_events_file_allow_empty(&mut file, run_id)?;
            let result = read_result()?;
            Ok((history, result))
        }
        None => Ok((EventHistory::new(), read_result()?)),
    }
}

pub fn read_events_file(file: &mut File, run_id: &str) -> Result<EventHistory, MeasureError> {
    read_events_file_allow_empty(file, run_id)?.finish()
}

fn read_events_file_allow_empty(
    file: &mut File,
    run_id: &str,
) -> Result<EventHistory, MeasureError> {
    let mut history = EventHistory::new();
    visit_jsonl_typed_file::<StoredEvent>(file, "events.jsonl", |event| {
        history.push(event, run_id)
    })?;
    Ok(history)
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
            first_event_at_ms: None,
            last_event_at_ms: None,
            timestamps_consistent: true,
            event_count: 0,
            tool_call_count: 0,
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
        if let Some(result) = event.result().cloned() {
            self.push_result(result)?;
        }
        let timestamp_ms = event.timestamp_ms();
        self.event_count = self
            .event_count
            .checked_add(1)
            .ok_or_else(|| MeasureError::new("event count overflow"))?;
        if event.event() == "tool.before" {
            self.tool_call_count = self
                .tool_call_count
                .checked_add(1)
                .ok_or_else(|| MeasureError::new("tool call count overflow"))?;
        }
        self.first_event_at_ms.get_or_insert(timestamp_ms);
        self.timestamps_consistent &= self
            .last_event_at_ms
            .is_none_or(|previous| previous <= timestamp_ms);
        self.last_event_at_ms = Some(timestamp_ms);
        self.last_event = Some(event.event().to_owned());
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

    pub fn first_event_at_ms(&self) -> Option<u64> {
        self.first_event_at_ms
    }

    pub fn last_event_at_ms(&self) -> Option<u64> {
        self.last_event_at_ms
    }

    pub fn timestamps_consistent(&self) -> bool {
        self.timestamps_consistent
    }

    pub fn event_count(&self) -> u64 {
        self.event_count
    }

    pub fn tool_call_count(&self) -> u64 {
        self.tool_call_count
    }
}

fn revision_error() -> MeasureError {
    MeasureError::new("result revisions must be unique and continuous")
}

#[cfg(test)]
mod tests;
