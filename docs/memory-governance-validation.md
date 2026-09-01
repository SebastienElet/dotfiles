# Durable memory end-to-end validation

- Candidate SHA-256: `10b8ba0be1e1aa8af9b79c59a397f8d78a71937b41cce92b02447172c8f108ac`
- Runtime SHA-256: `4fd4bc0deeaf1198a1242afb88c95d61549482addf964029a4708e1427694a45`
- Date: 2026-09-01T07:08:19.089Z
- Environment: darwin arm64
- Timeout: 120 seconds per process, then TERM and bounded KILL

Agent | Version | Status | Replicates | Failure | Cleanup
--- | --- | --- | ---: | --- | ---
Codex | codex-cli 0.152.0 | complete | 3/3 | none | complete
Claude | unavailable | failed | 0/3 | authentication_unavailable | complete
Cursor | unavailable | failed | 0/3 | authentication_unavailable | complete

Capability | Codex | Claude | Cursor
--- | ---: | ---: | ---:
authorized_admission | 3/3 | 0/3 | 0/3
complete_proposal | 3/3 | 0/3 | 0/3
contradiction_invalidated | 3/3 | 0/3 | 0/3
durable_detection | 3/3 | 0/3 | 0/3
fresh_retrieval | 3/3 | 0/3 | 0/3
freshness_before_influence | 3/3 | 0/3 | 0/3
no_implicit_write | 3/3 | 0/3 | 0/3
proof_before_influence | 3/3 | 0/3 | 0/3
rejection_redacted | 3/3 | 0/3 | 0/3
sensitive_rejected | 3/3 | 0/3 | 0/3
store_unchanged | 3/3 | 0/3 | 0/3
stored | 3/3 | 0/3 | 0/3
unavailable_no_mutation | 3/3 | 0/3 | 0/3
unavailable_omitted | 3/3 | 0/3 | 0/3
unrelated_not_injected | 3/3 | 0/3 | 0/3

## Commands

- `bun tooling/agent-memory-eval.ts --agent codex --replicates 3`
- `bun tooling/agent-memory-eval.ts --agent claude --replicates 3`
- `bun tooling/agent-memory-eval.ts --agent cursor --replicates 3`

Cleanup is recorded only after fixture removal is verified.

Missing or blocked agents establish no capability. This macOS arm64 evidence does not establish Linux behavior.
