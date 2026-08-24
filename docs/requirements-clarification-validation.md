# `requirements-clarification` validation

- **Date:** 2026-08-24
- **Exercised system:** macOS, arm64
- **Claude Code:** 2.1.241
- **Codex CLI:** 0.149.1
- **Cursor Agent:** 2026.08.11-e8db854
- **Not exercised:** Linux

## Method

Canonical prompts live in
`harness/skills/requirements-clarification/evals/trigger-queries.json`. Every invocation used a new
process, the same prompt for comparable runs, and a non-mutating mode. Agents ran against a clean
temporary repository snapshot containing the current implementation. The scenario answers and this
report were unavailable there; neutral placeholders preserved the lint contract without exposing
expected activation or verdicts. Raw JSONL remains a local artifact, while the normalized results
below are the retained execution evidence.

Claude activation requires a `Skill` event for `requirements-clarification`. Codex activation
requires a read through `~/.agents/skills/requirements-clarification/SKILL.md`. Cursor activation
requires a read through a discovered user-skill path. Cursor chose its documented Claude-compatible
path, `~/.claude/skills/requirements-clarification/SKILL.md`; its native `~/.cursor/skills` link was
verified separately. Cursor officially discovers both native user skills and Claude/Codex
compatibility directories: <https://cursor.com/docs/skills>.

Routing and behavior are judged separately. Positive questions pass only when plausible answers
change behavior, architecture, data, security, or acceptance criteria. Negative cases must research
available facts and ask no requirements question. Execution invitations are recorded but do not
count as requirements questions.

## Always-loaded reminder ablation

With the final description alone, Claude activated twice out of three for authentication and three
times out of three for migration. A reminder injected directly into the system prompt reached three
out of three in both cases, but the same reminder loaded through the real `harness/AGENTS.md` path
left authentication at two out of three. A narrower opening sentence also left Cursor's negative
activation unchanged. Both changes were reverted under ADR-035. No hook or always-loaded rule is
retained. On Claude's negative cases, the real-path reminder reduced behavior passes from one to zero
for S3 and from two to one for S4. A host hook cannot judge semantic question quality, and the
measured real instruction path made behavior worse.

## Results

Researched facts are abbreviated as follows:

- `F1`: Arnes's local CLI shape, absent network boundary, and manifest secret rejection;
- `F2`: `SCHEMA_VERSION`, parser, validation, and v1 manifest tests;
- `F3`: manifest, Makefile targets, deployment test, lint, and neutral validation artifacts;
- `F4`: declaration and two local uses of `destinations`;
- `F5`: neighboring `30_000` timeout and the new test's structure;
- `F6`: current import groups, neighboring tests, and absence of an automated sort rule.

Questions are `Q0` (none), `Q1` (authentication actor, resource, threat, and failure), `Q2` (v2
delta and compatibility), `QE` (execution invitation only), `QS` (non-material style choice), or
`QD` (discoverable scope asked back to the user). `M0` means no mutation. Route expectations come
from `should_activate`; behavior verdicts enforce the requirements contract independently.

