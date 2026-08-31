use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

const FORBIDDEN_RUNTIME_CRATES: &[&str] = &["agent-memory", "agent-handoff"];
const RUNTIME_DEPENDENCY_TABLES: &[&str] =
    &["dependencies", "dev-dependencies", "build-dependencies"];
const FORBIDDEN_RUNTIME_MODULES: &[&str] = &["memory", "handoff"];

#[test]
fn arnes_manifest_has_no_runtime_crate_dependency_on_memory_or_handoff() {
    let manifest: toml::Value =
        toml::from_str(&fs::read_to_string(arnes_root().join("Cargo.toml")).unwrap()).unwrap();
    let forbidden = forbidden_runtime_crate_dependencies(&manifest);
    assert!(forbidden.is_empty(), "{}", forbidden.join("\n"));
}

#[test]
fn runtime_crate_detection_rejects_package_aliases() {
    let manifest = parse_manifest(
        r#"
[package]
name = "arnes"
version = "0.1.0"
edition = "2024"

[dependencies]
memory_runtime = { package = "agent-memory", path = "../agent-memory" }
"#,
    );

    let forbidden = forbidden_runtime_crate_dependencies(&manifest);

    assert!(
        forbidden
            .iter()
            .any(|violation| violation.contains("agent-memory")),
        "{forbidden:?}"
    );
}

#[test]
fn runtime_crate_detection_rejects_target_specific_dependencies() {
    let manifest = parse_manifest(
        r#"
[package]
name = "arnes"
version = "0.1.0"
edition = "2024"

[target.'cfg(unix)'.dependencies]
handoff_runtime = { package = "agent-handoff", path = "../agent-handoff" }

[target.'cfg(windows)'.dev-dependencies]
memory_runtime = { package = "agent-memory", path = "../agent-memory" }
"#,
    );

    let forbidden = forbidden_runtime_crate_dependencies(&manifest);

    assert!(
        forbidden
            .iter()
            .any(|violation| violation.contains("agent-handoff")),
        "{forbidden:?}"
    );
    assert!(
        forbidden
            .iter()
            .any(|violation| violation.contains("agent-memory")),
        "{forbidden:?}"
    );
}

fn forbidden_runtime_crate_dependencies(manifest: &toml::Value) -> Vec<String> {
    let mut forbidden = Vec::new();
    collect_forbidden_runtime_crate_dependencies(manifest, "", &mut forbidden);
    forbidden
}

fn collect_forbidden_runtime_crate_dependencies(
    value: &toml::Value,
    path: &str,
    forbidden: &mut Vec<String>,
) {
    let Some(table) = value.as_table() else {
        return;
    };
    for (key, value) in table {
        let next_path = if path.is_empty() {
            key.clone()
        } else {
            format!("{path}.{key}")
        };
        if RUNTIME_DEPENDENCY_TABLES.contains(&key.as_str()) {
            collect_dependency_table(value, &next_path, forbidden);
        }
        collect_forbidden_runtime_crate_dependencies(value, &next_path, forbidden);
    }
}

fn collect_dependency_table(value: &toml::Value, path: &str, forbidden: &mut Vec<String>) {
    let Some(table) = value.as_table() else {
        forbidden.push(format!("{path} is not a dependency table"));
        return;
    };
    for (key, value) in table {
        let effective = dependency_package_name(key, value);
        if FORBIDDEN_RUNTIME_CRATES.contains(&effective.as_str()) {
            forbidden.push(format!("{path}.{key} resolves to {effective}"));
        }
    }
}

fn dependency_package_name(key: &str, value: &toml::Value) -> String {
    value
        .as_table()
        .and_then(|table| table.get("package"))
        .and_then(toml::Value::as_str)
        .unwrap_or(key)
        .to_owned()
}

#[test]
fn arnes_source_tree_has_no_memory_or_handoff_runtime_module() {
    let source = arnes_root().join("src");
    let forbidden = forbidden_runtime_modules(&source);
    assert!(forbidden.is_empty(), "{forbidden:?}");
}

#[test]
fn arnes_source_inventory_includes_core_canaries() {
    let files = rust_sources(&arnes_root().join("src"));
    assert!(
        files.iter().any(|path| path.ends_with("hooks.rs")),
        "{files:?}"
    );
    assert!(
        files.iter().any(|path| path.ends_with("main.rs")),
        "{files:?}"
    );
}

#[test]
fn runtime_module_detection_rejects_directory_modules() {
    let root = TempDir::new().unwrap();
    let source = root.path().join("src");
    for module in ["memory", "handoff"] {
        let directory = source.join(module);
        fs::create_dir_all(&directory).unwrap();
        fs::write(directory.join("mod.rs"), b"").unwrap();
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

fn parse_manifest(contents: &str) -> toml::Value {
    toml::from_str(contents).unwrap()
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
