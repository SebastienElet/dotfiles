use super::super::{InjectedMemory, SourceSummary};
use crate::memory::clock::timestamp;
use crate::memory::{MemoryEntry, OracleEvaluation, SourceKind};
use url::Url;

pub(super) fn injected(
    entry: &MemoryEntry,
    evaluation: &OracleEvaluation,
) -> Option<InjectedMemory> {
    let validated_at = timestamp(evaluation.validated_at()?)?;
    let now = timestamp(evaluation.evaluated_at())?;
    let age = now.duration_since(validated_at);
    let verdict_age_milliseconds = u64::try_from(age.as_millis()).ok()?;
    Some(InjectedMemory {
        id: entry.id().as_str().to_owned(),
        kind: entry.kind(),
        statement: entry.statement().as_str().to_owned(),
        sources: source_summaries(entry)?,
        verdict_age_milliseconds,
    })
}

fn source_summaries(entry: &MemoryEntry) -> Option<Vec<SourceSummary>> {
    entry
        .proof()
        .sources()
        .iter()
        .map(|source| match source.kind() {
            SourceKind::GitFile => Some(SourceSummary::with_locator(
                SourceKind::GitFile,
                source.locator(),
            )),
            SourceKind::OfficialUrl => {
                let mut url = Url::parse(source.locator()).ok()?;
                url.set_query(None);
                url.set_fragment(None);
                Some(SourceSummary::with_locator(SourceKind::OfficialUrl, url))
            }
            SourceKind::LocalFile => Some(SourceSummary::redacted(SourceKind::LocalFile)),
            SourceKind::UserDecision => Some(SourceSummary::redacted(SourceKind::UserDecision)),
        })
        .collect()
}
