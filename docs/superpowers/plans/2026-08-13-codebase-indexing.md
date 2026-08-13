# Codebase Indexing Evaluation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Measure CodeGraph, Serena, Zoekt, and `rg` on one private large corpus, select the admissible winner, and deploy one local indexing workflow to Claude Code, Codex, and Cursor without publishing corpus identifiers or source.

**Architecture:** A repository-owned Bash harness stores all sensitive inputs and raw traces under `${XDG_STATE_HOME:-$HOME/.local/state}/dotfiles/code-indexing`, while the public repository contains only the protocol, schemas, aggregate results, and redacted evidence. Native candidate interfaces are benchmarked first; only the selected backend is wrapped by the shared `code-index` lifecycle command and exposed through stdio MCP. A do-nothing runbook preserves manual truth-grounding and blind-scoring steps without mixing them into automated measurement steps.

**Tech Stack:** Portable Bash 3.2, `jq`, `/usr/bin/time` on macOS, Git, CodeGraph 1.5.0, Serena 1.5.3 through `uvx`, Zoekt pinned to commit `c6cd01494dc04d60883f7ae4c4e02ccdc97647c3`, MCP Inspector 2.2.0, Claude Code, Codex CLI, Cursor Agent, Make.

---

## File map

- `code-indexing/protocol.json`: public benchmark contract, weights, barriers, candidate pins, scenarios, and output schema version.
- `code-indexing/tasks.example.json`: redacted twelve-task shape with category and facet fields but no private paths, symbols, or answers.
- `code-indexing/results.json`: aggregate, source-free measurements and decision generated from private traces.
- `scripts/code_index_benchmark`: runbook and automated collector; never stores sensitive output in the repository.
- `scripts/code_index_benchmark_test`: fixture-based tests for privacy, validation, timing record shape, and fail-closed behavior.
- `scripts/code_index`: selected-backend diagnostic and lifecycle command (`doctor`, `init`, `sync`, `serve`, `clean`).
- `scripts/code_index_test`: fixture-based tests for threshold, coverage, freshness, exclusions, failures, and safe cleanup.
- `scripts/code_index_configure`: idempotent multi-agent MCP configuration using each client's supported interface.
- `scripts/code_index_configure_test`: isolated-home tests proving existing MCP configuration is preserved.
- `.agents/skills/code-indexing/SKILL.md`: shared routing and degraded-mode workflow.
- `.agents/skills/code-indexing/evals/trigger-queries.json`: positive and negative activation scenarios.
- `.agents/skills/README.md`: generated shared-skill index.
- `ai/AGENTS.md`: minimal measured routing rule, only if marginal ablation passes.
- `Makefile`: pinned benchmark dependencies, selected runtime, configuration target, and skill distribution.
- `docs/adr/038-indexation-code-agents.md`: final decision and relation to ADR-015/025/028/033/036.
- `docs/adr/README.md`: ADR-038 index entry.
- `docs/code-indexing-benchmark.md`: reproducible method, sanitized results, limitations, and removal instructions.

### Task 1: Freeze the public benchmark contract

**Files:**
- Create: `code-indexing/protocol.json`
- Create: `code-indexing/tasks.example.json`
- Test: `scripts/code_index_benchmark_test`

- [ ] **Step 1: Write the failing contract test**

Add a test that requires exactly four candidates, twelve task slots split 3/3/3/3, weights totaling
100, the 75-point acceptance score, the 200000-token lower bound, the 10-point quality delta, the
25-percent latency delta, three repetitions, local-only mode, and no corpus path or identifier.

- [ ] **Step 2: Run the test and verify RED**

Run: `bash scripts/code_index_benchmark_test contract`

Expected: non-zero with `code-indexing/protocol.json is missing`.

- [ ] **Step 3: Add the minimal JSON contract and redacted task template**

The candidate records must pin:

```json
{
  "codegraph": "1.5.0",
  "serena": "1.5.3",
  "zoekt": "c6cd01494dc04d60883f7ae4c4e02ccdc97647c3",
  "rg": "15.2.0"
}
```

Each task template contains only `id`, `category`, `question`, `expected_files`,
`expected_symbols`, `required_facets`, and `known_false_positives`, with private values represented
by empty arrays or neutral example strings.

- [ ] **Step 4: Run the contract test and verify GREEN**

