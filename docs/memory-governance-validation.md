# Durable memory end-to-end validation

- Candidate SHA-256: `047d036d36b160d5194e9edb09ae1eafc3665a590ba6b243c924e016116c30c4`
- Runtime SHA-256: `4fd4bc0deeaf1198a1242afb88c95d61549482addf964029a4708e1427694a45`
- Date: 2026-09-01T05:58:08.868Z
- Environment: darwin arm64
- Timeout: 120 seconds per process, then TERM and bounded KILL

Agent | Version | Status | Replicates | Failure | Cleanup
--- | --- | --- | ---: | --- | ---
Codex | unavailable | missing | 0/0 | not_run | not_run
Claude | 2.1.252 | complete | 1/1 | none | complete
Cursor | unavailable | missing | 0/0 | not_run | not_run

Capability | Codex | Claude | Cursor
--- | ---: | ---: | ---:
authorized_admission | not_run | 1/1 | not_run
complete_proposal | not_run | 1/1 | not_run
contradiction_invalidated | not_run | 1/1 | not_run
durable_detection | not_run | 1/1 | not_run
fresh_retrieval | not_run | 1/1 | not_run
freshness_before_influence | not_run | 1/1 | not_run
no_implicit_write | not_run | 1/1 | not_run
proof_before_influence | not_run | 1/1 | not_run
rejection_redacted | not_run | 1/1 | not_run
sensitive_rejected | not_run | 1/1 | not_run
store_unchanged | not_run | 1/1 | not_run
stored | not_run | 1/1 | not_run
unavailable_no_mutation | not_run | 1/1 | not_run
unavailable_omitted | not_run | 1/1 | not_run
unrelated_not_injected | not_run | 1/1 | not_run

## Commands

- `bun tooling/agent-memory-eval.ts --agent codex --replicates 3`
- `bun tooling/agent-memory-eval.ts --agent claude --replicates 1`
- `bun tooling/agent-memory-eval.ts --agent cursor --replicates 3`

Cleanup is recorded only after fixture removal is verified.

Missing or blocked agents establish no capability. This macOS arm64 evidence does not establish Linux behavior.
