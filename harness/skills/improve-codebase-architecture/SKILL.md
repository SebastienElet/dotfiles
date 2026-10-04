---
name: improve-codebase-architecture
description: >
  Survey architectural friction and present improvement candidates. Use only when the user
  explicitly invokes `$improve-codebase-architecture` or `/improve-codebase-architecture`; never
  select it implicitly for routine edits, local review or an implementation request.
license: MIT
disable-model-invocation: true
metadata:
  category: dev
---

# Improve Codebase Architecture

## Overview

Find concrete places where understanding or changing one behavior requires unnecessary coordination.
Present bounded candidates with before/after visuals and evidence, then explore the one the user
selects. This is an architectural survey and design conversation, not a refactoring authorization.

Adapted from [improve-codebase-architecture by Matt Pocock](https://github.com/mattpocock/skills/tree/d81f3a183412e71a5b1e84ca21bc1a35eea03a60/skills/engineering/improve-codebase-architecture).
The upstream license is preserved in [LICENSE](LICENSE).

## Usage

Invoke `$improve-codebase-architecture <scope>` in Codex or `/improve-codebase-architecture <scope>`
in Claude Code and Cursor. For example: `/improve-codebase-architecture Survey the invoice import
flow.` The invocation authorizes source inspection, a temporary visual report and exploration of
the selected candidate. Implementation, permanent documentation and publication require their own
established authorization.

## Steps

1. Locate the requested area and its consumers. Without a named area, inspect recent history and
   choose a bounded part that repeatedly changes; state that scope before scanning. Read the
   domain glossary and relevant ADRs before drawing conclusions about the architecture.
2. Consult `codebase-design` for depth, cohesion, locality and test boundaries. Retain canonical
   domain names and architectural constraints. Do not diagnose a useless module from one adapter,
   one caller or a line-count ratio alone.
3. Inspect the located flow and record concrete friction: scattered decisions, leaking internals,
   duplicated policy, excessive caller knowledge or a test boundary unable to exercise the real
   failure. Use a fresh reader for bounded synthesis when available; pass known source locations
   rather than delegating the search. Separate observed behavior from speculative improvement.
4. Select a small set of candidates with a demonstrated problem, proposed direction, invariant to
   preserve, expected gain and relevant validation gap. Do not propose a complete replacement
   interface yet. If a candidate conflicts with an ADR, mark the conflict and stop that implementation
   path; proposing investigation does not supersede the decision or authorize editing the ADR.
5. Read [references/report.md](references/report.md) and generate a self-contained HTML report in
   the operating-system temporary directory with a unique filename. Use before/after diagrams,
   source locations and a recommendation strength for each candidate. Open it with an available
   preview tool and give the absolute path. If preview is unavailable, retain the file and report
   that limitation; do not claim the rendering was checked.
6. Ask which candidate the user wants to explore. After the user selects one, invoke `grilling`
   for its constraints, alternatives, domain decisions and test boundaries. Report settled choices
   and remaining blockers without automatically changing the code, glossary or ADRs.
7. Hand off the selected outcome to the user's requested next workflow. `code-simplify` owns an
   authorized behavior-preserving cleanup; `to-spec` remains a separate manual invocation for
   specification synthesis. Choosing a candidate alone does not authorize either implementation.

## Gotchas

- **Scanning the whole repository by default** — unrelated debt overwhelms the useful finding;
  bound the area using the request or actual change history before reading deeply.
- **Equating depth with merging files** — domain or transaction boundaries disappear; state the
  responsibility and invariant before proposing consolidation.
- **Turning the report into a repair pass** — an advisory survey changes behavior; finish the
  visual candidates and wait for a separately authorized implementation scope.
- **Writing domain documentation during the interview** — proposals become durable authority;
  return the decision log and save it only under the user's established write authorization.

## Constraints

- Activate only on explicit `$improve-codebase-architecture` or `/improve-codebase-architecture` invocation.
- Preserve domain vocabulary and every architectural decision in force.
- Never turn a design heuristic or speculative candidate into a certain defect without evidence.
- Never implement a candidate or rewrite permanent documentation from survey authority alone.
- Invoke `grilling` only after the user selects a candidate from the requested survey.
