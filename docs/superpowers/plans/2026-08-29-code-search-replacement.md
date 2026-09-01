# Code Search Replacement Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace every repository-managed CodeGraph surface with a technology-neutral `code-search` capability backed by `rg`/`fd` and guarded ColGrep retrieval.

**Architecture:** Agents consume one shared `code-search` skill. Exact lookup stays on `rg`/`fd`; conceptual lookup calls `colgrep-search`, which proves and confines the canonical root of any Git checkout before publishing buffered JSON results. CodeGraph installation, MCP configuration, tooling, tests, documentation, ignore state, and CI disappear without a migration or uninstall script.

**Tech Stack:** Bun 1.4, TypeScript 7, Zod 4, Git CLI, ColGrep 1.7, Homebrew, GNU Make, GitHub Actions

**Spec:** `docs/superpowers/specs/2026-08-29-code-search-design.md`

## Global Constraints

- No CodeGraph executable, MCP registration, skill, target, test utility, workflow, or active operational documentation remains managed by the repository.
- Do not add a migration, uninstall helper, cleanup target, compatibility alias, or eager initialization hook.
- `code-search` is the only public skill name; `colgrep-search` is the only installed conceptual-search command.
- Exact lookup never initializes or invokes ColGrep.
- Conceptual lookup accepts both a main checkout and a linked worktree only after canonical Git-root proof.
- Any missing, ambiguous, stale, dirty, malformed, symlinked, or foreign evidence fails closed with empty stdout and a bounded `rg`/`fd` fallback diagnostic.
- Every changed TypeScript file is covered by Oxlint, Oxfmt, TypeScript 7, and behavioral tests.

---

### Task 1: Generalize the guarded ColGrep command

**Files:**

- Rename: `tooling/colgrep-worktree.ts` → `tooling/colgrep-search.ts`
- Rename: `tooling/colgrep-worktree-contract.ts` → `tooling/colgrep-search-contract.ts`
- Rename: `tooling/colgrep-worktree-cli.ts` → `tooling/colgrep-search-cli.ts`
- Rename: `tooling/colgrep-worktree-test-support.ts` → `tooling/colgrep-search-test-support.ts`
- Rename: `tooling/colgrep-worktree-test-provider.ts` → `tooling/colgrep-search-test-provider.ts`
- Rename: `tooling/colgrep-worktree-git-test-provider.ts` → `tooling/colgrep-search-git-test-provider.ts`
- Rename: `tooling/colgrep-worktree.test.ts` → `tooling/colgrep-search.test.ts`
- Rename: `tooling/colgrep-worktree-failures.test.ts` → `tooling/colgrep-search-failures.test.ts`
- Rename: `tooling/colgrep-worktree-integration.test.ts` → `tooling/colgrep-search-integration.test.ts`

**Interfaces:**

- Consumes: `git rev-parse --show-toplevel`, `git rev-parse --show-superproject-working-tree`, ColGrep `init`, `status`, and `search`.
- Produces: executable `colgrep-search <conceptual-query>` and `resolveCheckoutRoot(cwd, git, run): string`.

- [ ] **Step 1: Write the failing main-checkout test**

Replace the old refusal assertion with an accepted-root assertion:

```ts
test("searches the canonical main checkout", () => {
  const fixture = createCheckoutFixture();
  const result = runEntryPoint(
    fixture.mainRoot,
    "main symbol",
    fixture.mainEnvironment,
  );

  expect(result.exitCode).toBe(0);
  expect(JSON.parse(result.stdout)).toEqual([fixture.mainResult]);
});
```

- [ ] **Step 2: Run the focused test and observe RED**

Run: `PATH="$HOME/.volta/bin:$PATH" bun test tooling/colgrep-worktree-failures.test.ts`

Expected: FAIL because `resolveLinkedWorktreeRoot` rejects equal Git and common directories.

- [ ] **Step 3: Rename the surface and simplify root proof**

Use the `colgrep-search` prefix for files, environment overrides, diagnostics, and fixtures. Replace linked-worktree detection with:

```ts
function resolveCheckoutRoot(
  cwd: string,
  git: string,
  run: RunCommand,
): string {
  const environment = gitEnvironment();
  const root = canonicalPath(
    singleLine(
      runRequired(git, ["-C", cwd, "rev-parse", "--show-toplevel"], {
        environment,
        run,
      }).stdout,
      "Git checkout root",
    ),
  );
  const superproject = runRequired(
    git,
    ["-C", root, "rev-parse", "--show-superproject-working-tree"],
    { environment, run },
  ).stdout;
  if (superproject !== "") throw new Error("the active Git root is ambiguous");
  return root;
}
```

Keep all existing index metadata, UTF-8, buffering, state, symlink, and result-confinement checks.

- [ ] **Step 4: Expand the real divergent-checkout oracle**

Create unique main, active-worktree, and neighbor-worktree symbols. Pre-index one checkout, query all three through `colgrep-search`, and assert each response contains its own tracked/untracked symbols and excludes both foreign symbols.

- [ ] **Step 5: Run the focused barriers**

Run:

