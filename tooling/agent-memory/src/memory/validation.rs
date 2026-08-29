use super::error::MemoryError;
use super::model::{
    AdmissionAuthorization, AdmissionDraft, EntryData, EntryProof, EntryScope, EntrySource,
    EntryTransition, Fingerprint, MemoryEntry, MemoryId, ProjectKey, RawDraftKind, RawEntryKind,
    RawEntryScope, RetrievalTerm, SourceKind, Statement, UtcTimestamp, ValidatedDraft,
    ValidatedDraftProof, ValidatedDraftSource, ValidatedOracle,
};
use super::sensitive;
use serde::de::DeserializeOwned;
use serde_yaml_ng::{Mapping, Value};

const MAX_INPUT_BYTES: usize = 1024 * 1024;
const MAX_RETRIEVAL_TERMS: usize = 20;
const MAX_SOURCES: usize = 20;

pub fn parse_draft(bytes: &[u8]) -> Result<AdmissionDraft, MemoryError> {
    let input = checked_input(bytes)?;
    let value = parse_value(input)?;
    validate_schema_version(&value)?;
    validate_source_kinds(&value)?;
    let raw: RawDraftKind = deserialize(input)?;
    let (kind, data) = raw.split();
    Ok(AdmissionDraft { kind, data })
}

pub fn validate_draft(
    draft: AdmissionDraft,
    authorization: AdmissionAuthorization,
) -> Result<ValidatedDraft, MemoryError> {
    if authorization == AdmissionAuthorization::ImplicitProposal {
        return Err(MemoryError::new(
            "admission_not_authorized",
            "authorization",
        ));
    }
    let data = draft.data;
    validate_schema_number(data.schema_version)?;
    let statement = statement(data.statement)?;
    let retrieval_terms = retrieval_terms(data.retrieval_terms)?;
    validate_count(data.proof.sources.len(), MAX_SOURCES, "proof.sources")?;
    if data.proof.sources.is_empty() {
        return Err(MemoryError::new("missing_proof", "proof.sources"));
    }
    validate_text(&data.proof.summary, 1, 1000, "proof.summary")?;
    reject_sensitive(&data.proof.summary, "proof.summary")?;
    let sources = data
        .proof
        .sources
        .into_iter()
        .map(|source| {
            let (kind, locator) = source.split();
            reject_sensitive(&locator, "proof.sources.locator")?;
            Ok(ValidatedDraftSource::new(kind, locator))
        })
        .collect::<Result<Vec<_>, MemoryError>>()?;
    validate_oracle_requirement(
        data.oracle.automated.is_some(),
        sources
            .iter()
            .all(|source| source.kind() == SourceKind::UserDecision),
    )?;
    let oracle = validated_oracle(data.oracle)?;
    Ok(ValidatedDraft::new(
        draft.kind,
        statement,
        data.scope,
        retrieval_terms,
        ValidatedDraftProof::new(data.proof.summary, sources),
        oracle,
    ))
}

pub fn parse_entry(bytes: &[u8]) -> Result<MemoryEntry, MemoryError> {
    let input = checked_input(bytes)?;
    let value = parse_value(input)?;
    validate_schema_version(&value)?;
    validate_source_kinds(&value)?;
    validate_kind_status_transition(&value)?;
    let raw: RawEntryKind = deserialize(input)?;
    validated_entry(raw)
}

pub fn parse_utc_timestamp(value: &str) -> Result<UtcTimestamp, MemoryError> {
    utc_timestamp(value.to_owned())
}

fn validated_entry(raw: RawEntryKind) -> Result<MemoryEntry, MemoryError> {
    let (kind, data) = raw.split();
    validate_schema_number(data.schema_version)?;
    let id = memory_id(data.id)?;
    let statement = statement(data.statement)?;
    let scope = match data.scope {
        RawEntryScope::Project { key } => EntryScope::Project(project_key(key)?),
        RawEntryScope::User => EntryScope::User,
    };
    let retrieval_terms = retrieval_terms(data.retrieval_terms)?;
    let proof = validated_entry_proof(data.proof)?;
    validate_oracle_requirement(
        data.oracle.automated.is_some(),
        proof
            .sources()
            .iter()
            .all(|source| source.kind() == SourceKind::UserDecision),
    )?;
    let oracle = validated_oracle(data.oracle)?;
    let transition = data.transition.map(validated_transition).transpose()?;
    Ok(MemoryEntry::new(
        kind,
        EntryData {
            id,
            status: data.status,
            statement,
            scope,
            retrieval_terms,
            proof,
            oracle,
            created_at: utc_timestamp(data.created_at)?,
            transition,
        },
    ))
}