Run: `bash scripts/code_index_benchmark_test contract`

Expected: `ok: contract`.

- [ ] **Step 5: Commit**

```bash
git add code-indexing/protocol.json code-indexing/tasks.example.json scripts/code_index_benchmark_test
git commit -m "test(indexing): freeze benchmark contract"
```

### Task 2: Build the privacy-preserving benchmark runbook

**Files:**
- Modify: `scripts/code_index_benchmark_test`
- Create: `scripts/code_index_benchmark`

- [ ] **Step 1: Add failing tests for state placement and refusal paths**

Cover missing `CODE_INDEX_CORPUS`, a non-Git corpus, a state directory inside the public checkout,
missing private `tasks.json`, malformed task count, missing candidate executable, and raw output
attempted under the checkout. Each unavailable correctness probe must fail non-zero and name the
missing condition.

- [ ] **Step 2: Run the tests and verify RED**

Run: `bash scripts/code_index_benchmark_test runbook`

Expected: non-zero because `scripts/code_index_benchmark` does not exist.

- [ ] **Step 3: Implement one function per runbook step**

Implement these stable functions in order:

```text
validate_environment
record_host
freeze_corpus
prepare_private_tasks
verify_truth_ground
prepare_candidate
measure_initial_index
measure_queries
measure_incremental_changes
measure_failure_modes
score_blind_facets
aggregate_results
verify_redaction
```

Automated functions execute their checks; `prepare_private_tasks`, `verify_truth_ground`, and
`score_blind_facets` print exact paths and commands then call one shared `wait_for_enter`. Support
`--yes` only for tests and only when `CODE_INDEX_TEST_MODE=1`; production runs must stop for manual
steps. Default state is `${XDG_STATE_HOME:-$HOME/.local/state}/dotfiles/code-indexing` and the
script refuses any state path contained by the dotfiles checkout.

- [ ] **Step 4: Record measurements without source content**

Every run record contains candidate, pinned version, task ID, repetition, scenario, exit status,
elapsed milliseconds, maximum RSS bytes, CPU user/system seconds, index bytes, freshness verdict,
result count, and SHA-256 of the raw trace. Raw stdout/stderr remain in state; no source text enters
the aggregate record.

- [ ] **Step 5: Run tests and syntax checks**

Run: `bash scripts/code_index_benchmark_test runbook && bash -n scripts/code_index_benchmark scripts/code_index_benchmark_test`

Expected: `ok: runbook`, then exit 0.

- [ ] **Step 6: Commit**

```bash
git add scripts/code_index_benchmark scripts/code_index_benchmark_test
git commit -m "feat(indexing): add private benchmark runbook"
```

### Task 3: Install pinned benchmark dependencies through the Makefile

**Files:**
- Modify: `Makefile`
- Modify: `scripts/code_index_benchmark_test`

- [ ] **Step 1: Add a failing dry-run assertion**

Require a `code-index-benchmark-tools` target that depends on CodeGraph 1.5.0, Serena 1.5.3, MCP
Inspector 2.2.0, Zoekt at the pinned commit, Go, and Universal Ctags. Assert that no recipe invokes
a vendor configuration installer.

- [ ] **Step 2: Run the test and verify RED**

Run: `bash scripts/code_index_benchmark_test makefile`

Expected: non-zero with `missing target: code-index-benchmark-tools`.

- [ ] **Step 3: Add minimal Makefile targets**

Use Homebrew targets for Go and Universal Ctags, Volta for CodeGraph and MCP Inspector, `uv tool`
for Serena, and `go install ...@c6cd01494dc04d60883f7ae4c4e02ccdc97647c3` for the three Zoekt
commands. Do not call `codegraph install`, `serena config`, `claude mcp add`, `codex mcp add`, or
write Cursor configuration from dependency targets.

- [ ] **Step 4: Verify the installation graph without mutation**

Run: `make -n code-index-benchmark-tools`

Expected: pinned install commands only; no agent configuration writes.

- [ ] **Step 5: Run the Makefile test and commit**

```bash
bash scripts/code_index_benchmark_test makefile
git add Makefile scripts/code_index_benchmark_test
git commit -m "feat(indexing): pin benchmark tools"
```

### Task 4: Execute the four native benchmark arms

