# Harness evaluations

`moon run harness:check` is the public deterministic, non-mutating harness gate, locally and in CI.
It composes `validate-evals`, `validate-evidence`, `test-eval-runner` (the Arnes test suite),
Arnes format/lint/type checks, and the repository's existing TypeScript and Prettier checks.
No dependency invokes a real agent or an LLM API. Tests write disposable fixtures; Moon and Cargo
may write their normal caches and build artifacts, but no deployed harness or historical report
is changed. Rust/Cargo, Git, and the repository's Moon/Bun toolchain are prerequisites for the aggregate check;
the evaluation engine itself is the Arnes binary and does not require Bun.

The GitHub Actions workflow `test-harness.yml` selects `harness:validate-evals` and
`harness:validate-evidence` with `moon ci --downstream none` for PRs and pushes to main.
Arnes, TypeScript and text checks run in their dedicated workflows; CI does not invoke the local
aggregate `harness:check`. Evaluation tasks retain the project's disabled cache and declare
their inputs. Arnes retains its own inputs, mutex, and cache policy.

## Data and ownership

- `cases.json`: two behavior contracts with stable IDs, source sections, prompt or trigger-query
  reference, fixture, versioned oracle, and explicit success/failure conditions.
- `fixtures/repository-lookup-v1.json`: a tiny synthetic monorepo, with no dependency installation.
- `variants/no-op.md`: an explicit neutral replacement for the evaluated instruction section.
- `evidence/`: optional retained reports, never a prerequisite for a green check.
- [tooling/arnes/src/eval/](../../tooling/arnes/src/eval/): Arnes owns the execution engine,
  validation, instrumentation, reports, and comparison under ADR-038/041. Serde models and their
  validation implement the contracts; there is no duplicated JSON Schema or Bun evaluation runner.

`skill-manager/references/evals.md` owns activation-scenario semantics. `validate-evals` implements
that existing contract for both tracked skill collections and verifies the new case references.
No particular skill activation contract is required. The current cases cover exact literal lookup
and reading a known path, using native command names.
Arnes owns deployment validation; its existing tests exercise synthetic projections and the real
manifest/Moon deployments in temporary homes. They do not attest every current installation on your machine.
There is no existing automated full skill-manager doctor/resource-quality oracle to compose; this
gate does not claim to replace that procedural audit or optional `skills-ref` validation.

## Manual live evaluation

This operation spends quota. It is never a dependency of `check` and is never invoked by CI.
From the repository root, use `arnes eval run` (or the Moon alias below) with an installed
Codex CLI supporting `exec --json --ephemeral --ignore-user-config --ignore-rules`,
with saved `auth.json` or `CODEX_API_KEY`, and supply the exact model ID you intend to measure:

```bash
moon run harness:eval -- --model YOUR_EXACT_MODEL_ID --only repository-literal --runs 1 --report harness/evals/evidence/candidate.json
```

`arnes eval` exposes `validate-evals`, `validate-evidence [reports]`, `fixture-smoke`,
`preflight`, `run`, and `compare <baseline> <candidate>`. The optional `--repository` selects the source checkout
(default: current directory); report paths remain relative to the calling directory.
Moon aliases compile and run this checkout's Arnes with Cargo and a shared target-directory mutex.

Before spending quota, check startup without a model request:

```bash
cargo run --quiet --locked --manifest-path tooling/arnes/Cargo.toml -- eval preflight
```

This command uses saved authentication and the same isolated environment builder as live runs,
executes only `codex --version`, and fails within five seconds of process startup on a timeout.
It preserves `VOLTA_HOME` (or its default under the original HOME) while keeping HOME and CODEX_HOME
isolated. Every live replicate repeats this check in its own fixture and refuses an empty or changed
version before sending the prompt. This checks CLI startup, not authentication validity, model access,
or provider connectivity; `harness:check` remains independent of an installed Codex CLI.

`--only` is mandatory and accepts comma-separated IDs. `--runs` is 1–10 (default 1);
`--timeout-seconds` is 1–600 (default 120); `--reasoning-effort` is low/medium/high (default low).
There is no token ceiling. Select few cases/runs first. A non-PASS result gives a nonzero exit
after publishing the new report. Existing report paths are refused before any live execution,
and publication uses an exclusive hard link to prevent races from overwriting history.

Each replicate receives a fresh temporary HOME, Codex home, and synthetic workspace. Only the
`Context Management` section from `harness/AGENTS.md` is installed; USER/SOUL and other deployed instructions, plugins, hooks, and MCP configuration are not
part of this first experiment. Saved auth is copied temporarily when needed, never into evidence.
The runner passes the exact declared UTF-8 prompt on stdin, disables user configuration and rules,
uses workspace-write with agent-command network disabled, and requests no approvals.

