# Arnes MCP Doctor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Diagnose manifest-declared MCP registrations through both `arnes doctor mcp` and the aggregate `arnes doctor` command without executing a server or exposing secrets.

**Architecture:** Add an explicit WET manifest projection for each `(agent, scope, name)`, parse each agent's native MCP configuration into one internal observed-registration type, then compare fields and resolve local commands through filesystem inspection only. Keep parsing, comparison, command resolution, and Doctor orchestration in focused Rust modules.

**Tech Stack:** Rust 2024, Serde, `serde_json`, `toml`, Clap, Cargo integration tests

**Spec:** `docs/superpowers/specs/2026-09-01-arnes-mcp-doctor-design.md`

## Global Constraints

- Diagnosis is read-only: no agent CLI, server, wrapper, subprocess, container, image, network, or configuration mutation.
- A manifest entry contains only environment-variable names; observed secret values are never rendered.
- Command availability follows the actual resolved file: scope root for relative paths and ordered `PATH` lookup for bare names.
- ADR-031 wrappers are checked as executable files only; their Docker topology is never inspected.
- Unknown types, missing inputs, duplicate keys, and parser failures fail closed with an actionable diagnostic.
- Production files stay under the 250-line review trigger and production functions under 50 logical lines.
- No new dependency and no production comment.

---

### Task 1: Canonical MCP Manifest Model

**Files:**

- Create: `tooling/arnes/src/manifest/mcp.rs`
- Create: `tooling/arnes/src/manifest/validation/mcp.rs`
- Modify: `tooling/arnes/src/manifest.rs`
- Modify: `tooling/arnes/src/manifest/validation.rs`
- Test: `tooling/arnes/tests/manifest_mcp.rs`

**Interfaces:**

- Produces: `Manifest::mcp_registrations() -> impl Iterator<Item = McpRegistration<'_>>`
- Produces: `McpRegistration { name, agent, scope, command, args, environment, enabled }`

- [ ] **Step 1: Write failing manifest tests**

```rust
#[test]
fn parses_one_explicit_mcp_projection() {
    let manifest = manifest::parse(MANIFEST).unwrap();
    let registration = manifest.mcp_registrations().next().unwrap();
    assert_eq!(registration.name, "apple-notes");
    assert_eq!(registration.args, ["--stdio"]);
    assert_eq!(registration.environment, ["NOTES_PROFILE"]);
}

#[test]
fn rejects_duplicate_projection_and_environment_names() {
    assert_eq!(error(DUPLICATE), "mcp[1]: duplicates mcp[0] projection");
    assert_eq!(error(DUPLICATE_ENV), "mcp[0].environment[1]: duplicate environment reference");
}
```

- [ ] **Step 2: Verify RED**

Run: `cargo test --locked --test manifest_mcp`
Expected: compilation fails because `mcp_registrations` does not exist.

- [ ] **Step 3: Add the minimal schema and validation**

```rust
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct McpDeclaration {
    pub(super) name: String,
    pub(super) agent: Agent,
    pub(super) scope: Scope,
    pub(super) command: String,
    #[serde(default)] pub(super) args: Vec<String>,
    #[serde(default)] pub(super) environment: Vec<String>,
    pub(super) enabled: Option<bool>,
}
```

Validate non-empty portable names, non-empty commands, shell environment-name syntax, declared agent/scope targets, duplicate environment names, duplicate `(agent, scope, name)` projections, and reject `enabled` for Cursor.

- [ ] **Step 4: Verify GREEN and commit**

Run: `cargo test --locked --test manifest_mcp`
Expected: PASS.

```bash
git add tooling/arnes/src/manifest.rs tooling/arnes/src/manifest/mcp.rs tooling/arnes/src/manifest/validation.rs tooling/arnes/src/manifest/validation/mcp.rs tooling/arnes/tests/manifest_mcp.rs
git commit -m "feat(arnes): declare managed MCP registrations"
```

### Task 2: Fail-Closed Native Configuration Readers

**Files:**

- Create: `tooling/arnes/src/mcp/configuration.rs`
- Create: `tooling/arnes/src/mcp/json.rs`
- Create: `tooling/arnes/src/mcp/observed.rs`
- Create: `tooling/arnes/src/mcp.rs`
- Modify: `tooling/arnes/src/lib.rs`
- Test: `tooling/arnes/src/mcp/configuration/tests.rs`

**Interfaces:**

- Produces: `configuration::load(roots, agent, scope) -> Result<Option<ObservedConfiguration>, ConfigurationError>`
- Produces: `ObservedRegistration { command, args, environment, enabled }` with environment values reduced to reference names or redacted mismatch markers.

- [ ] **Step 1: Write failing reader tests**

