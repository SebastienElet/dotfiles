use super::{HookError, output_unavailable};
use crate::{InjectedMemory, OmissionEffect, OmittedMemory, RetrievalReport, SourceKind};
use serde::Serialize;

const CONTEXT_HEADER: &str = "AGENT_MEMORY_CONTEXT_V1\n";
const MAX_CONTEXT_BYTES: usize = 2000;
const MAX_INJECTIONS: usize = 5;
const MAX_SOURCES_PER_INJECTION: usize = 20;
const MAX_OMISSION_CANDIDATES: usize = 20;

pub(super) fn render(report: &RetrievalReport) -> Result<Option<String>, HookError> {
    if report.injected.is_empty() && report.omitted.is_empty() && report.omitted_by_limit == 0 {
        return Ok(None);
    }
    let candidates = report
        .injected
        .iter()
        .take(MAX_INJECTIONS)
        .collect::<Vec<_>>();
    let mut injections = Vec::new();
    for candidate in &candidates {
        if candidate.sources.len() > MAX_SOURCES_PER_INJECTION {
            continue;
        }
        let mut tentative = injections.clone();
        tentative.push(*candidate);
        if serialized_context(report, &candidates, &tentative, &[]).is_ok() {
            injections = tentative;
        }
    }
    let omission_candidates = report
        .omitted
        .iter()
        .take(MAX_OMISSION_CANDIDATES)
        .collect::<Vec<_>>();
    let mut omissions = Vec::new();
    for candidate in omission_candidates {
        let mut tentative = omissions.clone();
        tentative.push(candidate);
        if serialized_context(report, &candidates, &injections, &tentative).is_ok() {
            omissions = tentative;
        }
    }
    serialized_context(report, &candidates, &injections, &omissions).map(Some)
}

fn serialized_context(
    report: &RetrievalReport,
    candidates: &[&InjectedMemory],
    injections: &[&InjectedMemory],
    omissions: &[&OmittedMemory],
) -> Result<String, HookError> {
    let payload = ContextPayload {
        injected: injections
            .iter()
            .map(|memory| ContextInjection::from(*memory))
            .collect(),
        omitted: omissions
            .iter()
            .map(|omission| ContextOmission::from(*omission))
            .collect(),
        omitted_counts: ContextOmittedCounts {
            retrieval_limit: report.omitted_by_limit,
            injection_limit: report.injected.len().saturating_sub(MAX_INJECTIONS),
            context_injections: candidates.len().saturating_sub(injections.len()),
            context_omissions: report.omitted.len().saturating_sub(omissions.len()),
        },
    };
    let json = serde_json::to_string(&payload).map_err(|_| output_unavailable())?;
    let context = format!("{CONTEXT_HEADER}{json}");
    if context.len() > MAX_CONTEXT_BYTES {
        return Err(output_unavailable());
    }
    Ok(context)
}

#[derive(Serialize)]
struct ContextPayload<'a> {
    injected: Vec<ContextInjection<'a>>,
    omitted: Vec<ContextOmission<'a>>,
    omitted_counts: ContextOmittedCounts,
}

#[derive(Serialize)]
struct ContextOmittedCounts {
    retrieval_limit: usize,
    injection_limit: usize,
    context_injections: usize,
    context_omissions: usize,
}

#[derive(Serialize)]
struct ContextInjection<'a> {
    kind: crate::MemoryKind,
    statement: &'a str,
    sources: Vec<ContextSource<'a>>,
    verdict_age_milliseconds: u64,
}

impl<'a> From<&'a InjectedMemory> for ContextInjection<'a> {
    fn from(memory: &'a InjectedMemory) -> Self {
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

impl<'a> From<&'a OmittedMemory> for ContextOmission<'a> {
    fn from(omission: &'a OmittedMemory) -> Self {
        Self {
            code: &omission.code,
            effect: omission.effect,
        }
    }
}