PATH shims record reads and search invocations with their exit status. For a literal PASS, exact
`rg` must occur; for a known-path PASS, the target must be read without exploration. Only synthetic
`cat`, `rg`, and `fd` are installed. Other ways of reading a file can yield false negatives. The
shims simulate external tools and are not protected against a deliberately tampering agent. They
do not measure native tool quality. No final-answer self-report is used by the oracle.

## Evidence and comparison

Reports record version, case snapshots, prompt bytes/fingerprints, source fingerprints, agent and
version, requested model, Git revision and tested instruction fingerprints, fixture/executable
fingerprints, controls, environment, date, replicate count, PASS/FAIL/INVALID results, observations,
tokens/tool calls/duration when available, and limitations. Missing measurements remain null.
Timeout, nonzero exit, output overflow, broken events, or unreadable observation logs become
INVALID, never PASS. Raw transcripts and arbitrary tool arguments are not retained.

Version 1 reports produced by the previous Bun engine and retired code-search cases remain
readable and validatable. Historical reports retain their skill fingerprint and the original
versioned oracles, including structural activation and ColGrep observations. Current reports omit
`harness.skillFingerprint` and declare `shell-with-synthetic-cat-rg-fd-v1`; previous reports retain
`shell-with-synthetic-cat-rg-fd-colgrep-v1`. The old fixture identity remains valid for stored
snapshots, but it is no longer installed or selected by current cases. New reports record
`environment.runtime` instead of `environment.bun`, and fingerprint the running Arnes binary.
Comparisons across different cases, tool controls, engines, or binaries are refused.
PASS/FAIL/INVALID meanings and publication rules are preserved.

Historical validation checks the stored snapshot and recomputes its versioned oracle, not the
current harness bytes: changing a prompt or instruction does not rewrite yesterday's evidence.
The writer enforces no overwrite; it does not make Git history immutable or certify provenance.
Review retained reports before committing. Never retain secrets or private material.

Produce baseline and candidate explicitly with the same model, cases, runs, budget, and environment:

```bash
moon run harness:eval -- --model YOUR_EXACT_MODEL_ID --only repository-literal,repository-known-path --runs 3 --variant-file harness/evals/variants/no-op.md --report harness/evals/evidence/baseline.json
moon run harness:eval -- --model YOUR_EXACT_MODEL_ID --only repository-literal,repository-known-path --runs 3 --report harness/evals/evidence/candidate.json
moon run harness:compare -- harness/evals/evidence/baseline.json harness/evals/evidence/candidate.json
```

`compare` refuses mismatched recorded controls, case contracts, prompt bytes/fingerprints, fixtures,
runner, agent/version, model, environment, or replicate counts. It reports success rates, invalid
counts, failures, per-replicate regressions, and mean token/tool/duration metrics. INVALID stays in
the denominator. Results are descriptive: matching controls and a positive delta do not establish
statistical or causal uplift. Codex exposes neither paired random seeds nor attestation of a model
alias's resolved version here. Two identical reports also compare successfully as a no-op check.

## Deterministic testing and limits

`moon run harness:test-eval-runner` delegates to `arnes:test`; Cargo unit and CLI integration tests include fixture creation, actual shim execution, observation
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

## Migration verification

The port preserves deterministic scenario validation, isolated replicas, actual synthetic command
observations, bounded process execution, historical validation, exclusive publication, and controlled
comparison. Tests exercise the public Rust CLI with a synthetic Codex executable; they do not spend
LLM quota or establish live behavioral improvement. Timeout, malformed output, failure, and missing
measurement paths remain separate from success.

## Discovery sources

- Accepted ADR-038 and ADR-041 govern placement and implementation language.
- [Moon task options](https://moonrepo.dev/docs/config/project),
  [task types](https://moonrepo.dev/docs/concepts/task), and
  [native check](https://moonrepo.dev/docs/commands/check), consulted 2026-09-05 for pinned Moon 2.5.3.
- [Codex non-interactive execution](https://developers.openai.com/codex/noninteractive/), consulted
  2026-09-05 and checked against installed CLI help; no live execution was performed.
- [Reference repository](https://github.com/Syo-M/codex-frontend-skills/tree/0c25f8e1c616da6242ca09a7f3613412521cef69),
  inspected at that revision: single deterministic entry point, separate quota-consuming evaluation,
  constrained evidence, and controlled comparison. Its frontend and Codex distribution contracts
  were not copied.
