---
name: issue-simplify
description: >
  Simplify GitHub or Linear issues and drafts. Use when explicitly asked to simplify, lighten, or
  declutter issue content or scope. Make sure to use this skill whenever that request is expressed
  without naming the skill. Excludes status checks, tracker synchronization, implementation, and
  creation without simplification.
metadata:
  category: product
---

# Issue Simplification

## Overview

Make an issue easier to understand, decide, and implement by clarifying its outcome and removing
unnecessary specification complexity. This v0 reads sources and returns a proposal only; a shorter
issue is useful only if its requirements and evidence remain intact.

## Usage

`/issue-simplify <issue URL, identifier with project context, or draft>`

Example: "Simplify this invoice-export issue; keep the accepted CSV compatibility requirement."
An explicit natural-language simplification request also activates the skill. Write the proposal
in the user's language and retain the project's vocabulary and issue conventions.

## Steps

1. **Resolve and read the target.** Identify the exact repository and issue, Linear team and issue,
   or supplied draft. Ask for the missing identity if multiple targets fit. Use available read-only
   connector or CLI operations to retrieve the complete available body, relevant comment decisions,
   and necessary linked sources; follow pagination when present. Inspect neighboring issues only
   when overlap, dependencies, or conflicting scope can change the interpretation. Report failed,
   truncated, or inaccessible reads and their impact; never invent their contents. Continue with
   the available draft, marking material gaps as unresolved.
2. **Compose existing responsibilities.** For a Linear product issue, use `linear-issue-spec` for
   product evidence, functional slicing, visible states, and its existing draft structure. Keep a
   coherent functional increment; proposals to split or combine work need recognizable standalone
   value or a concrete coordination benefit. `issue-creation` owns lifecycle coherence and new
   publication; compose its draft/review checks when applicable, without entering publication.
   Before restructuring Linear criteria or evidence, read `linear-workflow`'s
   `references/completion-evidence.md` for evidence semantics only, not its mutation procedures.
3. **Separate facts from suggestions.** Identify the problem, promised outcome, established
   constraints, proposals, suggested technical means, and open questions. Preserve the source and
   decision status of important requirements. For Linear, retain `Requested`, `Established`, and
   `Proposed` labels from `linear-issue-spec`; existing issue prose alone does not establish a
   decision. Surface material contradictions and the smallest decision needed before the draft;
   do not finalize the affected scope while that decision remains open.
4. **Simplify at unchanged scope.** Remove repetition, session narration, speculative solutions,
   premature implementation sequences, and options with no established need. Regroup related
   statements while retaining expected behavior, useful data, relevant states and errors,
   permissions, compatibility constraints, acceptance criteria, and real dependencies. Keep a
   justified technical constraint with its source; remove a suggested means only when no
   established requirement depends on it. Do not prescribe files, architecture, or development
   order when these are not established constraints.
5. **Keep scope decisions separate.** Put reductions, additions, splits, combinations, and removal
   of allegedly obsolete requirements in a distinct proposal with rationale, impact, and required
   decision. Do not apply them to the draft presented as equivalent. Reducing coordination by
   regrouping prose is editorial; moving deliverables between issues changes scope.
6. **Preserve proof and uncertainty.** Retain evidence links, environments, and recorded checked
   or unchecked status without claiming fresh verification. Keep an unverified criterion visible
   and unverified. Do not soften it, delete it, check it, move residue elsewhere, or replace it with
   a completion summary. Keep criteria with different evidence statuses separately identifiable;
   report unavailable evidence without treating its absence as disproof.
7. **Present and verify the proposal.** Provide material gaps or contradictions, the proposed
   title/body at preserved scope, and a short semantic diff naming what is kept, removed, grouped,
   and awaiting decision. Respect the Linear specialist's structure; impose no universal issue
   template. Trace each promised outcome to a relevant observable acceptance criterion, and check
   that no original requirement disappeared. If a criterion is missing, expose the gap and label
   any suggested criterion as proposed rather than silently asserting a new requirement.
8. **State the write boundary.** End by explicitly stating that no remote write occurred. If asked
   to update, create, close, or merge remotely, say this v0 does not execute that action and return
   the proposal. Existing-issue editing is not an available `issue-creation` mode; leave that
   lifecycle evolution to a separate iteration rather than adding a second publication mechanism.

## Gotchas

- **Treating every technical detail as clutter** — removing an accepted compatibility constraint
  changes the contract; retain its provenance and separate it from speculative implementation.
- **Letting Linear slicing rewrite scope silently** — useful regrouping becomes an unapproved
  transfer of deliverables; keep the equivalent draft intact and present slicing for decision.
- **Grouping checked and unchecked criteria** — the combined line implies evidence that does not
  exist; preserve distinct statuses and proof references.
- **Following a sibling skill into publication** — composition escapes this v0's boundary; use
  only draft/review responsibilities and end with the no-write statement.

## Constraints

- Never create, edit, comment on, close, merge, or otherwise mutate remote issues in this v0,
  even when the user requests a remote update as part of simplification.
- Never present a scope change, unresolved contradiction, or inferred requirement as an
  equivalent editorial simplification or an established decision.
- Never remove or weaken unfinished acceptance criteria to imply completion.
- Never expand targeted neighboring-issue reads into a general backlog audit.
- Never add a tracker client, external activation router, or publication mechanism here.

## References

- [evals/scenarios.md](evals/scenarios.md) — offline fixtures and behavioral acceptance checks;
  these are scenarios, not evidence of executed tracker integrations.