```rust
#[test]
fn reads_claude_cursor_and_codex_native_entries() {
    for case in configured_cases() {
        let observed = case.load().unwrap().unwrap();
        assert_eq!(observed.registration("managed").unwrap().command, case.command);
    }
}

#[test]
fn duplicate_json_names_and_wrong_field_types_are_errors() {
    assert_error(DUPLICATE_JSON, "duplicate MCP name managed");
    assert_error(WRONG_ARGS, "managed.args must be an array of strings");
}
```

- [ ] **Step 2: Verify RED**

Run: `cargo test --locked mcp::configuration::tests`
Expected: compilation fails because `arnes::mcp` is absent.

- [ ] **Step 3: Implement native paths and duplicate-preserving parsing**

```rust
pub(super) struct ObservedRegistration {
    pub command: String,
    pub args: Vec<String>,
    pub environment: BTreeMap<String, EnvironmentValue>,
    pub enabled: Option<bool>,
}

pub(super) enum EnvironmentValue {
    Reference(String),
    RedactedLiteral,
}
```

Use a recursive Serde visitor in `json.rs` that rejects a repeated object key before building a
`serde_json::Value`; use `toml::Value` for TOML duplicate rejection. Extract only `mcpServers` or
`mcp_servers`, preserve ordered arguments, map Claude/Cursor `${NAME}` values and Codex `env_vars`
to references, and never retain literal environment values. For Claude, combine the selected
registration file with the current repository's `disabledMcpServers` state from `~/.claude.json`;
for Codex, read `enabled` with its documented default of `true`; leave Cursor state absent.

- [ ] **Step 4: Verify GREEN and commit**

Run: `cargo test --locked mcp::configuration::tests`
Expected: PASS for user/project JSON and TOML fixtures; malformed, duplicate, unreadable, and wrong-root cases return the asserted errors.

```bash
git add tooling/arnes/src/lib.rs tooling/arnes/src/mcp.rs tooling/arnes/src/mcp
git commit -m "feat(arnes): read native MCP configurations"
```

### Task 3: Registration Comparison and Local Command Oracle

**Files:**

- Create: `tooling/arnes/src/mcp/command.rs`
- Create: `tooling/arnes/src/mcp/comparison.rs`
- Test: `tooling/arnes/src/mcp/command/tests.rs`
- Test: `tooling/arnes/src/mcp/comparison/tests.rs`

**Interfaces:**

- Produces: `command::diagnose(roots, scope, command, path) -> Option<Diagnostic>`
- Produces: `comparison::diagnose(expected, observed) -> Vec<Diagnostic>`

- [ ] **Step 1: Write failing comparison and command tests**

```rust
#[test]
fn detects_each_contract_field_without_rendering_literals() {
    let diagnostics = diagnose_fixture(DIVERGENT);
    assert_messages(&diagnostics, ["command differs", "ordered arguments differ", "environment references differ", "enabled state differs"]);
    assert!(!render(&diagnostics).contains("actual-secret"));
}

#[test]
fn resolves_absolute_relative_and_path_commands_without_running_them() {
    let fixture = executable_cases();
    assert_eq!(fixture.doctor_code(), 0);
    assert!(!fixture.home().join("executed-sentinel").exists());
}
```

- [ ] **Step 2: Verify RED**

Run: `cargo test --locked mcp::`
Expected: compilation fails because comparison and command modules are absent.

- [ ] **Step 3: Implement exact comparisons and filesystem-only resolution**

```rust
fn candidate(command: &str, roots: &Roots, scope: Scope) -> Result<PathBuf, CommandError> {
    if command.contains(std::path::MAIN_SEPARATOR) {
        let path = PathBuf::from(command);
        return Ok(if path.is_absolute() { path } else { scope_root(roots, scope).join(path) });
    }
    resolve_in_path(command)
}
```

Use `metadata`, `is_file`, and Unix execute bits only. Missing commands are drift; unreadable,
non-file, non-executable, missing `PATH`, and non-UTF-8 paths are errors. Compare exact command,
ordered args, environment names/references, and represented enabled state.

- [ ] **Step 4: Verify GREEN and commit**

Run: `cargo test --locked mcp::`
Expected: PASS, including missing/non-executable cases and the execution sentinel.

```bash
git add tooling/arnes/src/mcp/command.rs tooling/arnes/src/mcp/command tooling/arnes/src/mcp/comparison.rs tooling/arnes/src/mcp/comparison
git commit -m "feat(arnes): compare MCP registrations"
```

### Task 4: `doctor mcp` Orchestration and Collision Diagnostics

**Files:**

- Modify: `tooling/arnes/src/mcp.rs`
- Modify: `tooling/arnes/src/doctor.rs`
- Test: `tooling/arnes/tests/mcp.rs`
- Test: `tooling/arnes/tests/mcp_failures.rs`
- Create: `tooling/arnes/tests/support/mcp.rs`

**Interfaces:**

- Produces: `mcp::diagnose(&Roots, &Manifest, Option<Agent>, Option<Scope>) -> Vec<Diagnostic>`
- Consumes: manifest projections, native readers, comparison, and command oracle from Tasks 1-3.

