# Durable memory end-to-end validation

- Candidate SHA-256: `3549c726633e8e4010ba0d3d0c7587023e552553a556c2b824280064aada5e23`
- Runtime SHA-256: `4d3b4ab39e02ec50a1a4dcb7e2902305ff75862d80b3ec0d0854fc8ad48ffe73`
- Date: 2026-09-01T09:00:46.462Z
- Environment: darwin arm64
- Timeout: 120 seconds per process, then TERM and bounded KILL

| Agent  | Version           | Status   | Replicates | Failure                    | Cleanup  |
| ------ | ----------------- | -------- | ---------: | -------------------------- | -------- |
| Codex  | codex-cli 0.152.0 | complete |        3/3 | none                       | complete |
| Claude | unavailable       | failed   |        0/3 | authentication_unavailable | complete |
| Cursor | unavailable       | failed   |        0/3 | evaluation_failure         | complete |

| Capability                 | Codex | Claude | Cursor |
| -------------------------- | ----: | -----: | -----: |
| authorized_admission       |   3/3 |    0/3 |    0/3 |
| complete_proposal          |   3/3 |    0/3 |    0/3 |
| contradiction_invalidated  |   3/3 |    0/3 |    0/3 |
| durable_detection          |   3/3 |    0/3 |    0/3 |
| fresh_retrieval            |   3/3 |    0/3 |    0/3 |
| freshness_before_influence |   3/3 |    0/3 |    0/3 |
| no_implicit_write          |   3/3 |    0/3 |    0/3 |
| proof_before_influence     |   3/3 |    0/3 |    0/3 |
| rejection_redacted         |   3/3 |    0/3 |    0/3 |
| sensitive_rejected         |   3/3 |    0/3 |    0/3 |
| store_unchanged            |   3/3 |    0/3 |    0/3 |
| stored                     |   3/3 |    0/3 |    0/3 |
| unavailable_no_mutation    |   3/3 |    0/3 |    0/3 |
| unavailable_omitted        |   3/3 |    0/3 |    0/3 |
| unrelated_not_injected     |   3/3 |    0/3 |    0/3 |

## Commands

- `bun tooling/agent-memory-eval.ts --agent codex --replicates 3`
- `bun tooling/agent-memory-eval.ts --agent claude --replicates 3`
- `bun tooling/agent-memory-eval.ts --agent cursor --replicates 3`

Cleanup is recorded only after fixture removal is verified.

Missing or blocked agents establish no capability. This macOS arm64 evidence does not establish Linux behavior.