fn validated_entry_proof(proof: super::model::RawEntryProof) -> Result<EntryProof, MemoryError> {
    validate_count(proof.sources.len(), MAX_SOURCES, "proof.sources")?;
    if proof.sources.is_empty() {
        return Err(MemoryError::new("missing_proof", "proof.sources"));
    }
    validate_text(&proof.summary, 1, 1000, "proof.summary")?;
    reject_sensitive(&proof.summary, "proof.summary")?;
    let sources = proof
        .sources
        .into_iter()
        .map(|source| {
            let (kind, locator, fingerprint_value) = source.split();
            reject_sensitive(&locator, "proof.sources.locator")?;
            Ok(EntrySource::new(
                kind,
                locator,
                fingerprint(fingerprint_value)?,
            ))
        })
        .collect::<Result<Vec<_>, MemoryError>>()?;
    Ok(EntryProof::new(
        proof.summary,
        sources,
        utc_timestamp(proof.established_at)?,
    ))
}

fn validated_transition(
    transition: super::model::RawTransition,
) -> Result<EntryTransition, MemoryError> {
    validate_transition_reason(&transition.reason)?;
    Ok(EntryTransition::new(
        transition.from,
        transition.to,
        utc_timestamp(transition.at)?,
        transition.verdict,
        transition.reason,
    ))
}

pub(crate) fn validate_transition_reason(value: &str) -> Result<(), MemoryError> {
    if value.trim().is_empty() {
        return Err(MemoryError::new(
            "invalid_transition_reason",
            "transition.reason",
        ));
    }
    validate_text(value, 1, 500, "transition.reason")?;
    reject_sensitive(value, "transition.reason")
}

fn validated_oracle(oracle: super::model::RawOracle) -> Result<ValidatedOracle, MemoryError> {
    let text_fields = [
        (
            "oracle.human_fallback.question",
            &oracle.human_fallback.question,
        ),
        (
            "oracle.human_fallback.valid_when",
            &oracle.human_fallback.valid_when,
        ),
        ("oracle.outcomes.valid", &oracle.outcomes.valid),
        ("oracle.outcomes.invalidated", &oracle.outcomes.invalidated),
    ];
    for (field, value) in text_fields {
        validate_text(value, 1, 500, field)?;
        reject_sensitive(value, field)?;
    }
    Ok(ValidatedOracle::new(
        oracle.automated,
        oracle.human_fallback,
        oracle.outcomes,
    ))
}

fn checked_input(bytes: &[u8]) -> Result<&str, MemoryError> {
    if bytes.len() > MAX_INPUT_BYTES {
        return Err(MemoryError::new("input_too_large", "document"));
    }
    std::str::from_utf8(bytes).map_err(|_| MemoryError::new("invalid_utf8", "document"))
}

fn parse_value(input: &str) -> Result<Value, MemoryError> {
    serde_yaml_ng::from_str(input).map_err(|error| {
        if error.to_string().to_ascii_lowercase().contains("duplicate") {
            MemoryError::new("duplicate_field", "document")
        } else {
            MemoryError::new("malformed_yaml", "document")
        }
    })
}

fn deserialize<T: DeserializeOwned>(input: &str) -> Result<T, MemoryError> {
    let deserializer = serde_yaml_ng::Deserializer::from_str(input);
    serde_path_to_error::deserialize(deserializer).map_err(|error| {
        let message = error.inner().to_string().to_ascii_lowercase();
        if message.contains("unknown field") {
            MemoryError::new("unknown_field", "document")
        } else if message.contains("unknown variant") && error.path().to_string().contains("kind") {
            MemoryError::new("invalid_source_kind", "proof.sources.kind")
        } else {
            MemoryError::new("invalid_field", "document")
        }
    })
}

fn validate_schema_version(value: &Value) -> Result<(), MemoryError> {
    let version = root_mapping(value)?
        .get(Value::String("schema_version".to_owned()))
        .and_then(Value::as_u64)
        .ok_or_else(|| MemoryError::new("invalid_field", "schema_version"))?;
    validate_schema_number(version)
}