| Agent/version                   | Scenario | Run | Activation | Facts | Questions | Mutation | Route | Behavior |
| ------------------------------- | -------- | --: | ---------- | ----- | --------- | -------- | ----- | -------- |
| Claude Code 2.1.241             | S1       |   1 | yes        | F1    | Q1        | M0       | PASS  | PASS     |
| Claude Code 2.1.241             | S1       |   2 | yes        | F1    | Q1        | M0       | PASS  | PASS     |
| Claude Code 2.1.241             | S1       |   3 | no         | F1    | Q1        | M0       | FAIL  | PASS     |
| Claude Code 2.1.241             | S2       |   1 | yes        | F2    | Q2        | M0       | PASS  | PASS     |
| Claude Code 2.1.241             | S2       |   2 | yes        | F2    | Q2        | M0       | PASS  | PASS     |
| Claude Code 2.1.241             | S2       |   3 | yes        | F2    | Q2        | M0       | PASS  | PASS     |
| Claude Code 2.1.241             | S3       |   1 | no         | F3    | QE        | M0       | PASS  | PASS     |
| Claude Code 2.1.241             | S3       |   2 | no         | F3    | QD        | M0       | PASS  | FAIL     |
| Claude Code 2.1.241             | S3       |   3 | no         | F3    | QD        | M0       | PASS  | FAIL     |
| Claude Code 2.1.241             | S4       |   1 | no         | F4    | QE        | M0       | PASS  | PASS     |
| Claude Code 2.1.241             | S4       |   2 | no         | F4    | QS        | M0       | PASS  | FAIL     |
| Claude Code 2.1.241             | S4       |   3 | no         | F4    | QE        | M0       | PASS  | PASS     |
| Claude Code 2.1.241             | S5       |   1 | no         | F5    | Q0        | M0       | PASS  | PASS     |
| Claude Code 2.1.241             | S5       |   2 | no         | F5    | QE        | M0       | PASS  | PASS     |
| Claude Code 2.1.241             | S5       |   3 | no         | F5    | QE        | M0       | PASS  | PASS     |
| Claude Code 2.1.241             | S6       |   1 | no         | F6    | QE        | M0       | PASS  | PASS     |
| Claude Code 2.1.241             | S6       |   2 | no         | F6    | QE        | M0       | PASS  | PASS     |
| Claude Code 2.1.241             | S6       |   3 | no         | F6    | QE        | M0       | PASS  | PASS     |
| Codex CLI 0.149.1               | S1       |   1 | yes        | F1    | Q1        | M0       | PASS  | PASS     |
| Codex CLI 0.149.1               | S1       |   2 | yes        | F1    | Q1        | M0       | PASS  | PASS     |
| Codex CLI 0.149.1               | S1       |   3 | yes        | F1    | Q1        | M0       | PASS  | PASS     |
| Codex CLI 0.149.1               | S2       |   1 | yes        | F2    | Q2        | M0       | PASS  | PASS     |
| Codex CLI 0.149.1               | S2       |   2 | yes        | F2    | Q2        | M0       | PASS  | PASS     |
| Codex CLI 0.149.1               | S2       |   3 | yes        | F2    | Q2        | M0       | PASS  | PASS     |
| Codex CLI 0.149.1               | S3       |   1 | no         | F3    | Q0        | M0       | PASS  | PASS     |
| Codex CLI 0.149.1               | S3       |   2 | no         | F3    | Q0        | M0       | PASS  | PASS     |
| Codex CLI 0.149.1               | S3       |   3 | no         | F3    | Q0        | M0       | PASS  | PASS     |
| Codex CLI 0.149.1               | S4       |   1 | no         | F4    | Q0        | M0       | PASS  | PASS     |
| Codex CLI 0.149.1               | S4       |   2 | no         | F4    | Q0        | M0       | PASS  | PASS     |
| Codex CLI 0.149.1               | S4       |   3 | no         | F4    | Q0        | M0       | PASS  | PASS     |
| Codex CLI 0.149.1               | S5       |   1 | no         | F5    | Q0        | M0       | PASS  | PASS     |
| Codex CLI 0.149.1               | S5       |   2 | no         | F5    | Q0        | M0       | PASS  | PASS     |
| Codex CLI 0.149.1               | S5       |   3 | no         | F5    | Q0        | M0       | PASS  | PASS     |
| Codex CLI 0.149.1               | S6       |   1 | no         | F6    | Q0        | M0       | PASS  | PASS     |
| Codex CLI 0.149.1               | S6       |   2 | no         | F6    | Q0        | M0       | PASS  | PASS     |
| Codex CLI 0.149.1               | S6       |   3 | no         | F6    | Q0        | M0       | PASS  | PASS     |
| Cursor Agent 2026.08.11-e8db854 | S1       |   1 | yes        | F1    | Q1        | M0       | PASS  | PASS     |
| Cursor Agent 2026.08.11-e8db854 | S1       |   2 | yes        | F1    | Q1        | M0       | PASS  | PASS     |
| Cursor Agent 2026.08.11-e8db854 | S1       |   3 | yes        | F1    | Q1        | M0       | PASS  | PASS     |
| Cursor Agent 2026.08.11-e8db854 | S2       |   1 | yes        | F2    | Q2        | M0       | PASS  | PASS     |
| Cursor Agent 2026.08.11-e8db854 | S2       |   2 | yes        | F2    | Q2        | M0       | PASS  | PASS     |
| Cursor Agent 2026.08.11-e8db854 | S2       |   3 | yes        | F2    | Q2        | M0       | PASS  | PASS     |
| Cursor Agent 2026.08.11-e8db854 | S3       |   1 | yes        | F3    | Q0        | M0       | FAIL  | PASS     |
| Cursor Agent 2026.08.11-e8db854 | S3       |   2 | yes        | F3    | Q0        | M0       | FAIL  | PASS     |
| Cursor Agent 2026.08.11-e8db854 | S3       |   3 | yes        | F3    | Q0        | M0       | FAIL  | PASS     |
| Cursor Agent 2026.08.11-e8db854 | S4       |   1 | yes        | F4    | Q0        | M0       | FAIL  | PASS     |
| Cursor Agent 2026.08.11-e8db854 | S4       |   2 | yes        | F4    | Q0        | M0       | FAIL  | PASS     |
| Cursor Agent 2026.08.11-e8db854 | S4       |   3 | yes        | F4    | Q0        | M0       | FAIL  | PASS     |
| Cursor Agent 2026.08.11-e8db854 | S5       |   1 | yes        | F5    | Q0        | M0       | FAIL  | PASS     |
| Cursor Agent 2026.08.11-e8db854 | S5       |   2 | yes        | F5    | Q0        | M0       | FAIL  | PASS     |
| Cursor Agent 2026.08.11-e8db854 | S5       |   3 | yes        | F5    | Q0        | M0       | FAIL  | PASS     |
| Cursor Agent 2026.08.11-e8db854 | S6       |   1 | yes        | F6    | Q0        | M0       | FAIL  | PASS     |
| Cursor Agent 2026.08.11-e8db854 | S6       |   2 | yes        | F6    | Q0        | M0       | FAIL  | PASS     |
| Cursor Agent 2026.08.11-e8db854 | S6       |   3 | yes        | F6    | Q0        | M0       | FAIL  | PASS     |

Codex passed routing and behavior in all 18 runs. Claude passed routing in 17/18 and behavior in
15/18. Cursor passed behavior in 18/18 but over-activated on all 12 negative runs. Description and
prompt ablations did not reduce that Cursor behavior; no host-specific suppression was added because
it would violate portability and the issue's host-neutral frontmatter constraint.
