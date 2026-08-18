use super::model::EventRecord;
use super::redaction::redact_string;
use serde_json::{Map, Value};

pub fn record(
    timestamp_ms: u64,
    event_id: &str,
    artifact: String,
    native_ids: Map<String, Value>,
    raw: &Value,
) -> EventRecord {
    let native_event = native_event(raw);
    EventRecord {
        timestamp_ms,
        event_id: event_id.to_owned(),
        event: normalized(&native_event).to_owned(),
        native_event,
        artifact,
        native_ids,
    }
}

fn native_event(value: &Value) -> String {
    let event = ["hook_event_name", "event", "type"]
        .iter()
        .find_map(|key| value.get(key).and_then(Value::as_str))
        .unwrap_or("unknown");
    redact_string(event)
}

fn normalized(native: &str) -> &'static str {
    let compact: String = native
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect();
    match compact.as_str() {
        "sessionstart" => "session.start",
        "sessionend" => "session.end",
        "userpromptsubmit" | "beforesubmitprompt" => "prompt.submit",
        "stop" => "agent.stop",
        "subagentstart" => "subagent.start",
        "subagentstop" => "subagent.stop",
        "permissionrequest" => "permission.request",
        "permissiondenied" => "permission.denied",
        "pretooluse" => "tool.before",
        "posttooluse" => "tool.after",
        "posttoolusefailure" => "tool.failure",
        "stopfailure" => "agent.failure",
        "precompact" => "context.compact.before",
        "postcompact" => "context.compact.after",
        "beforefileedit"
        | "beforereadfile"
        | "beforetabfileread"
        | "beforeshellexecution"
        | "beforemcpexecution" => "tool.before",
        "afterfileedit" | "aftershellexecution" | "aftermcpexecution" => "tool.after",
        "afteragentresponse" => "agent.response",
        _ => "unknown",
    }
}
