# Rust checks

`agent-handoff`, `agent-memory`, and `arnes` remain independent packages under
`tooling/`, with their existing lockfiles and deployment directories.

## Policy

All three manifests use the following native policy, without disabled group
members or local `allow`/`expect` suppressions:

```toml
[lints.rust]
unsafe_code = "forbid"
unused_must_use = "forbid"

[lints.clippy]
all = { level = "deny", priority = -2 }
pedantic = { level = "deny", priority = -2 }
nursery = { level = "deny", priority = -2 }
cargo = { level = "deny", priority = -2 }
correctness = { level = "deny", priority = -1 }
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
todo = "deny"
unimplemented = "deny"
unreachable = "deny"
dbg_macro = "deny"
print_stdout = "deny"
print_stderr = "deny"
indexing_slicing = "deny"
lossy_float_literal = "deny"
mem_forget = "deny"
exit = "deny"
allow_attributes = "deny"
allow_attributes_without_reason = "deny"
multiple_unsafe_ops_per_block = "deny"
undocumented_unsafe_blocks = "deny"
integer_division = "deny"
wildcard_enum_match_arm = "deny"
```

Clippy groups and restrictions use `deny`; CI also passes `-D warnings`.
`forbid` cannot be used for these Clippy rules with the current Clap and Serde
derives: their generated code contains lint attributes that conflict with it.
`unsafe_code` and `unused_must_use` remain forbidden by rustc.

The restriction group is not enabled wholesale. Each retained restriction is
listed explicitly. `clippy.toml` disables Clippy's unwrap/expect exceptions for
both tests and const contexts, and its test panic/indexing exceptions. The same
rules apply to owned production code and tests. Test setup returns `Result`,
required fixture data uses checked access, and assertions still check outcomes.

Public fallible APIs document their error conditions; pure APIs use `must_use`
and const functions where Clippy requires them. Package metadata references the
repository's existing license rather than disabling Cargo metadata checks.

No native lint configuration proves that all execution is panic-free, or
prevents a future change to the configuration itself. Dependency implementation
and macro expansion remain upstream code; dependency findings stay visible as
failures rather than being suppressed in the package.

Agent Memory uses `time` for UTC timestamps and cache ages. Its lockfile selects
`idna_adapter` 1.1.0, the [official unicode-rs backend](https://docs.rs/crate/idna_adapter/latest),
while retaining URL and IDNA processing. This avoids ICU's mixed macro dependencies
without downgrading crates past their soundness fixes. Compared with ICU, the
backend trades faster compilation for a larger binary and slower IDNA processing;
its Unicode data can differ. Cargo updates must preserve a duplicate-free graph
and pass the URL-policy and timestamp/cache tests with the strict Clippy checks.

## Commands

Run in each package directory:

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
RUSTDOCFLAGS=-Dwarnings cargo doc --workspace --all-features --no-deps --locked
```

`cargo test` includes doctests; `cargo test --doc --all-features --locked` runs
those alone. `cargo fmt --all` applies the default edition-2024 formatting.

Agent Memory compiles its scenario modules in one `integration` target, sharing
fixtures once. Use `cargo test --test integration memory_cli:: --locked` to select
one family; the complete commands above retain all scenarios and unit tests.

Arnes's deployment integration test also requires Bun, the root lockfile's
installed dependencies, and Moon on `PATH` or at `DEPLOYMENT_MOON`. The normal
Moon test task depends on `repository:dependencies`; prepare that task before
running Cargo directly.

Moon exposes `<package>:fmt`, `:check`, `:clippy`, `:test`, and `:doc`. In a
worktree whose runtimes and package dependencies are already installed, use
`moon exec --upstream none --no-actions` to run these checks without workstation
installation dependencies. Root Clippy/rustfmt configuration changes are inputs
to all Rust verification tasks and their affected CI selection.

Agent Memory and Agent Handoff CI target macOS and Ubuntu; Arnes CI targets
Ubuntu. Local evidence applies only to the platform exercised. Remote CI must
pass before merge.

References: [Cargo lint configuration](https://doc.rust-lang.org/cargo/reference/manifest.html#the-lints-section),
[rustc lint levels](https://doc.rust-lang.org/rustc/lints/levels.html),
and [Clippy configuration](https://doc.rust-lang.org/clippy/lint_configuration.html).
