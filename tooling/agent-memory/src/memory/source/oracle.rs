use super::{SourceContext, resolve_source};
use crate::memory::{EntrySource, SourceResolution, SourceResolver};

impl SourceResolver for SourceContext<'_> {
    fn resolve(&self, source: &EntrySource) -> SourceResolution {
        match resolve_source(source.kind(), source.locator(), self) {
            Ok(source) => SourceResolution::Fingerprint(source.fingerprint().as_str().to_owned()),
            Err(error) if error.code() == "source_invalid" => SourceResolution::Invalid,
            Err(_) => SourceResolution::Unavailable,
        }
    }

    fn work_cutoff_observed_expired(&self) -> bool {
        self.git.remaining_time().is_some_and(|time| time.is_zero())
            || self
                .curl
                .remaining_time()
                .is_some_and(|time| time.is_zero())
    }
}
