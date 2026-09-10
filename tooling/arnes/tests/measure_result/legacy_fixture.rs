use crate::measure_support::Harness;

impl Harness {
    pub(super) fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Self::with_legacy_runs(true)
    }
}