```bash
PATH="$HOME/.volta/bin:$PATH" bun run format:typescript
PATH="$HOME/.volta/bin:$PATH" bun run lint
PATH="$HOME/.volta/bin:$PATH" bun run typecheck
PATH="$HOME/.volta/bin:$PATH" bun test tooling/colgrep-search*.test.ts
COLGREP_INTEGRATION=1 PATH="$HOME/.volta/bin:$PATH" bun test tooling/colgrep-search-integration.test.ts
```

Expected: unit refusal/success suite green; real three-checkout isolation green.

- [ ] **Step 6: Commit**

```bash
git add tooling/colgrep-*
git commit -m "feat(ai): search every Git checkout with guarded ColGrep"
```

### Task 2: Replace the public skill and deployment surface

**Files:**

- Rename: `harness/skills/codegraph/` → `harness/skills/code-search/`
- Modify: `harness/skills/README.md`
- Modify: `home/.arnes.yaml`
- Modify: `Makefile`
- Modify: `tooling/deployment-links.test.ts`

**Interfaces:**

- Consumes: installed `colgrep-search`, `rg`, and `fd`.
- Produces: shared skill `code-search` projected to Claude Code, Codex, and Cursor.

- [ ] **Step 1: Make deployment tests expect the new names**

Change every expected shared-skill set from `codegraph` to `code-search`, and change the command destination assertion to:

```ts
const destination = join(fixture.home, ".local", "bin", "colgrep-search");
expect(linkTarget(destination)).toBe(
  join(project, "tooling", "colgrep-search-cli.ts"),
);
```

- [ ] **Step 2: Run deployment tests and observe RED**

Run: `PATH="$HOME/.volta/bin:$PATH" bun test tooling/deployment-links.test.ts`

Expected: FAIL because Make still exposes the old skill and command names.

- [ ] **Step 3: Write the technology-neutral skill**

Set frontmatter `name: code-search`. Its first description sentence becomes `Search codebases with exact and conceptual retrieval.` Steps must classify exact lookup first, invoke only `colgrep-search '<query>'` for conceptual lookup, verify important findings in source, and fall back to bounded `rg`/`fd` on refusal. Constraints explicitly forbid raw `colgrep`, CodeGraph, and eager hooks.

- [ ] **Step 4: Update Make projections without cleanup logic**

Replace the three agent projection targets and dependencies with `code-search`; change the canonical Arnes manifest slug to `code-search`; deploy `${LOCAL_BIN}/colgrep-search`; remove old names from snapshot and smoke lists. Do not add deletion recipes for already-installed CodeGraph artifacts.

- [ ] **Step 5: Pressure-test routing in fresh contexts**

For Claude Code, Codex, and Cursor, exercise exact lookup, conceptual lookup in main and linked checkouts, Git-proof failure, and wrapper failure. Each report must show no CodeGraph/raw-ColGrep/hook path.

- [ ] **Step 6: Run deployment and skill formatting checks**

Run:

```bash
PATH="$HOME/.volta/bin:$PATH" bun test tooling/deployment-links.test.ts
prettier --check harness/skills/code-search/SKILL.md harness/skills/code-search/evals/trigger-queries.json harness/skills/README.md
make -n "$HOME/.local/bin/colgrep-search"
```

- [ ] **Step 7: Commit**

```bash
git add Makefile home/.arnes.yaml harness/skills/code-search harness/skills/README.md tooling/deployment-links.test.ts
git commit -m "feat(ai): expose technology-neutral code search"
```

### Task 3: Remove every managed CodeGraph artifact

**Files:**

- Delete: `tooling/codegraph*` and `tooling/codegraph/**`
- Delete: `docs/codegraph.md`, `docs/codegraph-validation.md`
- Delete: `.github/workflows/test-codegraph.yml`
- Modify: `home/.config/git/ignore`, `home/.config/cspell/user.txt`, `Makefile`
- Modify: `docs/issue-257-installation-macos-audit.md`, `docs/issue-257-installation-macos-design.md`

**Interfaces:**

- Consumes: the working `code-search` deployment from Task 2.
- Produces: a repository with no active CodeGraph installation, MCP, tooling, test, ignore, or CI surface.

- [ ] **Step 1: Add a removal contract test**

Create `tooling/code-search-removal-contract.test.ts` that enumerates forbidden tracked paths and active configuration tokens:

```ts
test("repository manages no CodeGraph runtime surface", () => {
  expect(trackedPaths.filter((path) => path.includes("codegraph"))).toEqual([]);
  expect(makefile).not.toMatch(/codegraph|CODEGRAPH/iu);
  expect(globalIgnore).not.toContain(".codegraph/");
});
```

Allow the ADR, replacement spec, and issue-history prose to name the removed technology; do not allow executable paths, workflow names, skill slugs, Make targets, or operational docs.

- [ ] **Step 2: Run the removal contract and observe RED**

Run: `PATH="$HOME/.volta/bin:$PATH" bun test tooling/code-search-removal-contract.test.ts`

Expected: FAIL listing current CodeGraph runtime files and Make tokens.