**Files:**
- Create: `code-indexing/results.json`
- Modify: `scripts/code_index_benchmark` only when an observed runner defect has a failing test first

- [ ] **Step 1: Inspect then install from the canonical checkout**

Run: `make code-index-benchmark-tools`

Expected: all pinned commands resolve; CodeGraph telemetry reports disabled under `DO_NOT_TRACK=1`.

- [ ] **Step 2: Prepare private inputs**

Set `CODE_INDEX_CORPUS` only in the shell running the benchmark. Copy
`code-indexing/tasks.example.json` to the private state directory, fill the twelve questions and
truth facets there, and record the frozen corpus commit. Never print the corpus path into a tracked
file or commit message.

- [ ] **Step 3: Run CodeGraph, Serena, Zoekt, and rg**

Run: `DO_NOT_TRACK=1 CODEGRAPH_TELEMETRY=0 CODEGRAPH_NO_UPDATE_CHECK=1 scripts/code_index_benchmark all`

Expected: three repetitions for twelve tasks and every mutation/failure scenario per candidate;
raw traces under private state only.

- [ ] **Step 4: Blind-score and aggregate**

Use the runbook's shuffled trace identifiers. Complete facet scores without candidate labels, then
run `scripts/code_index_benchmark aggregate`.

Expected: `code-indexing/results.json` contains only aggregate values, trace hashes, environment,
pins, limitations, barrier verdicts, total scores, and one decision: candidate slug or `none`.

- [ ] **Step 5: Audit public output for professional data**

Run the benchmark redaction check plus a repository search for the private corpus basename,
absolute path, task symbols, and expected files.

Expected: no match in tracked or staged files.

- [ ] **Step 6: Commit the source-free result**

```bash
git add code-indexing/results.json
git commit -m "docs(indexing): record benchmark results"
```

### Task 5: Implement the selected lifecycle command with TDD

**Files:**
- Create: `scripts/code_index_test`
- Create: `scripts/code_index`
- Modify: `Makefile`

- [ ] **Step 1: Read the selected backend from results**

Run: `jq -er '.decision | select(. == "codegraph" or . == "serena" or . == "zoekt" or . == "none")' code-indexing/results.json`

Expected: one allowed decision. If `none`, skip backend initialization/serve implementation and
make `doctor` return `below-threshold` or `unsupported`; continue with the negative ADR path.

- [ ] **Step 2: Write failing doctor tests**

Use temporary Git repositories to cover: below threshold, required above threshold, ignored and
untracked files excluded, binary files excluded, unsupported language coverage below 90%, fresh
anchor, dirty tracked file, staged file, commit change, branch change, missing backend, missing
index, corrupt anchor, unreadable repository, and state directory inside the checkout.

- [ ] **Step 3: Run and verify RED**

Run: `bash scripts/code_index_test doctor`

Expected: non-zero because `scripts/code_index` does not exist.

- [ ] **Step 4: Implement `doctor` minimally**

The command outputs one JSON object with `verdict`, `token_estimate`, `eligible_files`,
`covered_files`, `coverage_ratio`, `freshness`, `backend`, and `reason`. Use `git ls-files`, byte
count divided by four as the conservative token estimate, extension coverage from the selected
backend record, and an anchor containing `HEAD`, branch, staged-tree hash, and tracked-dirty hash.
Return non-zero only when an index is required but unhealthy.

- [ ] **Step 5: Add failing lifecycle and cleanup tests**

Cover `init`, `sync`, `serve`, and `clean`; backend failures must propagate. Cleanup must preview by
default, require `--force`, resolve the exact state path, refuse `/`, home, checkout root, empty
paths, symlinks leaving state, and any index not carrying the expected repository identity.

- [ ] **Step 6: Implement lifecycle commands and verify GREEN**

Map commands only to the selected backend's pinned interface. Set `DO_NOT_TRACK=1` and any
backend-specific offline flags in the launcher. Write the freshness anchor atomically only after a
successful `init` or `sync`.

Run: `bash scripts/code_index_test && bash -n scripts/code_index scripts/code_index_test`

Expected: all cases print `ok` and exit 0.

- [ ] **Step 7: Add the selected runtime and local symlink target**

Add `code-index` to `ai` only when the decision is not `none`; link `${LOCAL_BIN}/code-index` to
the repository script and depend on the selected pinned runtime, not all benchmark candidates.

