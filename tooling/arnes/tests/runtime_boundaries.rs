use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn arnes_manifest_has_no_runtime_crate_dependency_on_memory_or_handoff() {
    let manifest: toml::Value =
        toml::from_str(&fs::read_to_string(arnes_root().join("Cargo.toml")).unwrap()).unwrap();
    for table_name in ["dependencies", "dev-dependencies", "build-dependencies"] {
        let Some(table) = manifest.get(table_name).and_then(toml::Value::as_table) else {
            continue;
        };
        for crate_name in ["agent-memory", "agent-handoff"] {
            assert!(
                !table.contains_key(crate_name),
                "{table_name} contains {crate_name}"
            );
        }
    }
}

#[test]
fn arnes_source_tree_has_no_memory_or_handoff_runtime_module() {
    let source = arnes_root().join("src");
    for forbidden in ["memory", "handoff"] {
        assert!(!source.join(format!("{forbidden}.rs")).exists());
    }
}

#[test]
fn arnes_source_tree_does_not_read_memory_state_or_name_memory_domain_types() {
    let mut forbidden = Vec::new();
    for file in rust_sources(&arnes_root().join("src")) {
        let contents = fs::read_to_string(&file).unwrap();
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
}

fn arnes_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn rust_sources(directory: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in fs::read_dir(directory).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            files.extend(rust_sources(&path));
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            files.push(path);
        }
    }
    files
}
