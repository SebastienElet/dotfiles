use crate::{Diagnostic, MemoryError};
use serde_path_to_error::Segment;
use serde_yaml_ng::Value;

const FIELDS: &[&str] = &[
    "schema_version",
    "kind",
    "statement",
    "scope",
    "retrieval_terms",
    "proof",
    "proof.summary",
    "proof.sources",
    "proof.sources.kind",
    "proof.sources.locator",
    "oracle",
    "oracle.automated",
    "oracle.automated.kind",
    "oracle.automated.expected",
    "oracle.human_fallback",
    "oracle.human_fallback.question",
    "oracle.human_fallback.valid_when",
    "oracle.outcomes",
    "oracle.outcomes.valid",
    "oracle.outcomes.invalidated",
    "id",
    "status",
    "scope.key",
    "proof.sources.fingerprint",
    "proof.established_at",
    "created_at",
    "transition",
    "transition.from",
    "transition.to",
    "transition.at",
    "transition.verdict",
    "transition.reason",
];

pub(super) fn yaml_error(code: &'static str, error: &serde_yaml_ng::Error) -> MemoryError {
    located(MemoryError::new(code, "document"), error)
}

pub(super) fn draft_error(mut value: Value, fallback: MemoryError) -> MemoryError {
    let Some(root) = value.as_mapping_mut() else {
        return fallback;
    };
    let mut kind = root
        .remove(Value::String("kind".to_owned()))
        .unwrap_or(Value::Null);
    while let Value::Tagged(tagged) = kind {
        kind = tagged.value;
    }
    if serde_yaml_ng::from_value::<crate::MemoryKind>(kind).is_err() {
        return MemoryError::new("invalid_field", "kind");
    }
    if let Some(error) = tagged_field(&value, "") {
        return error;
    }
    serde_path_to_error::deserialize::<_, crate::memory::model::RawDraftData>(value)
        .err()
        .map(deserialize_error)
        .unwrap_or(fallback)
}

pub(super) fn deserialize_error(
    error: serde_path_to_error::Error<serde_yaml_ng::Error>,
) -> MemoryError {
    let mut path = String::new();
    let mut index = None;
    for segment in error.path() {
        match segment {
            Segment::Map { key } => {
                let Some(field) = child_field(&path, key) else {
                    break;
                };
                path = field.to_owned();
            }
            Segment::Seq { index: position } => index = Some(*position),
            _ => {}
        }
    }
    let message = error.inner().to_string();
    let (code, field) = classify(&path, &message);
    let mut failure = MemoryError::new(code, field);
    if code == "unknown_field" {
        failure = failure.with_message(allowed_fields(field));
    }
    if let Some(index) = index {
        failure = failure.at_item(index);
    }
    located(failure, error.inner())
}

fn child_field(parent: &str, key: &str) -> Option<&'static str> {
    FIELDS.iter().copied().find(|field| {
        let (prefix, name) = field.rsplit_once('.').unwrap_or(("", field));
        prefix == parent && name == key
    })
}

fn tagged_field(value: &Value, field: &'static str) -> Option<MemoryError> {
    match value {
        Value::Tagged(_) => Some(MemoryError::new("invalid_field", field).with_message(
            "Remove custom YAML tags from this field; use the plain type required by the entry contract.",
        )),
        Value::Mapping(mapping) => mapping.iter().find_map(|(key, value)| {
            let child = child_field(field, key.as_str()?)?;
            tagged_field(value, child)
        }),
        Value::Sequence(values) => values.iter().enumerate().find_map(|(index, value)| {
            tagged_field(value, field).map(|error| error.at_item(index))
        }),
        _ => None,
    }
}

fn classify(path: &str, message: &str) -> (&'static str, &'static str) {
    let field = FIELDS
        .iter()
        .copied()
        .filter(|field| {
            *field == path
                || path
                    .strip_prefix(field)
                    .is_some_and(|suffix| suffix.starts_with('.'))
        })
        .max_by_key(|field| field.len())
        .unwrap_or("document");
    if message.starts_with("unknown field") || message.contains(": unknown field") {
        return ("unknown_field", field);
    }
    for candidate in FIELDS {
        let key = candidate.rsplit('.').next().unwrap();
        let parent = candidate.rsplit_once('.').map_or("", |(parent, _)| parent);
        if parent == path && message.contains(&format!("missing field `{key}`")) {
            return ("invalid_field", candidate);
        }
    }
    if field == "oracle.automated" && message.contains("all-proof-sources-unchanged") {
        return ("invalid_field", "oracle.automated.expected");
    }
    if field == "proof.sources" && message.contains("expected a string") {
        return ("invalid_field", "proof.sources.locator");
    }
    ("invalid_field", field)
}

fn located(error: MemoryError, yaml: &serde_yaml_ng::Error) -> MemoryError {
    let Some(location) = yaml.location() else {
        return error;
    };
    let diagnostic = Diagnostic {
        line: Some(location.line()),
        column: Some(location.column()),
        ..error.diagnostic()
    };
    error.with_diagnostic(diagnostic)
}

fn allowed_fields(field: &str) -> &'static str {
    match field {
        "proof" => {
            "Remove extra proof keys; allowed keys are summary and sources (plus established_at only in stored entries)."
        }
        "proof.sources" => {
            "Remove extra source keys; allowed keys are kind and locator (plus fingerprint only in stored entries)."
        }
        "oracle" => {
            "Remove extra oracle keys; allowed keys are automated, human_fallback, outcomes."
        }
        "oracle.automated" => {
            "Remove extra automated oracle keys; allowed keys are kind and expected."
        }
        "oracle.human_fallback" => {
            "Remove extra human_fallback keys; allowed keys are question and valid_when."
        }
        "oracle.outcomes" => "Remove extra outcomes keys; allowed keys are valid and invalidated.",
        _ => {
            "Remove extra draft keys; allowed top-level keys are schema_version, kind, statement, scope, retrieval_terms, proof, oracle. Runtime-assigned fields must be omitted."
        }
    }
}