- [ ] **Step 1: Write failing CLI tests**

```rust
#[test]
fn matching_managed_registrations_are_healthy_for_all_agents() {
    for agent in ["claude", "cursor", "codex"] {
        let output = fixture(agent).command(["doctor", "mcp", "--agent", agent, "-v"]);
        assert_eq!(output.status.code(), Some(0));
        assert!(stdout(output).contains("healthy mcp:"));
    }
}

#[test]
fn explicit_undeclared_combinations_are_unsupported() {
    assert_contains(run(["doctor", "mcp", "--agent", "cursor", "--scope", "project"]), "unsupported mcp:");
}
```

- [ ] **Step 2: Verify RED**

Run: `cargo test --locked --test mcp --test mcp_failures`
Expected: tests fail because `Resource::Mcp` still dispatches no diagnostics.

- [ ] **Step 3: Implement bounded selection and collisions**

Load only combinations selected from manifest declarations. Return `unsupported` only for an
explicitly requested empty combination. Diagnose missing entries, same-agent same-name scope
collisions, and a managed name occupied by an undeclared projection; ignore every unrelated and
plugin-owned name. Prefix every result with resource `mcp` and preserve deterministic manifest order.

- [ ] **Step 4: Verify GREEN and commit**

Run: `cargo test --locked --test mcp --test mcp_failures`
Expected: PASS for healthy, missing, collision, unsupported, redaction, read-only snapshot, and no-execution cases.

```bash
git add tooling/arnes/src/doctor.rs tooling/arnes/src/mcp.rs tooling/arnes/tests/mcp.rs tooling/arnes/tests/mcp_failures.rs tooling/arnes/tests/support/mcp.rs
git commit -m "feat(arnes): diagnose MCP registrations"
```

### Task 5: Aggregate Doctor and Repository Declaration

**Files:**

- Modify: `tooling/arnes/src/cli.rs`
- Modify: `tooling/arnes/src/doctor.rs`
- Modify: `tooling/arnes/src/doctor/render.rs`
- Modify: `tooling/arnes/tests/cli.rs`
- Modify: `tooling/arnes/tests/mcp.rs`
- Modify: `home/.arnes.yaml`
- Modify: `.github/workflows/lint.yml`

**Interfaces:**

- Produces: aggregate `arnes doctor` sections `Manifest`, `Skills`, `Hooks`, `MCP`.
- Preserves: resource-specific commands default to user scope; unfiltered aggregate MCP scans every declared scope.

- [ ] **Step 1: Write the failing aggregate test**

```rust
#[test]
fn default_doctor_checks_project_mcp_without_changing_other_scope_defaults() {
    let output = configured_fixture().command(["doctor", "-v"]);
    let stdout = stdout(output);
    assert!(stdout.contains("MCP"));
    assert!(stdout.contains("healthy mcp: claude project apple-notes"));
    assert!(stdout.contains("Skills · user scope"));
}
```

- [ ] **Step 2: Verify RED**

Run: `cargo test --locked --test mcp default_doctor_checks_project_mcp_without_changing_other_scope_defaults`
Expected: FAIL because the default Doctor omits MCP.

- [ ] **Step 3: Add aggregate scope semantics and the managed Apple Notes declaration**

Remove Clap's implicit scope value so explicitness is observable. Apply user scope inside each
resource-specific Doctor and to existing aggregate resources; pass no scope to aggregate MCP unless
the user supplied one. Add `Resource::Mcp` to default rendering and declare the existing
`apple-notes` Claude project entry in `home/.arnes.yaml` with its absolute executable command.
Add this plan and its spec to the Prettier file list in `.github/workflows/lint.yml`.

```yaml
mcp:
  - name: apple-notes
    agent: claude
    scope: project
    command: /Applications/Apple Notes Exporter.app/Contents/SharedSupport/notes-export-mcp
```

- [ ] **Step 4: Run focused and full verification**

Run from `tooling/arnes`:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --locked
```

Run from the repository root:

```bash
prettier --check docs/superpowers/specs/2026-09-01-arnes-mcp-doctor-design.md docs/superpowers/plans/2026-09-01-arnes-mcp-doctor.md .github/workflows/lint.yml home/.arnes.yaml
moon run tooling/arnes:check --cache write --summary detailed
git diff --check
```

Expected: every command passes on the local macOS environment. Confirm every changed production
function is at most 50 logical lines and every changed hand-written production file at most 250.

- [ ] **Step 5: Commit**

```bash
git add tooling/arnes/src/cli.rs tooling/arnes/src/doctor.rs tooling/arnes/src/doctor/render.rs tooling/arnes/tests/cli.rs tooling/arnes/tests/mcp.rs home/.arnes.yaml .github/workflows/lint.yml docs/superpowers
git commit -m "feat(arnes): include MCP in default doctor"
```