- [ ] **Step 8: Commit**

```bash
git add scripts/code_index scripts/code_index_test Makefile
git commit -m "feat(indexing): add shared index lifecycle"
```

### Task 6: Configure MCP safely for all three agents

**Files:**
- Create: `scripts/code_index_configure_test`
- Create: `scripts/code_index_configure`
- Modify: `Makefile`

- [ ] **Step 1: Write failing isolated-home tests**

Create fake `claude`, `codex`, and `cursor-agent` executables and seeded Claude/Codex/Cursor MCP
files. Require idempotent addition of one `code-index` stdio server, preservation of unrelated
servers and keys, explicit `DO_NOT_TRACK=1`, and refusal on malformed JSON/TOML or missing client.

- [ ] **Step 2: Run and verify RED**

Run: `bash scripts/code_index_configure_test`

Expected: non-zero because the configurator does not exist.

- [ ] **Step 3: Implement client-native configuration**

Use `claude mcp add --scope user`, `codex mcp add`, and an atomic `jq` merge for
`~/.cursor/mcp.json`. First inspect existing destinations; refuse unexpected symlinks and malformed
files. Never call a candidate's installer. Provide `--remove` with the same preservation rules.

- [ ] **Step 4: Verify GREEN and Makefile dry-run**

Run: `bash scripts/code_index_configure_test && make -n code-index-configure`

Expected: tests print `ok`; dry-run invokes only the repository configurator and selected runtime.

- [ ] **Step 5: Apply configuration on the canonical checkout**

Run: `make code-index-configure`, then list MCP servers with each client's native command.

Expected: `code-index` is present once for Claude Code, Codex, and Cursor, with no unrelated entry
changed.

- [ ] **Step 6: Commit**

```bash
git add scripts/code_index_configure scripts/code_index_configure_test Makefile
git commit -m "feat(indexing): configure shared MCP clients"
```

### Task 7: Create and forward-test the shared indexing skill

**Files:**
- Create: `.agents/skills/code-indexing/SKILL.md`
- Create: `.agents/skills/code-indexing/evals/trigger-queries.json`
- Modify: `.agents/skills/README.md`
- Modify: `Makefile`

- [ ] **Step 1: Run RED activation scenarios without the skill**

Use fresh sessions for Claude Code, Codex, and Cursor on: broad large-repository exploration, exact
literal in this repository, stale index, unsupported language, and failed refresh. Record raw
outcomes privately and preserve aggregate activation/search counts publicly.

- [ ] **Step 2: Create the skill through `skill-manager create code-indexing`**

Use category `dev`. Its description must trigger on broad exploration, architecture discovery,
symbol/reference/call-graph work, impact analysis, and cross-package research. The body must route
exact lookups to `rg`/`fd`, require `doctor` then `init`/`sync`, validate critical findings against
source and `git status`, and permit only one repair before bounded reported fallback.

- [ ] **Step 3: Add positive and negative eval queries**

Include the five RED scenarios plus adjacent small-repository and known-path negatives. Do not put
the private corpus name, path, symbols, or files in the eval JSON.

- [ ] **Step 4: Run GREEN scenarios with the skill**

Repeat identical prompts on all three agents. Require correct index use for broad eligible work,
zero index calls for exact/small work, stale detection before answer, and reported bounded fallback.

- [ ] **Step 5: Validate and distribute**

Run `skill-manager doctor code-indexing`, `skills-ref validate` when installed, deterministic
`sync-index` twice, then add explicit global skill symlink targets for Claude Code, Codex, and
Cursor in the Makefile.

- [ ] **Step 6: Commit**

```bash
git add .agents/skills/code-indexing .agents/skills/README.md Makefile
git commit -m "feat(skills): route large codebase indexing"
```

### Task 8: Measure and admit the shared instruction rule

**Files:**
- Create: `code-indexing/adoption-results.json`
- Modify: `ai/AGENTS.md` only if the measured rule passes

- [ ] **Step 1: Freeze the candidate rule text**

Keep it agent-agnostic and no longer than the current Context Management paragraph. It must defer
the detailed matrix to `code-indexing` and replace, not duplicate, the conflicting blanket
delegation rule.

- [ ] **Step 2: Run marginal ablation**

