# Cross-Check Skills

## Scope

- `/skill-manager cross-check` — analyse all skills in the selected `<skills-root>/` to detect inter-skill
  inconsistencies

This operation is **read-only**: it never modifies any file. It produces a report, presents findings
to the user, and stops. No fix is applied without explicit user instruction.

---

## Procedure

1. **Collect all skills**

   ```bash
   ls <skills-root>/
   ```

   Exclude `README.md`. For each skill directory found, read its `SKILL.md`.

   Also list the immediate skill directory slugs in the other canonical collection, counting only
   directories containing `SKILL.md`. Reuse this slug-only inventory for D3 and D5; do not read
   those skills' contents or merge the collections. If the other source is unavailable, record
   that limitation instead of treating an unverified slug as absent.

2. **Extract structured data** from each skill:

   - `name` (from frontmatter)
   - `description` (from frontmatter)
   - Trigger keywords (extracted from description: phrases after "Use when", "Make sure to use this
     skill whenever", "Make sure to use whenever", "even if")
   - `## Constraints` section (list of rules)
   - Cross-references (mentions of other skills in `## Steps`, `## References`, `## Overview`, or
     `## Constraints`)
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

### D3 — Reference Resolution

**Goal**: Distinguish a missing skill from a reference to the other canonical collection and from
a skill available only in the dotfiles checkout. Source existence does not prove availability in
the project where a user skill is being used.

**Method**:

- In each SKILL.md, search for short slug-style patterns only: `skill-<slug>`, `[<text>](<slug>)`
  where `<slug>` contains no `/`, `See <slug>` where slug contains no `/`, slugs in backticks
  without a `/`
- Include `## Constraints` references in the structured inventory used by D3; do not discard them
  after extraction or depend on a later scan of the complete file to recover them.
- **Exclude from checking**: `/skill-manager <cmd>` patterns (these are subcommand invocations, not
  skill references), any path containing `/` (absolute or relative paths like `.cursor/rules/...`,
  `services/api/...`, `AGENTS.md`), any URL starting with `http`, and externally qualified
  identifiers containing a namespace separator such as `superpowers:requesting-code-review`
- Resolve each extracted slug against the selected inventory, then the other collection's slugs.
  A slug present in both collections belongs to D5; never choose or merge duplicate copies.
- Report **Dead Reference** (CRITICAL) only when the slug is absent from both verified inventories.
  If no match is found and an inventory cannot be checked, report **Unverified Reference** (INFO)
  and name the missing lookup; do not recommend creating a skill on that evidence. A known match
  remains resolved even when the other inventory is unavailable.
- A slug found only in the other collection is **Cross-Scope Reference** (INFO), not dead. Name
  its canonical scope and source. Never recommend copying it into the selected collection.
- Assess availability separately using the current project's discovered skill inventory or an
  explicit availability statement. A project skill in dotfiles is not implicitly installed for a
  user skill used elsewhere. When it exists only in that checkout, report **Checkout-Only
  Reference** (INFO); if current availability is unknown, say so rather than assuming it.
- Before diagnosing a dependency, verify that the referring instruction actually invokes,
  delegates to, or requires following the referenced skill. A comparison, ownership note, or
  illustrative mention still resolves by slug, but is not a dependency and needs no availability
  guard or dependency warning. If actual use is unclear, report that uncertainty rather than
  infer a dependency from the extracted name alone.
- For an actual dependency known to be unavailable to the current agent or project, report
  **Unavailable Skill Dependency** (WARN) unless the instruction explicitly handles that
  unavailability. This applies to matches in the selected collection as well as either cross-scope
  direction, including project-to-user references.
  Recommend satisfying the declared prerequisite in its proper scope, or an authorized condition
  and fallback. Never copy the skill into another collection or make mandatory validation optional;
  a workflow with an unmet mandatory prerequisite must stop. Unknown availability alone is INFO,
  not evidence of unavailability; the user-to-project conditionality check below still applies.
  Combine this with an unconditional-project-dependency warning in one finding when both apply.
- For a user skill that depends on a project skill, verify an explicit condition covering both the
  referenced skill's availability and its applicability under the current project's conventions.
  Without that condition, add **Unconditional Project Dependency** (WARN), even while auditing
  from dotfiles where the referenced skill is available. Recommend making the dependency
  conditional and following the current project's conventions when it is unavailable or
  inapplicable. An explicit condition and fallback need no dependency warning outside dotfiles.

**Output format**:

```text
[D3] Dead Reference: <skill-A> mentions "<slug>" — absent from both canonical collections
  Line: <number or excerpt>
  Recommendation: fix the name or, if required, create the skill in its appropriate scope

[D3] Cross-Scope Reference: <skill-A> mentions "<slug>" — found in <scope>: <source>
  Availability: available in the current project | checkout-only | unknown
  Dependency: none | unknown | conditional | unconditional project dependency (WARN)
  Recommendation: <conditional dependency correction, if needed; otherwise none>

[D3] Checkout-Only Reference: <skill-A> mentions "<slug>" — exists in dotfiles project scope only
  Current project: <project where the user skill is used>; skill unavailable here
  Dependency: none | unknown | conditional with project-convention fallback | unconditional project dependency (WARN)
  Recommendation: <conditional dependency correction, if needed; otherwise none>

[D3] Unverified Reference: <skill-A> mentions "<slug>" — lookup unavailable for <collection>
  Recommendation: verify that inventory before concluding the skill is missing

[D3] Unavailable Skill Dependency: <skill-A> requires "<slug>" — source exists in <scope>
  Availability: known unavailable to <current agent or project>
  Severity: WARN; combine with unconditional project dependency if applicable
  Recommendation: satisfy the prerequisite in its proper scope; preserve mandatory validation
```

Run the targeted examples in [d3-scenarios.md](d3-scenarios.md) when changing D3. They exercise
reference resolution only, not the complete six-detector audit or host activation.

---

### D4 — Rule Contradiction

**Goal**: Detect two skills whose `## Constraints` contain opposing rules on the same subject.

**Method**:

- Extract each line from `## Constraints` of every SKILL.md and any explicitly normative constraint
  section in its reference files
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

**Goal**: Detect two skills whose names are identical across user and project scope or close enough
to cause confusion (typo, plural, name variation).

**Method**:

- Compare all selected slugs pairwise and reuse the other canonical collection's slug inventory
- Flag an exact slug in both collections as CRITICAL because hosts can discover both copies
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

**Context**: The local convention splits `references/` files by scope when behaviour differs per
target (e.g. `integration-api.md` vs `integration-worker.md`). This detector validates conditional
reference routing, not activation routing.

**Method**:

Step 1 — **Inventory scoped files** across all skills:

- For every `references/` file whose name matches `<topic>-<scope>.md`, classify it as a **scoped
  file** only when **at least one sibling file in the same `references/` directory shares the same
  `<topic>` with a different `<scope>` suffix** (e.g. `integration-api.md` and
  `integration-worker.md` both present).
- A file that has no same-topic sibling is treated as a **plain reference file** and skipped by D6
  entirely — regardless of how its name looks (e.g. `cross-check.md`, `sync-index.md`,
  `react-testing.md` all fail this test and are ignored).
- The sibling test is the only classification gate. Scope names such as `api`, `worker`,
  `frontend`, or `backend` are examples, not an allowlist.
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
# Cross-Check Report — <skills-root>/

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

### [D3] Unconditional Project Dependency — <skill-A> → <project-skill>
...

### [D3] Unavailable Skill Dependency — <skill-A> → <slug>
...

### [D6] Scoped Reference Conflict — <skill>
...

### [D6] Scoped Reference Duplication — <skill-A> / <skill-B>
...

---

## 🔵 Info (non-blocking)

### [D3] Cross-Scope / Checkout-Only / Unverified Reference — <skill-A> → <slug>
...

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
| 9 | D3 Unconditional Project Dependency | 🟡 | user-skill, project-skill | Make availability and project applicability conditional |
| 10 | D3 Cross-Scope / Checkout-Only Reference | 🔵 | skill-A, slug | Record scope and current availability; never duplicate |
| 11 | D3 Unverified Reference | 🔵 | skill-A, slug | Verify unavailable inventory before declaring absence |
| 12 | D3 Unavailable Skill Dependency | 🟡 | skill-A, slug | Satisfy the prerequisite in its proper scope; never bypass mandatory validation |

**No files were modified.**
Please indicate which inconsistencies you want to fix and how to proceed.
```

---

## Severity Levels

| Level       | Meaning                                          | Required action           |
| ----------- | ------------------------------------------------ | ------------------------- |
| 🔴 CRITICAL | Risk of incorrect activation or broken reference | Fix before next doctor    |
| 🟡 WARN     | Risk of future divergence or confusion           | Address in current sprint |
| 🔵 INFO     | Cosmetic, no functional impact                   | Address if time permits   |

---

## Constraints

- **Strict scope**: never read skill content outside `<skills-root>/`; reuse the other canonical
  collection's slug-only inventory for D3 and D5. Availability checks use discovery metadata, not
  additional skill bodies. An unavailable inventory is a limitation, not proof of absence.
- If an out-of-scope check seems necessary, **complete the full analysis first**, then ask the user
  for confirmation at the end of the report, explaining why the extra read would be needed.
- **This operation is read-only** — if the user asks you to apply a fix inline during the
  cross-check, refuse: "Cross-check is read-only. Run `/skill-manager fix <name>` to apply this
  change."
- **Activation router absence is not a finding** — descriptions route by default. Only report a
  router rule whose claimed behavioral evidence is missing or contradicted.

---

## Known Limitations

- **D1 is heuristic** — token overlap is an indicator, not a proof. Two skills may legitimately
  share keywords if their domains are complementary (e.g. `git-commit` and `pr-create` both mention
  "git").
- **D2 does not detect semantics** — two procedures expressed differently but doing the same thing
  may go undetected.
- **D3 only checks slug-style references** — it does not validate absolute paths (`.cursor/rules/`,
  `services/api/...`, `AGENTS.md`, etc.) or URLs. References to files outside `<skills-root>/` are
  intentionally out of scope and will not be flagged.
- **D4 requires human judgment** — some contradictions are intentional (e.g. "always use Jest" in a
  backend skill, "never use Jest" in a frontend skill).
- **D6 similarity thresholds are approximate** — 80% / 60% are heuristic; a 79% similar pair may
  still need merging. Always read the actual files before deciding.
- **D6 Step 2 intentional splits** — if a skill deliberately splits `api` and `worker` references
  because the rules genuinely differ, D6 will report 🔵 Info (not a warning). The detector does not flag
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