- [ ] **Step 3: Delete owned CodeGraph runtime and tests**

Remove configuration writers, repository measurement, MCP/network probes, fixtures, launchers, and their tests. Remove CodeGraph targets and Volta artifacts from Make. Remove `.codegraph/` from the global ignore and `codegraph` from the user spelling dictionary. Keep `tokei`, which remains an independent terminal tool.

- [ ] **Step 4: Correct active installation documentation**

Replace CodeGraph references in the current macOS installation design/audit with `code-search` and ColGrep where the installed capability is intended. Delete the obsolete operational and validation documents instead of preserving historical instructions as current behavior.

- [ ] **Step 5: Run removal and global TypeScript gates**

Run:

```bash
PATH="$HOME/.volta/bin:$PATH" bun test tooling/code-search-removal-contract.test.ts
PATH="$HOME/.volta/bin:$PATH" bun run lint
PATH="$HOME/.volta/bin:$PATH" bun run typecheck
PATH="$HOME/.volta/bin:$PATH" bun run format:typescript:check
git diff --check
```

- [ ] **Step 6: Commit**

```bash
git add -A tooling docs/codegraph.md docs/codegraph-validation.md home/.config/git/ignore home/.config/cspell/user.txt Makefile docs/issue-257-installation-macos-*.md
git commit -m "refactor(ai): remove CodeGraph from managed tooling"
```

### Task 4: Replace ADR-039 and CI with CodeSearch

**Files:**

- Rename: `docs/adr/039-codegraph-recuperation-structurelle.md` → `docs/adr/039-code-search.md`
- Modify: `docs/adr/README.md`
- Create: `docs/code-search.md`
- Create: `.github/workflows/test-code-search.yml`
- Modify: `.github/workflows/lint.yml`
- Rename: `tooling/colgrep-worktree-ci-contract.test.ts` → `tooling/code-search-ci-contract.test.ts`
- Modify: PR #261 title/body after push

**Interfaces:**

- Consumes: `code-search`, `colgrep-search`, Homebrew ColGrep.
- Produces: accepted architecture and merge-blocking CI for the replacement.

- [ ] **Step 1: Rewrite the CI contract to expect replacement-only paths**

Assert two path filters for `harness/skills/code-search/**` and `tooling/colgrep-search*`, installation of `${HOME}/.local/bin/colgrep-search`, the real `COLGREP_INTEGRATION=1` test, and absence of `codegraph` tokens from the workflow.

- [ ] **Step 2: Run the CI contract and observe RED**

Run: `PATH="$HOME/.volta/bin:$PATH" bun test tooling/code-search-ci-contract.test.ts`

Expected: FAIL until the workflow and lint paths use CodeSearch names.

- [ ] **Step 3: Replace the architecture and operations docs**

ADR-039 must state the exact/conceptual split, guarded canonical root, on-demand initialization, no hooks, no CodeGraph, and no automated migration. `docs/code-search.md` documents installation through `make minimal`, agent routing, refusal behavior, and the manual responsibility for old local artifacts.

- [ ] **Step 4: Create the lean CodeSearch workflow**

Use macOS to install Bun dependencies and the minimal Homebrew bundle, deploy `colgrep-search`, print `colgrep --version`, run deployment/unit tests, run the real three-checkout integration, then typecheck. Remove Linux measurement and every CodeGraph job or command.

- [ ] **Step 5: Run all final barriers**

Run:

```bash
PATH="$HOME/.volta/bin:$PATH" bun test
PATH="$HOME/.volta/bin:$PATH" bun run lint
PATH="$HOME/.volta/bin:$PATH" bun run typecheck
PATH="$HOME/.volta/bin:$PATH" bun run format:typescript:check
prettier --check .github/workflows/lint.yml .github/workflows/test-code-search.yml docs/adr/039-code-search.md docs/adr/README.md docs/code-search.md docs/superpowers/specs/2026-08-29-code-search-design.md docs/superpowers/plans/2026-08-29-code-search-replacement.md harness/skills/code-search/SKILL.md harness/skills/code-search/evals/trigger-queries.json harness/skills/README.md
actionlint
COLGREP_INTEGRATION=1 PATH="$HOME/.volta/bin:$PATH" bun test tooling/colgrep-search-integration.test.ts
git diff --check main...HEAD
```

Expected: all standard tests pass; only unrelated opt-in suites skip in the default run; real ColGrep isolation passes.

- [ ] **Step 6: Audit simplicity and remaining references**

Run `rg -n -i 'codegraph' Makefile Brewfile home harness tooling .github docs`. Every remaining match must be replacement history in ADR-039, the approved spec/plan, or adjacent issue prose; no runtime, skill, CI, installation, or current-operations match is allowed. Inspect every changed production function against 50 logical lines and every hand-written file against 250 lines.

- [ ] **Step 7: Commit and publish one reviewed head**

```bash
git add -A
git commit -m "ci(ai): verify CodeSearch replacement"
```

Request an independent review, correct all Critical/Important findings, push normally to PR #261, update its title/body to state complete CodeGraph removal, and wait for the replacement workflow.
