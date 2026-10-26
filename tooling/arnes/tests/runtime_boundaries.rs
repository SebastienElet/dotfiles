#![cfg(test)]
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
const FORBIDDEN_RUNTIME_MODULES: &[&str] = &["memory", "handoff"];
#[test]
fn arnes_source_tree_has_no_memory_or_handoff_runtime_module() {
    let source = arnes_root().join("src");
    let forbidden = forbidden_runtime_modules(&source);
    assert!(forbidden.is_empty(), "{forbidden:?}");
}
#[test]
fn arnes_source_inventory_includes_core_canaries()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let files = rust_sources(&arnes_root().join("src"))?;
    assert!(
        files.iter().any(|path| path.ends_with("hooks.rs")),
        "{files:?}"
    );
    assert!(
        files.iter().any(|path| path.ends_with("main.rs")),
        "{files:?}"
    );
    Ok(())
}
#[test]
fn runtime_module_detection_rejects_directory_modules()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let root = TempDir::new()?;
    let source = root.path().join("src");
    for module in ["memory", "handoff"] {
        let directory = source.join(module);
        fs::create_dir_all(&directory)?;
        fs::write(directory.join("mod.rs"), b"")?;
    }
    let forbidden = forbidden_runtime_modules(&source);
    assert!(
        forbidden.iter().any(|path| path.ends_with("memory/mod.rs")),
        "{forbidden:?}"
    );
    assert!(
        forbidden
            .iter()
            .any(|path| path.ends_with("handoff/mod.rs")),
        "{forbidden:?}"
    );
    Ok(())
}
fn forbidden_runtime_modules(source: &Path) -> Vec<PathBuf> {
    let mut forbidden = Vec::new();
    for module in FORBIDDEN_RUNTIME_MODULES {
        for path in [
            source.join(format!("{module}.rs")),
            source.join(module).join("mod.rs"),
        ] {
            if path.exists() {
                forbidden.push(path);
            }
        }
    }
    forbidden
}
#[test]
fn arnes_source_tree_does_not_read_memory_state_or_name_memory_domain_types()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut forbidden = Vec::new();
    for file in rust_sources(&arnes_root().join("src"))? {
        let contents = fs::read_to_string(&file)?;
        for token in [
            "AGENT_MEMORY_ROOT",
            ".local/share/agent-memory",
            "MemoryEntry",
            "RetrievalReport",
        ] {
            if contents.contains(token) {
                forbidden.push(format!("{} contains {token}", file.display()));
            }
        }
    }
    assert!(forbidden.is_empty(), "{}", forbidden.join("\n"));
    Ok(())
}
fn arnes_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}
fn rust_sources(
    directory: &Path,
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error + Send + Sync>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            files.extend(rust_sources(&path)?);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            files.push(path);
        }
    }
    Ok(files)
}