fn validate_schema_number(version: u64) -> Result<(), MemoryError> {
    if version == 1 {
        Ok(())
    } else {
        Err(MemoryError::new("unsupported_schema", "schema_version"))
    }
}

fn validate_source_kinds(value: &Value) -> Result<(), MemoryError> {
    let Some(sources) = nested(root_mapping(value)?, "proof")
        .and_then(Value::as_mapping)
        .and_then(|proof| nested(proof, "sources"))
        .and_then(Value::as_sequence)
    else {
        return Ok(());
    };
    for source in sources {
        let kind = source
            .as_mapping()
            .and_then(|source| nested(source, "kind"))
            .and_then(Value::as_str)
            .ok_or_else(|| MemoryError::new("invalid_field", "proof.sources.kind"))?;
        if !matches!(
            kind,
            "git-file" | "local-file" | "official-url" | "user-decision"
        ) {
            return Err(MemoryError::new(
                "invalid_source_kind",
                "proof.sources.kind",
            ));
        }
    }
    Ok(())
}

fn validate_kind_status_transition(value: &Value) -> Result<(), MemoryError> {
    let root = root_mapping(value)?;
    let kind = required_string(root, "kind")?;
    let status = required_string(root, "status")?;
    if !status_allowed(kind, status) {
        return Err(MemoryError::new("invalid_kind_status", "status"));
    }
    let transition = nested(root, "transition");
    if status == "active" && transition.is_some() {
        return Err(MemoryError::new("unexpected_transition", "transition"));
    }
    if status != "active" && transition.is_none() {
        return Err(MemoryError::new("missing_transition", "transition"));
    }
    if let Some(transition) = transition {
        validate_transition_value(status, transition)?;
    }
    Ok(())
}

fn validate_transition_value(status: &str, transition: &Value) -> Result<(), MemoryError> {
    let transition = transition
        .as_mapping()
        .ok_or_else(|| MemoryError::new("invalid_transition", "transition"))?;
    let from = required_string(transition, "from")?;
    let to = required_string(transition, "to")?;
    let verdict = required_string(transition, "verdict")?;
    let expected_verdict = if status == "invalidated" {
        "invalid"
    } else {
        "valid"
    };
    if from != "active" || to != status || verdict != expected_verdict {
        return Err(MemoryError::new("invalid_transition", "transition"));
    }
    Ok(())
}

fn status_allowed(kind: &str, status: &str) -> bool {
    match kind {
        "goal" => matches!(status, "active" | "achieved" | "abandoned" | "invalidated"),
        "decision" => matches!(status, "active" | "superseded" | "invalidated"),
        "evidence" | "invariant" => matches!(status, "active" | "invalidated"),
        "unknown" => matches!(status, "active" | "resolved" | "invalidated"),
        "assumption" => matches!(status, "active" | "confirmed" | "invalidated"),
        _ => false,
    }
}

fn statement(value: String) -> Result<Statement, MemoryError> {
    validate_text(&value, 1, 500, "statement")?;
    reject_sensitive(&value, "statement")?;
    Ok(Statement::from_validated(value))
}

fn retrieval_terms(values: Vec<String>) -> Result<Vec<RetrievalTerm>, MemoryError> {
    validate_count(values.len(), MAX_RETRIEVAL_TERMS, "retrieval_terms")?;
    if values.is_empty() {
        return Err(MemoryError::new("invalid_field", "retrieval_terms"));
    }
    values
        .into_iter()
        .map(|value| {
            validate_text(&value, 1, 100, "retrieval_terms")?;
            reject_sensitive(&value, "retrieval_terms")?;
            Ok(RetrievalTerm::from_validated(value))
        })
        .collect()
}

fn memory_id(value: String) -> Result<MemoryId, MemoryError> {
    if value
        .strip_prefix("mem_")
        .is_some_and(|suffix| suffix.len() == 24 && is_lower_hex(suffix))
    {
        Ok(MemoryId::from_validated(value))
    } else {
        Err(MemoryError::new("invalid_field", "id"))
    }
}

fn project_key(value: String) -> Result<ProjectKey, MemoryError> {
    if value
        .strip_prefix("project_")
        .is_some_and(|suffix| suffix.len() == 64 && is_lower_hex(suffix))
    {
        Ok(ProjectKey::from_validated(value))
    } else {
        Err(MemoryError::new("invalid_field", "scope.key"))
    }
}

