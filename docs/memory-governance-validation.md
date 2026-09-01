# Durable memory end-to-end validation

- Candidate SHA-256: `b33c2ffa84d371915b6275a4a2953b2391c32172319735b473b1203c4091770a`
- Runtime SHA-256: `98bf23a5ea62e3a0c308c1ddf0be57c429a9b53a12a396cd804e4f58724dddb6`
- Date: 2026-09-01T08:29:48.927Z
- Environment: darwin arm64
- Timeout: 120 seconds per process, then TERM and bounded KILL

Agent | Version | Status | Replicates | Failure | Cleanup
--- | --- | --- | ---: | --- | ---
Codex | codex-cli 0.152.0 | complete | 3/3 | none | complete
Claude | unavailable | failed | 0/3 | authentication_unavailable | complete
Cursor | unavailable | failed | 0/3 | evaluation_failure | complete

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
