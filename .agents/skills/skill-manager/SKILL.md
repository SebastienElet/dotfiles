---
name: skill-manager
description: >
  Manage .agents/skills: create new skills with proper scaffolding, run doctor checks on existing skills for quality and portability, fix non-conforming skills after a doctor run, detect inter-skill
  inconsistencies with cross-check, and keep the README index synchronized. Use when creating a new
  skill, improving an existing skill, modifying a skill's description or frontmatter, fixing doctor findings, detecting trigger overlaps or rule contradictions between skills, organizing the skills
  directory, or ensuring multi-agent compatibility. Make sure to use this skill whenever authoring a
  new skill, reviewing skill quality, cross-checking skills for conflicts, restructuring skills,
  modifying any SKILL.md file (even a single field like description or metadata), applying fixes from doctor, or migrating to the agentskills.io standard, even if you're just tweaking one line in an
  existing skill — references/conventions.md must be read first.
metadata:
  category: ops
  author: Bellman
---

# Skill Manager

## Overview

Central tool for managing the `.agents/skills` directory. Handles five operations: scaffolding new
skills with enforced conventions aligned to the agentskills.io standard (compatible with Cursor,
Claude Code, Codex, Gemini CLI, and Copilot), running doctor checks on existing skills for quality and portability,
applying fixes from doctor, detecting inter-skill inconsistencies (trigger overlaps, content duplications,
dead references, rule contradictions, slug ambiguity, scoped reference conflicts), and keeping the
README index synchronized.

## Usage

```text
/skill-manager create <name>    — scaffold a new skill
/skill-manager doctor [name]     — check one skill or all skills
/skill-manager fix <name>       — apply fixes from doctor to a specific skill
/skill-manager cross-check      — detect inter-skill inconsistencies
/skill-manager sync-index       — rebuild the README index
```

- `<name>`: kebab-case slug for the skill (e.g. `my-feature-skill`)
- `[name]`: optional for `doctor` — omit to check all skills; **required for `fix`** — if omitted,
  the agent asks which skill to fix

Examples:

- `/skill-manager create ics-import` → scaffold `.agents/skills/ics-import/SKILL.md`
- `/skill-manager doctor pr-create` → check a single skill
- `/skill-manager doctor` → check all skills and produce a report
- `/skill-manager fix testing` → fix all non-conformances in the `testing` skill
- `/skill-manager fix` → agent asks: "Which skill do you want to fix?"
- `/skill-manager cross-check` → scan all skills and report inter-skill conflicts
- `/skill-manager sync-index` → rebuild the README from current state

## Steps

1. **Identify the operation** — `create <name>`, `doctor [name]`, `fix <name>`, `cross-check`, or
   `sync-index`. If `fix` is invoked without a name, ask: "Which skill do you want to fix?"
2. **Read the reference file** for that operation (see References below).
3. **Execute the procedure** — follow the steps in the reference file.
   - **For `cross-check` specifically**: produce the report, present it to the user, and **stop**.
     Do not modify anything without explicit instruction.
4. **If you created, fixed, or renamed a skill** — run the sync-index procedure so the README stays
   up to date. This does not apply after `doctor`-only or `cross-check` runs.

## Gotchas

- **`.cursor/skills` is a symlink to `.agents/skills`** — both paths point to the same directory.
  Never treat `.cursor/skills/<skill>/SKILL.md` and `.agents/skills/<skill>/SKILL.md` as separate
  copies to synchronize: any write to one is immediately visible in the other. Always work
  exclusively on `.agents/skills/`.

- **Editing a skill without reading `conventions.md` first** — even a minor change (one description
  line, one metadata field) must go through `references/conventions.md`. Reading the target SKILL.md
  alone is not enough: formatting rules (pushy description, mandatory sections, allowed fields) live
  in `conventions.md`. Skipping this step produces non-conforming descriptions that other agents
  will not activate correctly.
- **Frontmatter fields are not all standard** — if you're using `disable-model-invocation`,
  `category` (top-level), or `paths`, remove them. Only `name`, `description`, `license`,
  `compatibility`, and `metadata` work across all 5 agents. Move `category` to `metadata.category`.
- **Description format matters for activation** — generic descriptions ("Helps manage skills") won't
  trigger. Use the pushy format: "[What it does]. Use when [triggers]. Make sure to use whenever
  [keywords], even if [edge case]." This is how Cursor, Claude, Codex, Gemini, and Copilot all
  decide to load the skill.
- **Forgetting `## Gotchas` breaks doctor** — every skill must include a `## Gotchas` section with 3+
  project-specific warnings. This is mandatory in Phase 1+. If you see a failing doctor check due to
  missing `## Gotchas`, add it immediately.
- **SKILL.md over 500 lines causes truncation** — if your SKILL.md exceeds 500 lines, move content
  to `references/` subdirectory and add a `## References` section linking to them. Progressive
  disclosure keeps the skill focused and context-efficient.
- **Doctor before sync-index** — if skills look inconsistent, run doctor first, fix issues, then sync.
  Syncing a broken state captures the broken state in README.
- **Applying a cross-check fix inline** — you spot a D3 dead reference and are tempted to rename the
  slug directly. Refuse: cross-check is read-only. Redirect the user: "Run
  `/skill-manager fix <name>` to apply this change." The `fix` command accepts cross-check findings
  as valid input — no need to re-run `doctor` if a `cross-check` report already exists.
- **Run cross-check after doctor, not before** — non-conforming skills (wrong description format,
  missing sections) produce noisy false positives: D1 over-counts trigger overlap because the
  description tokens are malformed, and D4 may miss real contradictions because `## Constraints` is
  absent. Fix individual skills first.

## Constraints

- **Always read `references/conventions.md` first** — it is the SSOT for skill format and
  agentskills.io portability.
- **Never overwrite** an existing `SKILL.md` without explicit user confirmation.
- **Sync the index** after any create, rename, or delete operation.
- **Doctor before sync** if skills look inconsistent — fix issues first, then sync.
- **cross-check produces no writes** — the output of cross-check is a report only. Even an obviously
  trivial finding (a dead reference, a slug typo) must go through `/skill-manager fix` before any
  file is touched.
- **Use only standard frontmatter** — `name`, `description`, `license`, `compatibility`, `metadata`.
  No custom top-level fields.
- **Move `category` to `metadata.category`** — top-level `category` breaks portability with Codex,
  Gemini, and Copilot.
- **Description must be pushy** — this is the single trigger mechanism across all 5 agents. Make it
  specific and include implicit cases.
- **All content must be in English** — skill files, gotchas, references, and inline comments are
  consumed by multi-agent tooling; non-English content breaks portability and reduces activation
  reliability.

## References

Read the corresponding reference file for detailed procedures:

- [references/conventions.md](references/conventions.md) — **READ FIRST**: canonical skill
  structure, agentskills.io standard, multi-agent compatibility, progressive disclosure, gotchas
  section
- [references/create.md](references/create.md) — scaffolding procedure for new skills, SKILL.md
  template
- [references/doctor.md](references/doctor.md) — doctor checklist and scoring, portability validation
- [references/fix.md](references/fix.md) — remediation procedure: apply fixes from doctor, ordered by
  priority, with before/after examples
- [references/cross-check.md](references/cross-check.md) — cross-check procedure: 6 inter-skill
  detectors (trigger overlap, content duplication, dead references, rule contradictions, slug
  ambiguity, scoped reference conflict)
- [references/sync-index.md](references/sync-index.md) — README index maintenance
- [references/evals.md](references/evals.md) — trigger-queries.json format for skill activation
  validation (Phase 4)
