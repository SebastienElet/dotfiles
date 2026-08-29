# Task 3 report: verify freshness before retrieval

## Status

Task 3 is complete. The `agent-memory` domain now enforces the approved oracle, cache, retrieval,
and transition contracts. No Task 4+ CLI or runtime adapter was added or changed.

## Sources verified

- ADR-038 is accepted and places the independent local application under `tooling/`.
- ADR-041 is accepted and selects Rust for this durable-state application.
- ADR-042 is accepted and makes `agent-memory` the owner of cache, oracle, retrieval, and lifecycle
  transitions. It requires strict `<48 h` validity, immediate local-source invalidation,
  `invalid → invalidated`, five typed human business terminals, and no reactivation.
- The approved design, Task 3 brief, and execution ledger agree on ordered cache identity,
  redacted retrieval output, atomic transitions, and omission without stale context.

## RED evidence and ruling

The brief's initial Task 3 command passed the preserved WIP at 28/28. The carried validation
contradiction reproduced separately:

```text
cargo test --manifest-path tooling/agent-memory/Cargo.toml --test memory_validation enforces_transition_reason_boundaries -- --exact --nocapture
left: "invalid_transition_reason"
right: "invalid_field"
FAILED, 0 passed, 1 failed
```

The WIP had centralized both YAML and human-conclusion reason validation under the specific
`invalid_transition_reason` code, while the older parser test retained the provisional generic
code. The specific code is now the stable contract on both boundaries; empty and whitespace-only
reasons are rejected consistently.

The `ProofValid` type test then failed to compile because the WIP exposed only a boolean-like
answer path. That RED established the missing closed domain value before its implementation.

## Changes

- Added the closed, entry-bound `ProofValid` response. Invalid memory IDs fail with
  `invalid_memory_id`; an answer for another entry cannot validate the selected proof.
- Kept `HumanConclusion` closed to the five business terminals. `ProofAnswers` now accepts only
  typed `ProofValid` values, so proof validation cannot be used as a YAML lifecycle conclusion.
- Preserved automated contradiction precedence: a matching human proof response cannot override a
  changed source fingerprint. A valid fallback verdict is cacheable while the YAML remains active
  and transition-free.
- Closed cache checks for malformed timestamps and non-`valid` records, exact compact canonical
  oracle/proof digests, ordered source fingerprints, environment identity, and absence of cleartext
  locators.
- Extended retrieval checks for query/fragment-free official URL summaries, redacted local and
  user-decision locators, five-entry loading, stale-selection omission, and exact `NotApplied`
  effects.
- Added a concurrent transition test proving that two human terminal conclusions publish exactly
  one terminal state through the atomic store boundary.

## Verification

Environment exercised: macOS 26.6.2, Darwin 25.6.0 arm64, Rust/Cargo 1.98.0, Apple Git 2.50.1,
worktree `/Users/sebastien/.dotfiles/.worktrees/pr-249-memory-defects`. Portable Rust components are
also supported on Linux; Linux was not exercised in this task.

- `cargo fmt --manifest-path tooling/agent-memory/Cargo.toml --check`: pass.
- `cargo clippy --manifest-path tooling/agent-memory/Cargo.toml --all-targets -- -D warnings`:
  pass.
- `cargo test --manifest-path tooling/agent-memory/Cargo.toml`: 127/127 primary integration tests
  pass; unit and doc-test harnesses also pass.
- Task 3 targeted suites: cache 10/10, oracle 8/8, retrieval 13/13, validation 10/10.
- `git diff --check`: pass before staging.

## Size and comment review

All changed production files remain below 250 lines and every changed production function remains
below 50 logical lines. All new or materially extended Task 3 test files remain below 250 lines;
the proof-fallback and transition-atomicity matrices were separated by responsibility.
`memory_validation.rs` remains above the file trigger because it is the existing cohesive schema
boundary suite; this task changed only the transition-reason expectation, and splitting the full
validation matrix would be unrelated churn.

Comments added: none.

## Concerns

- Verification is local to macOS arm64. Linux evidence remains required from the later portable CI
  task before making a cross-platform completion claim.
