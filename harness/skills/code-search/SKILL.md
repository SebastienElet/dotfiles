---
name: code-search
description: >
  Search codebases with exact and conceptual retrieval. Use when auditing an unfamiliar repository,
  or when locating architecture, call paths, dependencies, cross-package behavior, or change
  impact. Make sure to use this skill before any broad exploration, including an audit or a
  technical-debt review, even if no search tool was requested.
metadata:
  category: dev
---

# Code Search

## Overview

Use `rg` and `fd` for exact retrieval and guarded ColGrep for conceptual retrieval in any canonical
Git checkout. Verify important retrieval claims in the source before editing.

## Usage

Invoke this skill for architecture, call paths, dependencies, cross-package behavior, change
impact, or conceptual retrieval in a Git checkout. For example: `$code-search explain the
architecture and package dependencies in this monorepo`.

## Steps

1. Classify the task before measuring the repository. Use `rg` and `fd` directly for exact
   literals, regular expressions, known paths, and targeted source verification.
2. For conceptual retrieval, run `colgrep-search '<conceptual query>'` from the target Git checkout.
3. Treat its JSON output only as candidate locations and verify consequential findings in source.
4. If the guarded command refuses or fails, report the reason and fall back to bounded `rg` and
   `fd` searches.

## Gotchas

- **Using conceptual search for an exact lookup** — keep literals, regular expressions, and known
  paths on `rg` or `fd`.
- **Calling ColGrep directly** — raw `colgrep` bypasses canonical-root and result-confinement
  checks; invoke only `colgrep-search`.
- **Trusting candidate locations** — retrieval can be incomplete or imprecise; verify consequential
  findings against source files.
- **Treating retrieval as refactoring or debugging** — graph results do not provide semantic edits
  or runtime state; use language servers for rename and code actions, and debuggers for execution.

## Constraints

- Never call raw `colgrep`; use `colgrep-search` so root and results remain confined.
- Never initialize ColGrep from a worktree creation hook or another eager lifecycle hook.
- Never describe a failed or unchecked search as authoritative.
- Keep retrieval read-oriented and verify consequential claims against source files.