For each supported agent, run three replicates with the candidate text, three without it, and the
placebo required by ADR-036. Record index invocations, freshness checks, shell search count/bytes,
latency, and blind quality facets. Keep prompts and raw outputs private.

- [ ] **Step 3: Apply the decision rule**

Admit the rule only if it materially increases correct index routing on the large corpus, never
activates the index on the small control, never suppresses stale detection, and causes no retrieval
quality regression. Otherwise leave `ai/AGENTS.md` unchanged and record skill-only governance.

- [ ] **Step 4: Verify adapters**

Run `make -n claude-code codex cursor` and confirm the shared instruction source and skill links are
used without duplicated policy in agent-specific files.

- [ ] **Step 5: Commit evidence and any admitted rule**

```bash
git add code-indexing/adoption-results.json ai/AGENTS.md
git commit -m "feat(ai): govern code index routing"
```

Omit `ai/AGENTS.md` from `git add` when the rule fails.

### Task 9: Record ADR-038 and the reproducible report

**Files:**
- Create: `docs/adr/038-indexation-code-agents.md`
- Modify: `docs/adr/README.md`
- Create: `docs/code-indexing-benchmark.md`

- [ ] **Step 1: Write the report from aggregate evidence**

Document the method, exact public pins, host environment, scoring, aggregate measurements,
barriers, threshold, limitations, selected candidate or `none`, installation, freshness, quota,
upgrade, cleanup, and removal. Refer to the corpus only as the private large corpus.

- [ ] **Step 2: Write ADR-038**

Record the decision and consequences. State that no hook intercepts searches, ADR-033 remains in
force, the workflow complements ADR-015, adapters obey ADR-025/028, and the instruction rule obeys
ADR-036. If no candidate passes, record `rg` as the retained solution and do not ship dormant MCP
configuration.

- [ ] **Step 3: Update the ADR index and scan confidentiality**

Search every tracked change for corpus basenames, absolute paths, private symbols, source excerpts,
organization names, and raw trace content. Any match blocks the commit.

- [ ] **Step 4: Commit**

```bash
git add docs/adr/038-indexation-code-agents.md docs/adr/README.md docs/code-indexing-benchmark.md
git commit -m "docs: decide codebase indexing strategy"
```

### Task 10: Final verification and issue closure

**Files:**
- Modify only files needed to fix a reproduced verification failure

- [ ] **Step 1: Run executable coverage**

```bash
bash scripts/code_index_benchmark_test
bash scripts/code_index_test
bash scripts/code_index_configure_test
bash -n scripts/code_index_benchmark scripts/code_index_benchmark_test
bash -n scripts/code_index scripts/code_index_test
bash -n scripts/code_index_configure scripts/code_index_configure_test
shellcheck --severity=error scripts/code_index_benchmark scripts/code_index_benchmark_test scripts/code_index scripts/code_index_test scripts/code_index_configure scripts/code_index_configure_test
```

Expected: all tests and syntax/lint checks pass on macOS.

- [ ] **Step 2: Verify configuration and generated artifacts**

Run `make -n code-index-benchmark-tools code-index-configure claude-code codex cursor`, skill doctor,
two index syncs, JSON parsing on every new JSON file, and MCP tool listing in Claude Code, Codex,
and Cursor.

- [ ] **Step 3: Verify privacy and repository state**

Run the redaction audit, `git diff --check`, `git status --short`, and confirm generated indexes,
raw traces, state anchors, corpus tasks, and machine paths are absent from tracked files and commit
history introduced by this branch.

- [ ] **Step 4: Name verification scope**

Record that runtime, Bash, Makefile dry-runs, and client integrations were exercised on the current
macOS host. List Linux and any unavailable Cursor surface as unexercised or blocking; never infer
cross-platform success.

- [ ] **Step 5: Request code review**

Use `superpowers:requesting-code-review`, address findings through
`superpowers:receiving-code-review`, and rerun every affected barrier.

- [ ] **Step 6: Close #86 only after integration**

Post the source-free decision, scores, selected threshold, three-agent proof, and verification
environment. Close the issue only when every acceptance criterion is either satisfied or recorded
as a blocking negative conclusion.

## Comments budget

New source comments are not planned. Existing Makefile comments may be adjusted only when an
external tool behavior cannot be expressed by names or structure; every such comment must name the
upstream fact and appear in the delivery note.
