# Harness evaluations

`moon run harness:check` is the public deterministic, non-mutating harness gate, locally and in CI.
It composes `validate-evals`, `validate-evidence`, `test-eval-runner`, the existing `arnes:test`,
and the repository's existing TypeScript lint/typecheck/format-check and Prettier check tasks.
No dependency invokes a real agent or an LLM API. Tests write disposable fixtures; Moon and Cargo
may write their normal caches and build artifacts, but no deployed harness or historical report
is changed. Rust/Cargo, Make, Git, and the repository's pinned Moon/Bun toolchain are prerequisites.

The GitHub Actions workflow `test-harness.yml` runs exactly `moon run harness:check` for every PR
and push to main. It has no affected-file or path filter, so changes to the manifest, projections,
skills, runner, evidence, Moon, or CI cannot escape through a stale filter. Task-level `runInCI`
overrides enable only the new deterministic tasks; operational harness tasks retain their settings.
The new tasks retain the project's disabled cache and declare evaluation inputs. Arnes retains its
own inputs, mutex, and cache policy.

## Data and ownership

- `cases.json`: three behavior contracts with stable IDs, source sections, prompt or trigger-query
  reference, fixture, versioned oracle, and explicit success/failure conditions.
- `fixtures/code-search-v1.json`: a tiny synthetic monorepo, with no dependency installation.
- `variants/no-op.md`: an explicit neutral replacement for the evaluated instruction section.
- `evidence/`: optional retained reports, never a prerequisite for a green check.
- [tooling/harness-eval/](../../tooling/harness-eval/): executable logic lives there under
  ADR-038/041. Zod schemas in `contracts.ts` and `report-schema.ts` are the
  executable contracts; TypeScript types derive from them. There is no duplicated JSON Schema.

`skill-manager/references/evals.md` owns activation-scenario semantics. `validate-evals` implements
that existing contract for both tracked skill collections and verifies the new case references.
The structural case refers to an existing query without copying its prompt. The literal case
permits skill activation, consistent with the existing scenario; it rejects conceptual search.
Arnes owns deployment validation; its existing tests exercise synthetic projections and the real
manifest/Makefile in temporary homes. They do not attest every current installation on your machine.
There is no existing automated full skill-manager doctor/resource-quality oracle to compose; this
gate does not claim to replace that procedural audit or optional `skills-ref` validation.

## Manual live evaluation

This operation spends quota. It is never a dependency of `check` and is never invoked by CI.
Use an installed Codex CLI supporting `exec --json --ephemeral --ignore-user-config --ignore-rules`,
with saved `auth.json` or `CODEX_API_KEY`, and supply the exact model ID you intend to measure:

```bash
moon run harness:eval -- --model YOUR_EXACT_MODEL_ID --only code-search-structural --runs 1 --report harness/evals/evidence/candidate.json
```

`--only` is mandatory and accepts comma-separated IDs. `--runs` is 1–10 (default 1);
`--timeout-seconds` is 1–600 (default 120); `--reasoning-effort` is low/medium/high (default low).
There is no token ceiling. Select few cases/runs first. A non-PASS result gives a nonzero exit
after publishing the new report. Existing report paths are refused before any live execution,
and publication uses an exclusive hard link to prevent races from overwriting history.

Each replicate receives a fresh temporary HOME, Codex home, and synthetic workspace. Only the
`Context Management` section from `harness/AGENTS.md` and the canonical `code-search` skill are
installed; USER/SOUL and other deployed instructions, plugins, hooks, and MCP configuration are not
part of this first experiment. Saved auth is copied temporarily when needed, never into evidence.
The runner passes the exact declared UTF-8 prompt on stdin, disables user configuration and rules,
uses workspace-write with agent-command network disabled, and requests no approvals.

