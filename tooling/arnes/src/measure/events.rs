use super::model::EventRecord;
use serde_json::Value;

pub fn record(timestamp_ms: u64, raw: &Value) -> EventRecord {
    EventRecord {
        schema_version: 2,
        timestamp_ms,
        event: normalized(native_event(raw)).to_owned(),
    }
}

fn native_event(value: &Value) -> &str {
    ["hook_event_name", "event", "type"]
        .iter()
        .find_map(|key| value.get(key).and_then(Value::as_str))
        .unwrap_or("unknown")
}

fn normalized(native: &str) -> &'static str {
    let compact: String = native
        .chars()
        .filter(char::is_ascii_alphanumeric)
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
        "pretooluse"
        | "beforefileedit"
        | "beforereadfile"
        | "beforetabfileread"
        | "beforeshellexecution"
        | "beforemcpexecution" => "tool.before",
        "posttooluse" | "afterfileedit" | "aftershellexecution" | "aftermcpexecution" => {
            "tool.after"
        }
        "posttoolusefailure" => "tool.failure",
        "stopfailure" => "agent.failure",
        "precompact" => "context.compact.before",
        "postcompact" => "context.compact.after",
        "afteragentresponse" => "agent.response",
        _ => "unknown",
    }
}
