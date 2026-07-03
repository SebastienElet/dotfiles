# Cross-Check Skills

## Scope

- `/skill-manager cross-check` — analyse all skills in `.agents/skills/` to detect inter-skill
  inconsistencies

This operation is **read-only**: it never modifies any file. It produces a report, presents findings
to the user, and stops. No fix is applied without explicit user instruction.

---

## Procedure

1. **Collect all skills**

   ```bash
   ls .agents/skills/
   ```

   Exclude `README.md`. For each skill directory found, read its `SKILL.md`.

2. **Extract structured data** from each skill:

   - `name` (from frontmatter)
   - `description` (from frontmatter)
   - Trigger keywords (extracted from description: phrases after "Use when", "Make sure to use this
     skill whenever", "Make sure to use whenever", "even if")
   - `## Constraints` section (list of rules)
   - Cross-references (mentions of other skills in `## Steps`, `## References`, or `## Overview`)
   - Inferred functional domain (git, PR, tests, infra, support, etc.)
   - `references/` directory listing — for each file, parse its name as `<topic>-<scope>.md` if it
     matches that pattern; **do not read the content yet** (lazy load: content is only read during
     detector passes that need it — D2 pass 2 for pairs ≥ 30%, D4 for `## Constraints` in reference
     files, D6 for classified scoped files)

3. **Run all 6 detectors** (see below).

4. **Produce the report** in the defined format.

5. **Present the report to the user and stop** — do not modify anything without explicit
   instruction. If the user asks you to apply a fix inline, refuse: "Cross-check is read-only. Run
   `/skill-manager fix <name>` to apply this change."

---

## Detectors

### D1 — Trigger Overlap

**Goal**: Detect two skills whose descriptions share nearly identical trigger keywords, which
confuses the agent about which skill to activate.

**Method**:

- Extract significant tokens from each `description` (ignore stop words: "use", "when", "make",
  "sure", "this", "skill", "even", "if", "the", "a", "an", "for", "with", "to", "and", "or", "in",
  "on", "is", "are")
- Compute pairwise overlap: `|tokens_A ∩ tokens_B| / |tokens_A ∪ tokens_B|`
- Threshold: ≥ 40% overlap → 🟡 WARN; ≥ 65% → 🔴 CRITICAL

> ⚠️ **Performance note**: D1 is O(N²) on the number of skills. With N=20 skills this means ~190
> pairs — manageable. Above 50 skills (~1225 pairs), consider whether the context window can
> accommodate all comparisons before starting.

**Output format**:

```text
[D1] Trigger Overlap: <skill-A> ↔ <skill-B> — <X>% overlap
  Shared tokens: <list>
  Risk: agent may activate <skill-A> when <skill-B> is intended
```

> ⚠️ D1 is heuristic — token overlap is an indicator, not a proof. See
> [Known Limitations](#known-limitations) before acting on a D1 finding.

---

### D2 — Content Duplication

**Goal**: Detect the same procedure or rule described in two different skills, which inevitably
causes divergence over time.

**Method**:

- **Two-pass strategy** to bound context usage:
  1. First, compare only `## Steps` and `## Constraints` sections in `SKILL.md` files (N² but
     small).
  2. For pairs reaching ≥ 30% similarity in pass 1, read their `references/` files and compare those
     as well.
- Flag when ≥ 2 consecutive steps are structurally similar (same action verbs, same tools mentioned,
  same sequence).

> ⚠️ **Performance note**: Reading all `references/` files upfront for D2 is O(N²) and may overflow
> context on large skill sets. The two-pass approach above avoids unnecessary reads.

**Output format**:

```text
[D2] Content Duplication: <skill-A>[/<file>] § Steps / <skill-B>[/<file>] § Steps
  Similar steps: "<excerpt>"
  Recommendation: extract to a shared reference or a dedicated skill
```

---

### D3 — Dead Reference

**Goal**: Detect a skill that mentions another skill (by slug) that does not exist in
`.agents/skills/`.

**Method**:

- In each SKILL.md, search for short slug-style patterns only: `skill-<slug>`, `[<text>](<slug>)`
  where `<slug>` contains no `/`, `See <slug>` where slug contains no `/`, slugs in backticks
  without a `/`
- **Exclude from checking**: `/skill-manager <cmd>` patterns (these are subcommand invocations, not
  skill references), any path containing `/` (absolute or relative paths like `.cursor/rules/...`,
  `services/api-graphql/...`, `AGENTS.md`), any URL starting with `http`
- Verify each extracted slug has a matching directory under `.agents/skills/`

**Output format**:

```text
[D3] Dead Reference: <skill-A> mentions "<slug>" — no skill found with this name
  Line: <number or excerpt>
  Recommendation: fix the name or create the missing skill
```

---

### D4 — Rule Contradiction

**Goal**: Detect two skills whose `## Constraints` contain opposing rules on the same subject.

**Method**:

- Extract each line from `## Constraints` (or `## Rules`) of every skill — both in SKILL.md and in
  every file under `references/`
- Search for antagonistic pairs: "always X" vs "never X", "prefer X" vs "avoid X", on the same
  subject `X`
- The subject `X` is identified by the main nouns and verbs in the rule

**Output format**:

```text
[D4] Rule Contradiction: <skill-A>[/<file>] vs <skill-B>[/<file>]
  Rule A: "<full text>"
  Rule B: "<full text>"
  Conflicting subject: <X>
  Recommendation: align rules or clarify the context of application for each
```

---

### D5 — Slug Ambiguity

**Goal**: Detect two skills whose names are close enough to cause confusion (typo, plural, name
variation).

**Method**:

- Compare all slugs pairwise
- Flag if: same root with different suffix (`git-commit` / `git-commits`), or Levenshtein distance ≤
  2, or same domain + similar verb (`pr-create` / `pr-open`)

**Output format**:

```text
[D5] Slug Ambiguity: "<slug-A>" ↔ "<slug-B>"
  Distance: <Levenshtein or description>
  Recommendation: rename one to clarify the distinction
```

---

### D6 — Scoped Reference Conflict

**Goal**: Detect inconsistencies between `references/<topic>-<scope>.md` files — either
contradictions between scopes that should diverge intentionally, or duplications between scopes that
should have been shared.

**Context**: The `conventions.md` standard requires splitting `references/` files by scope when
behaviour differs per package (e.g. `integration-pm.md` vs `integration-lm.md`). This detector
validates that the split is correct and that no unintended divergence has crept in.

**Method**:

Step 1 — **Inventory scoped files** across all skills:

- For every `references/` file whose name matches `<topic>-<scope>.md`, classify it as a **scoped
  file** only when **at least one sibling file in the same `references/` directory shares the same
  `<topic>` with a different `<scope>` suffix** (e.g. `integration-pm.md` and `integration-lm.md`
  both present).
- A file that has no same-topic sibling is treated as a **plain reference file** and skipped by D6
  entirely — regardless of how its name looks (e.g. `cross-check.md`, `sync-index.md`,
  `react-testing.md` all fail this test and are ignored).
- Known scope tokens for reference: `pm`, `lm`, `api-graphql`, `lobby`, `common`, `prisma-pm`,
  `prisma-lm`, `frontend`, `backend`. These are examples — the sibling test is the authoritative
  gate, not this list.
- Record `(skill, scope, topic, path)` for each classified scoped file.
- Group by `topic` across all skills.

Step 2 — **Same topic, same skill, multiple scopes** (intra-skill split check):

- For each `(skill, topic)` group with ≥ 2 scopes, compare their content
- If ≥ 80% of content is identical → 🟡 WARN: the split may be unnecessary; content could be merged
  into a shared file
- If content differs structurally → 🔵 INFO: intentional split, no action needed

Step 3 — **Same topic, different skills** (inter-skill duplication check):

- For each `topic` found in ≥ 2 different skills, compare the files
- If content is substantially similar (≥ 60%) → 🟡 WARN: consider extracting to a shared reference
  or a dedicated skill

Step 4 — **Routing coverage check**:

- For each skill with `<topic>-<scope>.md` files, verify that `## Steps` in SKILL.md contains
  conditional routing ("if … read `references/<topic>-<scope>.md`")
- Missing routing → 🔴 CRITICAL: agent will not know which scoped file to load

**Output format**:

```text
[D6] Scoped Reference Conflict: <skill>/references/<topic>-<scope-A>.md ↔ <skill>/references/<topic>-<scope-B>.md
  Topic: <topic> | Similarity: <X>%
  Recommendation: merge into a shared file or confirm intentional divergence

[D6] Scoped Reference Duplication: <skill-A>/references/<topic>-<scope>.md ↔ <skill-B>/references/<topic>-<scope>.md
  Topic: <topic> | Similarity: <X>%
  Recommendation: extract to a shared skill or reference

[D6] Missing Routing: <skill> has <topic>-<scope>.md files but no conditional routing in ## Steps
  Files: <list>
  Recommendation: add "if … read references/<topic>-<scope>.md" logic to ## Steps
```

---

## Output Format

```text
# Cross-Check Report — .agents/skills/

Analysed: <N> skills | Detectors: D1 D2 D3 D4 D5 D6
Date: <date>

---

## 🔴 Critical Issues (blocking)

### [D1] Trigger Overlap — <skill-A> ↔ <skill-B>
...

### [D3] Dead Reference — <skill-A>
...

### [D6] Missing Routing — <skill-A>
...

---

## 🟡 Warnings

### [D1] Trigger Overlap — <skill-A> ↔ <skill-B>  *(40–64% overlap)*
...

### [D2] Content Duplication — <skill-A> / <skill-B>
...

### [D4] Rule Contradiction — <skill-A> vs <skill-B>
...

### [D6] Scoped Reference Conflict — <skill>
...

### [D6] Scoped Reference Duplication — <skill-A> / <skill-B>
...

---

## 🔵 Info (non-blocking)

### [D5] Slug Ambiguity — <slug-A> ↔ <slug-B>
...

---

## Summary

| # | Detector | Severity | Skills | Suggested Action |
|---|----------|----------|--------|------------------|
| 1 | D1 Trigger Overlap | 🔴 ≥ 65% | skill-A, skill-B | Differentiate descriptions |
| 2 | D1 Trigger Overlap | 🟡 40–64% | skill-A, skill-B | Review and differentiate if needed |
| 3 | D3 Dead Reference | 🔴 | skill-A | Fix link to "slug" |
| 4 | D2 Content Duplication | 🟡 | skill-A, skill-B | Extract to shared reference |
| 5 | D4 Rule Contradiction | 🟡 | skill-A, skill-B | Align rules or clarify context of application |
| 6 | D6 Missing Routing | 🔴 | skill-A | Add conditional routing in ## Steps for `<topic>-<scope>.md` files |
| 7 | D6 Scoped Duplication | 🟡 | skill-A, skill-B | Extract to shared reference |
| 8 | D5 Slug Ambiguity | 🔵 | slug-A, slug-B | Rename slug-B |

**No files were modified.**
Please indicate which inconsistencies you want to fix and how to proceed.
```

---

## Severity Levels

| Level       | Meaning                                          | Required action           |
| ----------- | ------------------------------------------------ | ------------------------- |
| 🔴 CRITICAL | Risk of incorrect activation or broken reference | Fix before next doctor     |
| 🟡 WARN     | Risk of future divergence or confusion           | Address in current sprint |
| 🔵 INFO     | Cosmetic, no functional impact                   | Address if time permits   |

---

## Constraints

- **Strict scope**: never read any file outside `.agents/skills/`, even to validate a dead reference
  (D3), confirm a rule (D4), or verify a routing pattern (D6).
- If an out-of-scope check seems necessary, **complete the full analysis first**, then ask the user
  for confirmation at the end of the report, explaining why the extra read would be needed.
- **This operation is read-only** — if the user asks you to apply a fix inline during the
  cross-check, refuse: "Cross-check is read-only. Run `/skill-manager fix <name>` to apply this
  change."

---

## Known Limitations

- **D1 is heuristic** — token overlap is an indicator, not a proof. Two skills may legitimately
  share keywords if their domains are complementary (e.g. `git-commit` and `pr-create` both mention
  "git").
- **D2 does not detect semantics** — two procedures expressed differently but doing the same thing
  may go undetected.
- **D3 only checks slug-style references** — it does not validate absolute paths (`.cursor/rules/`,
  `services/api-graphql/...`, `AGENTS.md`, etc.) or URLs. References to files outside
  `.agents/skills/` are intentionally out of scope and will not be flagged.
- **D4 requires human judgment** — some contradictions are intentional (e.g. "always use Jest" in a
  backend skill, "never use Jest" in a frontend skill).
- **D6 similarity thresholds are approximate** — 80% / 60% are heuristic; a 79% similar pair may
  still need merging. Always read the actual files before deciding.
- **D6 Step 2 intentional splits** — if a skill deliberately splits `pm` and `lm` references because
  the rules genuinely differ, D6 will report 🔵 Info (not a warning). The detector does not flag
  intentional divergence as an error.
- **D6 false positives on plain two-word filenames** — files like `cross-check.md` or
  `sync-index.md` look like `<topic>-<scope>.md` but their second word is not a scope token. D6 must
  not classify them as scoped files (see Step 1 criteria). When in doubt, require the sibling-file
  evidence: if no other file in the same directory shares the same topic with a recognized scope
  suffix, the file is not scoped.
- **Cross-check does not replace doctor** — it complements per-skill doctor checks. Run `doctor` first, then
  `cross-check`.
- **Non-conforming skills amplify D1 and D4 noise** — if a skill has a malformed description
  (missing "Use when" / "Make sure to use whenever" pattern), D1 token extraction produces garbage
  tokens, which inflates overlap scores artificially. Similarly, if `## Constraints` is absent from
  a skill, D4 cannot detect real contradictions for that skill. Fix individual skills with `doctor` +
  `fix` before running `cross-check` to reduce false positives.
