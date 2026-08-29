---
name: codegraph
description: >
  Explore large repositories structurally with CodeGraph. Use when auditing an unfamiliar
  repository, or when locating architecture, call paths, dependencies, cross-package behavior,
  change impact, validating an existing .codegraph index, or routing conceptual retrieval in a Git
  linked worktree through ColGrep. Make sure to use this skill before any broad exploration,
  including an audit or a technical-debt review, even if neither tool was requested.
metadata:
  category: dev
---

# CodeGraph

## Overview

Route open-ended structural retrieval by checkout type: ColGrep in a Git linked worktree, CodeGraph
outside one. Keep exact searches on `rg` and `fd`, and verify important retrieval claims in the
source before editing.

## Usage

Invoke this skill for architecture, call paths, dependencies, cross-package behavior, change
impact, or conceptual retrieval in a linked worktree. For example: `$codegraph explain the
architecture and package dependencies in this monorepo`.

## Steps

1. Classify the task before measuring the repository. Use `rg` and `fd` directly for exact
   literals, regular expressions, known paths, and targeted source verification; stop without
   measuring or initializing either retrieval index.
2. For structural exploration in a Git repository, clear `GIT_DIR`, `GIT_WORK_TREE`,
   `GIT_INDEX_FILE`, `GIT_COMMON_DIR`, and `GIT_PREFIX`; obtain `--absolute-git-dir` and the
   absolute `--git-common-dir` from `git rev-parse`; canonicalize both existing directories. If
   either command or canonicalization fails, fall back to bounded `rg` and `fd` searches. Different
   canonical directories prove a linked worktree; equal directories prove a non-linked checkout.
3. In a linked worktree, run `colgrep-worktree '<conceptual query>'`. Treat its JSON output only as
   candidate locations and verify consequential findings in the source. If it refuses or fails,
   report the reason and fall back to bounded `rg` and `fd` searches. Stop this workflow here.
4. Outside a linked worktree, check whether `.codegraph/` exists.
5. When the index exists, prefix the CLI environment below and run `codegraph status --json`.
6. If status reports a non-null `worktreeMismatch`, do not query it. Measure this checkout and
   initialize here when the threshold is met, otherwise continue with `rg` and `fd`.
7. If status is healthy, use `codegraph_explore` before broad grep, find, or file reads.
8. If status reports stale or incomplete state, apply the cache-write check in step 10,
   synchronize once when allowed, and check status again.
9. When the index is absent, run `codegraph-repository-size .`.
10. Before `codegraph init` or `codegraph sync`, run
    `git check-ignore -q --no-index .codegraph/index.db`. When it succeeds, `.codegraph/` is ignored
    retrieval state: a read-only analysis still permits the cache write unless the user explicitly
    forbids local cache writes or all filesystem writes. Do not infer a cache-write prohibition from
    a request to analyze, review, or audit without editing the repository. When the check fails,
    name the missing exclusion and fall back without editing a project ignore file.
11. If `initialize` is true and the cache-write check permits it, prefix the CLI environment, run
    `codegraph init`, confirm status, then use `codegraph_explore`.
12. If `initialize` is false, continue with `rg` and `fd` without initializing.
13. If measurement, initialization, synchronization, or the second status check fails, name the
    failure and fall back explicitly to `rg` and `fd`.
14. Verify important retrieval claims in the source before editing.

Prefix every CodeGraph CLI call with:

```bash
CODEGRAPH_TELEMETRY=0 CODEGRAPH_NO_UPDATE_CHECK=1 CODEGRAPH_NO_DOWNLOAD=1
```

The MCP entry already supplies the same environment. An existing index remains usable below the
threshold. Initialization uses OR: 50000 source lines or 500 source files.

## Gotchas

- **Measuring an exact lookup** — unnecessary work can initialize an irrelevant index; classify
  the task first and keep literals, regular expressions, and known paths on `rg` or `fd`.
- **Using CodeGraph in a linked worktree** — even a healthy local index violates the checkout
  routing boundary; use only `colgrep-worktree` for conceptual retrieval there.
- **Calling ColGrep directly** — raw `colgrep` bypasses canonical-root and result-confinement
  checks; invoke only the worktree wrapper.
- **Trusting unchecked graph state** — a stale index can produce obsolete conclusions; run status
  before structural exploration and synchronize at most once when required.
- **Querying a sibling working tree** — an index resolved from a parent repository reports
  `state: complete` while describing another checkout; read `worktreeMismatch` before trusting
  freshness.
- **Treating an ignored index as a source edit** — broad analysis needlessly falls back to file
  scans; verify the global or local Git exclusion and allow the retrieval cache unless writes were
  explicitly forbidden.
- **Recovering destructively** — automatic uninitialization or reindexing can discard useful state;
  diagnose corruption, locks, or incompatibility and request approval before recovery.
- **Treating retrieval as refactoring or debugging** — graph results do not provide semantic edits
  or runtime state; use language servers for rename and code actions, and debuggers for execution.

## Constraints

- Never initialize below both thresholds unless `.codegraph/` already exists.
- Never call CodeGraph or initialize `.codegraph/` in a linked worktree.
- Never call raw `colgrep`; use `colgrep-worktree` only after linked-worktree proof.
- Never initialize ColGrep from a worktree creation hook or other eager lifecycle hook.
- Never edit a project `.gitignore` to initialize CodeGraph.
- Never write or synchronize `.codegraph/` unless Git confirms that the path is ignored.
- Never run `codegraph uninit --force`, `codegraph index`, or remove `.codegraph/` automatically.
- Never describe a failed or unchecked index as fresh.
- Use only the default `codegraph_explore` MCP surface; do not enable hidden tools or add a query
  wrapper.
- Keep CodeGraph retrieval-only and verify consequential claims against source files.
