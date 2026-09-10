#![cfg(test)]

use agent_memory::{MemoryRoot, parse_draft};

#[test]
fn domain_is_exported_by_agent_memory() {
    let _: fn(&[u8]) -> Result<agent_memory::AdmissionDraft, agent_memory::MemoryError> =
        parse_draft;
    let _: fn(std::path::PathBuf) -> Result<MemoryRoot, agent_memory::MemoryError> =
        MemoryRoot::new;
}