PATH shims record successful reads and search invocations. For a structural PASS, the skill read
must precede conceptual search; for a literal PASS, exact `rg` must occur without conceptual search;
for a known-path PASS, the target must be read without exploration. Other ways of reading a file
can yield false negatives. The shims simulate external tools, not an agent, and are not protected
against a deliberately tampering agent. They prove neither ColGrep retrieval quality nor internal
skill activation. No final-answer self-report is used by the oracle.

## Evidence and comparison

Reports record version, case snapshots, prompt bytes/fingerprints, source fingerprints, agent and
version, requested model, Git revision and tested instruction/skill fingerprints, fixture/runner
fingerprints, controls, environment, date, replicate count, PASS/FAIL/INVALID results, observations,
tokens/tool calls/duration when available, and limitations. Missing measurements remain null.
Timeout, nonzero exit, output overflow, broken events, or unreadable observation logs become
INVALID, never PASS. Raw transcripts and arbitrary tool arguments are not retained.

Historical validation checks the stored snapshot and recomputes its versioned oracle, not the
current harness bytes: changing a prompt or instruction does not rewrite yesterday's evidence.
The writer enforces no overwrite; it does not make Git history immutable or certify provenance.
Review retained reports before committing. Never retain secrets or private material.

Produce baseline and candidate explicitly with the same model, cases, runs, budget, and environment:

```bash
moon run harness:eval -- --model YOUR_EXACT_MODEL_ID --only code-search-structural,code-search-literal,code-search-known-path --runs 3 --variant-file harness/evals/variants/no-op.md --report harness/evals/evidence/baseline.json
moon run harness:eval -- --model YOUR_EXACT_MODEL_ID --only code-search-structural,code-search-literal,code-search-known-path --runs 3 --report harness/evals/evidence/candidate.json
moon run harness:compare -- harness/evals/evidence/baseline.json harness/evals/evidence/candidate.json
```

`compare` refuses mismatched recorded controls, case contracts, prompt bytes/fingerprints, fixtures,
runner, agent/version, model, environment, or replicate counts. It reports success rates, invalid
counts, failures, per-replicate regressions, and mean token/tool/duration metrics. INVALID stays in
the denominator. Results are descriptive: matching controls and a positive delta do not establish
statistical or causal uplift. Codex exposes neither paired random seeds nor attestation of a model
alias's resolved version here. Two identical reports also compare successfully as a no-op check.

## Deterministic testing and limits

`moon run harness:test-eval-runner` includes fixture creation, actual shim execution, observation
collection, scoring, report construction/validation, publication refusal, comparator controls,
process failures/timeouts, and CLI isolation tests with sentinel executables. The smoke executor
is a fixed command sequence, not simulated live evidence. Its report has `agent: fixture-smoke`
and `model: none`. Tests validate that report in memory and never store it as a historical live
result.

Do not substitute `moon check harness`: Moon currently infers `test` for `semctx`, which installs
plugins, and for other operational tasks lacking outputs/persistence. `eval` also remains a manual
one-shot task with mandatory arguments. Auditing/reclassifying those tasks is a separate change.

This v1 does not include Claude/Cursor adapters, full-harness or composed behavior evaluation,
automatic ablation, automatic live runs, LLM judges, a database, or a multi-agent scheduler.

## Discovery sources

- Accepted ADR-038, ADR-039, and ADR-041 govern placement, retrieval, and implementation language.
- [Moon task options](https://moonrepo.dev/docs/config/project),
  [task types](https://moonrepo.dev/docs/concepts/task), and
  [native check](https://moonrepo.dev/docs/commands/check), consulted 2026-09-05 for pinned Moon 2.5.3.
- [Codex non-interactive execution](https://developers.openai.com/codex/noninteractive/), consulted
  2026-09-05 and checked against installed CLI help; no live execution was performed.
- [Reference repository](https://github.com/Syo-M/codex-frontend-skills/tree/0c25f8e1c616da6242ca09a7f3613412521cef69),
  inspected at that revision: single deterministic entry point, separate quota-consuming evaluation,
  constrained evidence, and controlled comparison. Its frontend and Codex distribution contracts
  were not copied.
