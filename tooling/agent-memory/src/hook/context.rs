use super::{HookError, output_unavailable};
use crate::{OmissionEffect, RetrievalReport, SourceKind};
use serde::Serialize;

const MAX_INJECTIONS: usize = 5;

pub(super) fn render(report: &RetrievalReport) -> Result<Option<String>, HookError> {
    if report.injected.is_empty() && report.omitted.is_empty() && report.omitted_by_limit == 0 {
        return Ok(None);
    }
    let extra = report.injected.len().saturating_sub(MAX_INJECTIONS);
    let payload = ContextPayload {
        injected: report
            .injected
            .iter()
            .take(MAX_INJECTIONS)
            .map(ContextInjection::from)
            .collect(),
        omitted: report.omitted.iter().map(ContextOmission::from).collect(),
        omitted_by_limit: report.omitted_by_limit.saturating_add(extra),
    };
    let json = serde_json::to_string(&payload).map_err(|_| output_unavailable())?;
    Ok(Some(format!("AGENT_MEMORY_CONTEXT_V1\n{json}")))
}

#[derive(Serialize)]
struct ContextPayload<'a> {
    injected: Vec<ContextInjection<'a>>,
    omitted: Vec<ContextOmission<'a>>,
    omitted_by_limit: usize,
}

#[derive(Serialize)]
struct ContextInjection<'a> {
    kind: crate::MemoryKind,
    statement: &'a str,
    sources: Vec<ContextSource<'a>>,
    verdict_age_milliseconds: u64,
}

impl<'a> From<&'a crate::InjectedMemory> for ContextInjection<'a> {
    fn from(memory: &'a crate::InjectedMemory) -> Self {
        Self {
            kind: memory.kind,
            statement: &memory.statement,
            sources: memory.sources.iter().map(ContextSource::from).collect(),
            verdict_age_milliseconds: memory.verdict_age_milliseconds,
        }
    }
}

#[derive(Serialize)]
struct ContextSource<'a> {
    kind: SourceKind,
    summary: &'a str,
}

impl<'a> From<&'a crate::SourceSummary> for ContextSource<'a> {
    fn from(source: &'a crate::SourceSummary) -> Self {
        Self {
            kind: source.kind,
            summary: source.locator.as_deref().unwrap_or("redacted"),
        }
    }
}

#[derive(Serialize)]
struct ContextOmission<'a> {
    code: &'a str,
    effect: OmissionEffect,
}

impl<'a> From<&'a crate::OmittedMemory> for ContextOmission<'a> {
    fn from(omission: &'a crate::OmittedMemory) -> Self {
        Self {
            code: &omission.code,
            effect: omission.effect,
        }
    }
}
