use super::super::super::MeasureError;
use serde_json::{Map, Value};

pub fn required_string(handler: &Map<String, Value>, field: &str) -> Result<(), MeasureError> {
    if !handler.get(field).is_some_and(Value::is_string) {
        return Err(invalid(field, "a string"));
    }
    Ok(())
}

pub fn optional_string(handler: &Map<String, Value>, field: &str) -> Result<(), MeasureError> {
    optional(handler, field, Value::is_string, "a string")
}

pub fn optional_bool(handler: &Map<String, Value>, field: &str) -> Result<(), MeasureError> {
    optional(handler, field, Value::is_boolean, "a boolean")
}

pub fn optional_number(handler: &Map<String, Value>, field: &str) -> Result<(), MeasureError> {
    optional(handler, field, Value::is_number, "a number")
}

pub fn optional_strings(handler: &Map<String, Value>, field: &str) -> Result<(), MeasureError> {
    optional(
        handler,
        field,
        |value| {
            value
                .as_array()
                .is_some_and(|values| values.iter().all(Value::is_string))
        },
        "an array of strings",
    )
}

pub fn optional_string_map(handler: &Map<String, Value>, field: &str) -> Result<(), MeasureError> {
    optional(
        handler,
        field,
        |value| {
            value
                .as_object()
                .is_some_and(|values| values.values().all(Value::is_string))
        },
        "an object of strings",
    )
}

pub fn optional_nonnegative_integer(
    handler: &Map<String, Value>,
    field: &str,
) -> Result<(), MeasureError> {
    optional(
        handler,
        field,
        |value| value.as_u64().is_some(),
        "a nonnegative integer",
    )
}

pub fn reject(handler: &Map<String, Value>, fields: &[&str]) -> Result<(), MeasureError> {
    if let Some(field) = fields.iter().find(|field| handler.contains_key(**field)) {
        return Err(MeasureError::new(format!(
            "hook handler field {field} is incompatible with its type"
        )));
    }
    Ok(())
}

pub fn max(handler: &Map<String, Value>, field: &str, maximum: f64) -> Result<(), MeasureError> {
    if handler
        .get(field)
        .and_then(Value::as_f64)
        .is_some_and(|value| value > maximum)
    {
        return Err(MeasureError::new(format!(
            "hook handler {field} must not exceed {maximum}"
        )));
    }
    Ok(())
}

pub fn invalid(field: &str, expected: &str) -> MeasureError {
    MeasureError::new(format!("hook handler {field} must be {expected}"))
}

fn optional(
    handler: &Map<String, Value>,
    field: &str,
    predicate: impl FnOnce(&Value) -> bool,
    expected: &str,
) -> Result<(), MeasureError> {
    if handler.get(field).is_some_and(|value| !predicate(value)) {
        return Err(invalid(field, expected));
    }
    Ok(())
}
