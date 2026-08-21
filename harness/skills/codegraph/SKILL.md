---
name: codegraph
description: >
  Explore large repositories structurally with CodeGraph. Use when auditing an unfamiliar
  repository, or when locating architecture, call paths, dependencies, cross-package behavior,
  change impact, or validating an existing .codegraph index. Make sure to use this skill before any
  broad exploration, including an audit or a technical-debt review, even if CodeGraph was not
  requested.
metadata:
  category: dev
---

# CodeGraph

## Overview

Use CodeGraph for open-ended structural retrieval when repository size makes file-by-file
exploration inefficient. Keep exact searches on `rg` and `fd`, and verify important graph claims in
the source before editing.

## Usage

Invoke this skill for architecture, call paths, dependencies, cross-package behavior, or change
impact. For example: `$codegraph explain the architecture and package dependencies in this
monorepo`.

## Steps

1. Classify the task before measuring the repository. Use `rg` and `fd` directly for exact
   literals, regular expressions, known paths, and targeted source verification; stop without
   measuring or initializing CodeGraph.
2. For structural exploration, check whether `.codegraph/` exists.
3. When the index exists, prefix the CLI environment below and run `codegraph status --json`.
4. If status reports a non-null `worktreeMismatch`, the index belongs to another working tree: do
   not query it. Measure this worktree and initialize here when the threshold is met, otherwise
   continue with `rg` and `fd`.
5. If status is healthy, use `codegraph_explore` before broad grep, find, or file reads.
6. If status reports stale or incomplete state, apply the cache-write check in step 8, synchronize
   once when allowed, and check status again.
7. When the index is absent, run `codegraph-repository-size .`.
8. Before `codegraph init` or `codegraph sync`, run
   `git check-ignore -q --no-index .codegraph/index.db`. When it succeeds, `.codegraph/` is ignored
   retrieval state: a read-only analysis still permits the cache write unless the user explicitly
   forbids local cache writes or all filesystem writes. Do not infer a cache-write prohibition from
   a request to analyze, review, or audit without editing the repository. When the check fails,
   name the missing exclusion and fall back without editing a project ignore file.
9. If `initialize` is true and the cache-write check permits it, prefix the CLI environment, run
   `codegraph init`, confirm status, then use `codegraph_explore`.
10. If `initialize` is false, continue with `rg` and `fd` without initializing.
11. If measurement, initialization, synchronization, or the second status check fails, name the
    failure and fall back explicitly to `rg` and `fd`.
12. Verify important graph claims in the source before editing.

Prefix every CodeGraph CLI call with:

```bash
CODEGRAPH_TELEMETRY=0 CODEGRAPH_NO_UPDATE_CHECK=1 CODEGRAPH_NO_DOWNLOAD=1
```

The MCP entry already supplies the same environment. An existing index remains usable below the
threshold. Initialization uses OR: 50000 source lines or 500 source files.

## Gotchas

- **Measuring an exact lookup** — unnecessary work can initialize an irrelevant index; classify
  the task first and keep literals, regular expressions, and known paths on `rg` or `fd`.
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
- Never edit a project `.gitignore` to initialize CodeGraph.
- Never write or synchronize `.codegraph/` unless Git confirms that the path is ignored.
- Never run `codegraph uninit --force`, `codegraph index`, or remove `.codegraph/` automatically.
- Never describe a failed or unchecked index as fresh.
- Use only the default `codegraph_explore` MCP surface; do not enable hidden tools or add a query
  wrapper.
- Keep CodeGraph retrieval-only and verify consequential claims against source files.
