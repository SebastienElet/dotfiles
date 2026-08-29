# ColGrep Worktree Routing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development
> (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use
> checkbox (`- [ ]`) syntax for tracking.

**Goal:** Route exact worktree searches to `rg`/`fd`, conceptual worktree searches through a
fail-closed ColGrep entry point, and retain CodeGraph outside linked worktrees.

**Architecture:** A Bun/TypeScript boundary resolves the canonical linked-worktree root, updates a
ColGrep index on conceptual demand, validates ColGrep's status and persisted metadata, and buffers
and confines results before publication. The existing `codegraph` skill remains the canonical
three-agent routing policy; Makefile symlinks remain projections.

**Tech Stack:** Bun 1.4, TypeScript 7, Zod 4, Git CLI, ColGrep 1.7+, Homebrew Bundle, Bun tests.

**Spec:** `docs/superpowers/specs/2026-08-29-recherche-worktrees-design.md`

## Global Constraints

- Do not modify `tooling/daily-routine/`.
- Do not initialize ColGrep from any worktree-creation hook.
- Remove inherited Git routing variables from every root probe.
- Publish no ColGrep result until root, index metadata, clean state, and every result path pass.
- Keep CodeGraph behavior outside linked worktrees unchanged.
- Add no code comments unless they record a named external fact; list all comments at delivery.
- Exercise installation targets only with `make -n` from this worktree.

---

### Task 1: Fail-closed ColGrep process boundary

**Files:**

- Create: `tooling/colgrep-worktree`
- Create: `tooling/colgrep-worktree.ts`
- Create: `tooling/colgrep-worktree-contract.ts`
- Create: `tooling/colgrep-worktree-test-support.ts`
- Create: `tooling/colgrep-worktree.test.ts`
- Create: `tooling/colgrep-worktree-failures.test.ts`

**Interfaces:**

- Consumes: one non-empty conceptual query; `COLGREP_WORKTREE_GIT_BIN` and
  `COLGREP_WORKTREE_COLGREP_BIN` test overrides.
- Produces: JSON result array on stdout and exit `0`, or one diagnostic ending in
  `Fall back to bounded rg/fd searches.` on stderr and non-zero exit.
- Internal: `resolveLinkedWorktreeRoot(cwd, run): string`,
  `parseColgrepStatus(stdout): { projectRoot: string; indexDirectory: string }`, and
  `validateColgrepIndex(root, indexDirectory): void`.

- [ ] **Step 1: Write the linked-worktree happy-path test against the shipped entry point**

Create a real temporary Git repository and linked worktree. The fake ColGrep executable records
argv, creates `project.json` and clean `state.json` on `init`, emits strict status, then returns one
JSON result inside the linked root.

```ts
test("searches only after proving the active linked-worktree index", () => {
  const fixture = createLinkedWorktreeFixture();
  const result = runEntryPoint(
    fixture.linkedRoot,
    "authentication boundary",
    fixture.environment,
  );

  expect(result.exitCode).toBe(0);
  expect(JSON.parse(result.stdout)).toEqual([fixture.activeResult]);
  expect(readInvocations(fixture)).toEqual([
    ["init", "-y", fixture.linkedRoot],
    ["status", fixture.linkedRoot],
    [
      "search",
      "--json",
      "--no-update",
      "authentication boundary",
      fixture.linkedRoot,
    ],
  ]);
});
```

- [ ] **Step 2: Run the happy-path test and verify RED**

Run: `bun test tooling/colgrep-worktree.test.ts`

Expected: FAIL because `tooling/colgrep-worktree` does not exist.

- [ ] **Step 3: Add the smallest entry point, schemas, root resolver, and orchestration**

Use `#!/usr/bin/env bun`; reject argument counts other than one; spawn fixed argv arrays; decode
stdout as fatal UTF-8; strip `GIT_DIR`, `GIT_WORK_TREE`, `GIT_INDEX_FILE`, `GIT_COMMON_DIR`, and
`GIT_PREFIX`; canonicalize `--show-toplevel`, `--absolute-git-dir`, and absolute
`--git-common-dir`; require different Git/common dirs and an empty superproject path. Parse external
JSON once with strict Zod schemas and write buffered search output only after validation.

```ts
const projectMetadataSchema = z
  .object({
    project_path: z.string().min(1),
    project_name: z.string().min(1),
    model: z.string().min(1),
  })
  .strict();

const indexStateSchema = z
  .object({
    cli_version: z.string().min(1),
    index_format_version: z.number().int().positive(),
    files: z.record(
      z.string(),
      z
        .object({
          content_hash: z.number().nonnegative(),
          mtime: z.number().nonnegative(),
          size: z.number().nonnegative(),
        })
        .strict(),
    ),
    ignored_files: z.array(z.string()),
    search_count: z.number().int().nonnegative(),
    dirty: z.literal(false),
  })
  .strict();
```

- [ ] **Step 4: Run the happy-path test and verify GREEN**

Run: `bun test tooling/colgrep-worktree.test.ts`

Expected: PASS with no warnings.

- [ ] **Step 5: Add failure-path tests before extending production**

Table-drive these real-entry-point cases: main checkout, non-Git cwd, nested submodule, poisoned Git
environment, missing Git, missing ColGrep, empty/multiple/malformed Git output, failed init, absent
index, malformed/ambiguous status, missing or symlinked index directory, malformed `project.json`,
foreign/non-canonical project root, malformed/dirty/empty state, malformed JSON results, relative or
foreign result path, and failed ColGrep search. Assert non-zero exit, empty stdout, the bounded
fallback diagnostic, and no search invocation whenever preconditions fail.

```ts
test.each([
  "missing-project",
  "foreign-project",
  "dirty-state",
  "ambiguous-status",
])("%s refuses before index search", (mode) => {
  const fixture = createLinkedWorktreeFixture({ mode });
  const result = runEntryPoint(
    fixture.linkedRoot,
    "query",
    fixture.environment,
  );
  expect(result.exitCode).not.toBe(0);
  expect(result.stdout).toBe("");
  expect(result.stderr).toEndWith("Fall back to bounded rg/fd searches.\n");
  expect(readInvocations(fixture)).not.toContainEqual(
    expect.arrayContaining(["search"]),
  );
});
```

- [ ] **Step 6: Run failure tests and verify RED, then implement each refusal minimally**

Run after adding tests: `bun test tooling/colgrep-worktree-failures.test.ts`

Expected before production changes: FAIL on the first unsupported refusal. Add guard clauses and
rerun after each case until all pass; never loosen a schema into a plausible default.

- [ ] **Step 7: Verify Task 1 and commit**

Run:

```bash
bun test tooling/colgrep-worktree*.test.ts
bun run typecheck
bun run lint
bun run format:typescript:check
```

Commit: `feat(ai): guard ColGrep worktree searches`

### Task 2: Installation and two-worktree integration

**Files:**

- Modify: `Brewfile`
- Modify: `Makefile`
- Modify: `tooling/deployment-links.test.ts`
- Create: `tooling/colgrep-worktree-integration.test.ts`

**Interfaces:**

- Consumes: Homebrew formula `lightonai/tap/colgrep` and repository entry point.
- Produces: minimal-profile ColGrep installation and `~/.local/bin/colgrep-worktree` symlink.

- [ ] **Step 1: Write deployment and integration tests first**

Extend the deployment fixture to request `${fixture.home}/.local/bin/colgrep-worktree`, assert the
link targets the repository entry point, replay idempotently, and preserve a divergent destination.
Add an opt-in `COLGREP_INTEGRATION=1` test with isolated `COLGREP_DATA_DIR`: create main plus two
linked worktrees, commit unique symbols on each branch, modify a tracked file and add an untracked
file in A, pre-index B, then query A through the shipped entry point. Assert A's tracked/untracked
symbols are present and B's symbol absent.

- [ ] **Step 2: Run tests and verify RED**

Run:

```bash
bun test tooling/deployment-links.test.ts
COLGREP_INTEGRATION=1 bun test tooling/colgrep-worktree-integration.test.ts
```

Expected: deployment target missing; integration entry point or ColGrep dependency unavailable.

- [ ] **Step 3: Add the declarative package and symlink wiring**

Add `tap "lightonai/tap"` and `brew "lightonai/tap/colgrep", trusted: true` to `Brewfile`. Add the
wrapper to `minimal-artifacts`, `MINIMAL_SNAPSHOT_PATHS`, the smoke executable loop, a
`${LOCAL_BIN}/colgrep-worktree` symlink rule, and the CodeGraph test aggregate without adding a
ColGrep installation recipe or hook.

- [ ] **Step 4: Verify Task 2 and commit**

Run:

```bash
make -n bundle-minimal
make -n "$HOME/.local/bin/colgrep-worktree"
bun test tooling/deployment-links.test.ts tooling/colgrep-worktree*.test.ts
COLGREP_INTEGRATION=1 bun test tooling/colgrep-worktree-integration.test.ts
```

Commit: `feat(install): deploy ColGrep worktree search`

### Task 3: Canonical three-agent routing policy and ADR

**Files:**

- Modify: `harness/skills/codegraph/SKILL.md`
- Modify: `harness/skills/codegraph/evals/trigger-queries.json`
- Delete: `harness/skills/codegraph/scripts/skill_contract_test.sh`
- Modify (derived): `harness/skills/README.md`
- Modify: `docs/adr/039-codegraph-recuperation-structurelle.md`
- Modify: `docs/adr/README.md`
- Modify: `docs/codegraph.md`

**Interfaces:**

- Consumes: `colgrep-worktree <query>`, CodeGraph MCP, `rg`, and `fd`.
- Produces: one skill projected unchanged to Claude Code, Codex, and Cursor.

- [ ] **Step 1: Record the skill baseline and pressure-test RED**

Run the current skill doctor checks, preserving every PASS. Dispatch fresh-context Claude Code,
Codex, and Cursor scenarios for exact and conceptual lookup in a linked worktree plus structural
lookup in a main checkout; record CodeGraph/raw-ColGrep choices, omitted root proof, and unwanted
initialization. This current behavior is the RED evidence for the evolution contract: “linked
worktrees use only `rg`/`fd` for exact retrieval and `colgrep-worktree` for conceptual retrieval.”

- [ ] **Step 2: Remove the advisory text-mirror contract**

Delete `skill_contract_test.sh` and its Makefile/workflow invocations. It greps prose and cannot
prove agent behavior; the RED/GREEN pressure scenario is the oracle for policy, while the
TypeScript boundary and deployment tests remain automated enforcement.

- [ ] **Step 3: Evolve the skill minimally**

Keep the slug `codegraph`; update its description and steps so exact lookup stops immediately on
`rg`/`fd`, linked-worktree conceptual lookup calls only `colgrep-worktree`, and non-worktree
structural lookup follows the existing CodeGraph status/threshold flow. Explicitly label the skill
advisory and the TypeScript entry point enforcing; require source verification and forbid raw
ColGrep, CodeGraph, and hook initialization in linked worktrees.

- [ ] **Step 4: Re-run pressure scenarios, doctor, evals, and derived index**

Run the same six worktree scenarios and three main-checkout scenarios with the changed skill and
require the expected routing for each named agent. Add positive linked-worktree conceptual and
negative exact-search eval cases. Validate frontmatter, section order, gotchas, constraints, eval
JSON, adapter destinations, and `skills-ref` only if installed. Regenerate
`harness/skills/README.md` from frontmatter twice and require identical SHA-256.

- [ ] **Step 5: Amend the accepted architecture and operating documentation**

Change ADR-039's title/index and decision to scope CodeGraph outside linked worktrees and ColGrep
inside them. Record that #220 and #121 remain open for non-worktree CodeGraph/MCP behavior. Update
`docs/codegraph.md` without opportunistically correcting unrelated historical claims unless the new
routing makes them false in the touched section.

- [ ] **Step 6: Verify Task 3 and commit**

Run:

```bash
bun test tooling/deployment-links.test.ts tooling/colgrep-worktree*.test.ts
git diff --check
```

Commit: `docs(ai): route worktree retrieval through ColGrep`

### Task 4: CI coverage and final proof

**Files:**

- Modify: `.github/workflows/test-codegraph.yml`
- Create: `tooling/colgrep-worktree-ci-contract.test.ts`

**Interfaces:**

- Consumes: every TypeScript, skill, Makefile, Brewfile, and ADR file changed above.
- Produces: Linux unit/type/lint coverage and macOS real ColGrep integration coverage.

- [ ] **Step 1: Add a failing workflow contract**

Add a Bun contract test that loads `.github/workflows/test-codegraph.yml` and asserts the macOS job
installs ColGrep and runs `COLGREP_INTEGRATION=1 bun test
tooling/colgrep-worktree-integration.test.ts`; verify RED before editing the workflow.

- [ ] **Step 2: Extend the CodeGraph workflow without duplicating global TypeScript gates**

Install the minimal Brew bundle on macOS, assert `colgrep --version`, run all
`tooling/colgrep-worktree*.test.ts` with `COLGREP_INTEGRATION=1`, and retain existing CodeGraph tests.
Ensure path filters include the new tooling, skill, Brewfile, Makefile, docs/ADR, and workflow.

- [ ] **Step 3: Run the complete fresh verification barrier**

Run in this macOS worktree with Volta first in `PATH`:

```bash
PATH="$HOME/.volta/bin:$PATH" bun test
bun run typecheck
bun run lint
bun run format:typescript:check
make -n minimal
make -n codegraph-test
COLGREP_INTEGRATION=1 bun test tooling/colgrep-worktree-integration.test.ts
git diff --check
git status --short
```

Read every output and confirm each touched extension is covered. Inspect every changed production
function against 50 logical lines and every hand-written file against 250 lines; split or justify.

- [ ] **Step 4: Run adversarial enforcement review and fix blockers**

Dispatch a fresh-context agent to bypass the guard using poisoned Git variables, symlinked cwd,
submodule/nested repository, malformed provider output, foreign metadata, alternate direct paths,
and race-shaped index replacement. Convert every cheap real bypass into an automated test, repeat
RED/GREEN, and rerun the full barrier.

- [ ] **Step 5: Commit final CI changes**

Commit: `ci(ai): verify ColGrep worktree isolation`
