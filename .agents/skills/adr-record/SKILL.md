---
name: adr-record
description: >
  Record one architecture decision as an ADR in docs/adr/ and keep its index current. Use only when
  the user explicitly invokes `$adr-record` or `/adr-record`; never select it implicitly from
  requests to decide, document, or review architecture.
disable-model-invocation: true
metadata:
  category: dev
---

# ADR Record

## Overview

Record a structural decision of this repository as one ADR under `docs/adr/`, numbered, named and
formatted like the ADRs already there, and update `docs/adr/README.md` in the same change. The skill
writes to the repository, so it runs only on explicit invocation. The existing ADRs, their index and
the instruction files are the authority; this skill only derives from them. Every recorded fact
comes from the user or a verifiable source; anything missing is asked for or recorded as unknown.

## Usage

```text
/adr-record <decision to record>
$adr-record <decision to record>
```

Example: `/adr-record Replace npm with Bun for global packages`.

The skill may stop without writing when no ADR is warranted. It never commits or pushes; the user
decides when the change lands.

## Steps

1. **Read the authority.** Read the architecture decision section of the root `AGENTS.md`,
   `docs/AGENTS.md`, and `docs/adr/README.md` completely: scope, numbering, language, reliability
   of motivations and index. Where they disagree with this skill, follow them and report the drift.
2. **Gate the request.** An ADR records a structural decision still in force. Stop with a short
   explanation, writing nothing, when:
   - the change is routine in the sense of the root `AGENTS.md`, whose examples are a tool target,
     a lockfile update and a skill edit;
   - the decision is a proposal, an experiment, or not yet taken.

   A withdrawal of a tool or practice gets no new ADR, unless its justification is measured and
   reusable: the exception `docs/adr/README.md` states, with ADR-033 and ADR-034 as precedents.
   If an ADR records the withdrawn decision, continue in order to retire it; otherwise stop. When
   the case is borderline, ask the user one question before going further.

3. **Map related ADRs.** Search `docs/adr/` for the decision's subject: exact terms with `rg`, the
   subject as a concept through the `code-search` skill. Classify each related ADR as:
   - _replaced_: same subject, its decision no longer in force as a whole;
   - _amended_: part of its decision no longer in force;
   - _completed_: still in force as written; the new ADR states that it completes it without
     replacing it, as ADR-039 does;
   - _unrelated_.

   Before writing, tell the user about every replaced or amended ADR and quote the contradicted
   passage: the root `AGENTS.md` forbids contradicting an ADR silently. When replaced versus
   amended is unclear from the request, ask.

4. **Assign the number and filename.**
   - Apply the numbering section of `docs/adr/README.md`. Retired numbers are absent from the
     directory listing, so compute the highest number ever assigned from history and the working
     tree:

     ```bash
     (git log --format= --name-only HEAD main -- docs/adr/; ls docs/adr/*.md) |
       sed -n 's#^docs/adr/\([0-9]\{3\}\)-.*#\1#p' | sort -u | tail -n 1
     ```

   - Name the file like its neighbors: the three-digit number, a hyphen, then a short lowercase
     ASCII slug of the French title, without accents.

5. **Gather the facts.** Derive the field set from the three most recent ADRs and the related ones:
   the H1 pattern, the metadata lines, the section headings and their order.
   - Take every value from the user, the code, Git history (`git log`, `git show`) or the forge
     (`gh issue view`, `gh pr view`).
   - Verify every commit hash with `git show -s` and every issue or pull request before citing it.
     Omit an optional metadata line rather than guess its value; ADR-040 and ADR-043 carry no
     commit line.
   - Quote a commit body explicitly when the context relies on it: the index states that an
     unquoted context is an after-the-fact reconstruction.
   - List as rejected alternatives only options the user or a source actually considered. For a
     replacement, the replaced decision is one of them, as the index's scope section requires.
   - Ask the user once, grouping every missing context, consequence or alternative. Whatever stays
     unknown after the answer is written as unknown in its field, naming what is missing.

6. **Write the change.**
   - Write the ADR in French and keep identifiers, commands, paths and quoted commit bodies
     verbatim, as `docs/AGENTS.md` requires. Copy the H1 pattern, metadata labels, status value and
     section headings verbatim from existing ADRs.
   - _Replaced ADR_: once the user confirms, remove its file, write the successor under the same
     number, and replace its index row. Ask which marking precedent to follow (see Gotchas).
   - _Amended ADR_: once the user confirms, revise it in place, removing or rewriting only the
     passage no longer in force, and add or update its revision date line as ADR-001 to ADR-004 do.
   - _Retired ADR without successor_: once the user confirms, remove its file and its index row,
     and leave its number unused.
   - Add the new index row in numeric order, its title identical to the ADR's H1 title and its date
     identical to the ADR's date line.
   - When the decision asserts a guarantee such as authority, completeness, atomicity or
     validation, the `design-claim-audit` skill governs that claim; do not certify it here.
7. **Verify and report.**
   - Format the changed Markdown with `bun run prettier --write <paths>`.
   - Check that `git status --short` lists only files under `docs/adr/`, that the number appears
     once in the directory, that the index link resolves, and that every cited commit and ADR
     exists.
   - Report the number and path, each related ADR with its classification, the facts recorded as
     unknown, and every inconsistency between existing ADRs met on the way.

## Gotchas

- **Filling a gap in the numbering** — the listing shows 020, 030 and 032 free, but they belong to
  retired decisions on other subjects; reusing one merges two subjects under one number in Git
  history. Apply the numbering section of `docs/adr/README.md` to the number computed in step 4.
- **Keeping a replaced ADR with a superseded status** — the index records only decisions in force
  and every ADR carries the same status value, so a second value would be a new convention. Remove
  the replaced file and record its decision among the successor's rejected alternatives.
- **Choosing a marking silently** — precedents disagree. ADR-039 carries a line naming the decision
  it replaces, ADR-037 carries none, and ADR-001 and ADR-003 were rewritten in place under their
  original filename with a revision line; ADR-028 was revised by `50c4c90` without one. Report the
  precedents that apply and let the user choose.
- **Writing a plausible motivation** — later agents read an ADR as authority, so an invented reason
  becomes a false constraint. Quote sources, ask the user, and record the rest as unknown.
- **Normalizing neighboring ADRs** — differing titles, date lines or typography between ADRs and
  the index are tempting to align, but aligning them widens the change and settles a convention the
  user has not chosen. Report them and leave them unchanged.

## Constraints

- Write only under `docs/adr/`: the new ADR, the index, and any replaced, amended or retired ADR
  the user confirmed.
- Never invent context, commits, issues, dates, consequences or alternatives; ask the user or record
  the fact as unknown.
- Never write an ADR for a routine change or for a decision that is not in force.
- Never delete or rewrite an existing ADR without the user's explicit confirmation.
- Never resolve an inconsistency between existing ADRs; report it.
- Never commit or push unless the user asks.