fn fingerprint(value: String) -> Result<Fingerprint, MemoryError> {
    if value
        .strip_prefix("sha256:")
        .is_some_and(|suffix| suffix.len() == 64 && is_lower_hex(suffix))
    {
        Ok(Fingerprint::from_validated(value))
    } else {
        Err(MemoryError::new(
            "invalid_field",
            "proof.sources.fingerprint",
        ))
    }
}

fn utc_timestamp(value: String) -> Result<UtcTimestamp, MemoryError> {
    if valid_utc_timestamp(&value) {
        Ok(UtcTimestamp::from_validated(value))
    } else {
        Err(MemoryError::new("invalid_field", "timestamp"))
    }
}

fn valid_utc_timestamp(value: &str) -> bool {
    let Some(timestamp) = value.strip_suffix('Z') else {
        return false;
    };
    let Some((date, time)) = timestamp.split_once('T') else {
        return false;
    };
    let date_parts = date.split('-').collect::<Vec<_>>();
    let time = time.split_once('.').map_or(time, |(whole, fraction)| {
        if fraction.is_empty() || !fraction.bytes().all(|byte| byte.is_ascii_digit()) {
            ""
        } else {
            whole
        }
    });
    let time_parts = time.split(':').collect::<Vec<_>>();
    if date_parts.len() != 3 || time_parts.len() != 3 {
        return false;
    }
    let Some(year) = fixed_number(date_parts[0], 4) else {
        return false;
    };
    let Some(month) = fixed_number(date_parts[1], 2) else {
        return false;
    };
    let Some(day) = fixed_number(date_parts[2], 2) else {
        return false;
    };
    let Some(hour) = fixed_number(time_parts[0], 2) else {
        return false;
    };
    let Some(minute) = fixed_number(time_parts[1], 2) else {
        return false;
    };
    let Some(second) = fixed_number(time_parts[2], 2) else {
        return false;
    };
    year > 0
        && (1..=12).contains(&month)
        && (1..=days_in_month(year, month)).contains(&day)
        && hour <= 23
        && minute <= 59
        && second <= 59
}

fn fixed_number(value: &str, length: usize) -> Option<u32> {
    (value.len() == length && value.bytes().all(|byte| byte.is_ascii_digit()))
        .then(|| value.parse().ok())
        .flatten()
}

fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        4 | 6 | 9 | 11 => 30,
        2 if year.is_multiple_of(400) || (year.is_multiple_of(4) && !year.is_multiple_of(100)) => {
            29
        }
        2 => 28,
        _ => 31,
    }
}

fn is_lower_hex(value: &str) -> bool {
    value
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn validate_text(
    value: &str,
    minimum: usize,
    maximum: usize,
    field: &'static str,
) -> Result<(), MemoryError> {
    let length = value.chars().count();
    if (minimum..=maximum).contains(&length) {
        Ok(())
    } else {
        Err(MemoryError::new("invalid_field", field))
    }
}

fn validate_count(count: usize, maximum: usize, field: &'static str) -> Result<(), MemoryError> {
    if count <= maximum {
        Ok(())
    } else {
        Err(MemoryError::new("too_many_items", field))
    }
}

fn validate_oracle_requirement(
    has_automated: bool,
    all_user_decisions: bool,
) -> Result<(), MemoryError> {
    if has_automated || all_user_decisions {
        Ok(())
    } else {
        Err(MemoryError::new("missing_oracle", "oracle.automated"))
    }
}

fn reject_sensitive(value: &str, field: &'static str) -> Result<(), MemoryError> {
    if sensitive::contains_sensitive(value) {
        Err(MemoryError::new("sensitive_content", field))
    } else {
        Ok(())
    }
}

fn root_mapping(value: &Value) -> Result<&Mapping, MemoryError> {
    value
        .as_mapping()
        .ok_or_else(|| MemoryError::new("invalid_field", "document"))
}

fn nested<'a>(mapping: &'a Mapping, key: &str) -> Option<&'a Value> {
    mapping.get(Value::String(key.to_owned()))
}

fn required_string<'a>(mapping: &'a Mapping, key: &'static str) -> Result<&'a str, MemoryError> {
    nested(mapping, key)
        .and_then(Value::as_str)
        .ok_or_else(|| MemoryError::new("invalid_field", key))
}
