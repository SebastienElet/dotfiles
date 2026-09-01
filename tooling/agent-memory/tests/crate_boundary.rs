use agent_memory::{MemoryRoot, parse_draft};

#[test]
fn domain_is_exported_by_agent_memory() {
    let _parse = parse_draft;
    let _root: fn(std::path::PathBuf) -> Result<MemoryRoot, agent_memory::MemoryError> =
        MemoryRoot::new;
}
