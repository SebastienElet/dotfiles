---
name: skill-manager
description: >
  Manage .agents/skills: create, doctor, fix, cross-check, and sync the README index. Use when
  creating or changing any skill, including requested behavior changes to a passing skill. Make
  sure to use this skill whenever editing SKILL.md, validating portability, resolving inter-skill
  conflicts, or rebuilding the skills index, even for one frontmatter field; read
  references/conventions.md first.
metadata:
  category: ops
---

# Skill Manager

## Overview

Manage the repository-wide `.agents/skills/` source of truth. The five operations scaffold skills,
audit individual quality, apply justified changes, detect inter-skill conflicts, and rebuild the
derived README index without changing the repository's adapter architecture.

## Usage

```text
/skill-manager create <name>    — scaffold a new skill
/skill-manager doctor [name]    — check one skill or all skills
/skill-manager fix <name>       — fix findings or apply an explicitly requested evolution
/skill-manager cross-check      — report inter-skill inconsistencies without writing
/skill-manager sync-index       — rebuild the deterministic README index
```

- `<name>` is a kebab-case skill slug.
- Omitting `[name]` from `doctor` audits every skill.
- Omitting `<name>` from `fix` requires asking which skill to change.

Examples:

- `/skill-manager create release-checks`
- `/skill-manager doctor scripts`
- `/skill-manager doctor`
- `/skill-manager fix apple-notes`
- `/skill-manager cross-check`
- `/skill-manager sync-index`

## Steps

1. Identify `create`, `doctor`, `fix`, `cross-check`, or `sync-index`.
2. Read `references/conventions.md` completely.
3. Read the operation reference listed below and follow it exactly.
4. For `cross-check`, present the report and stop; every write requires a later `fix` operation.
5. After create, fix, rename, or delete, run `sync-index` and verify its second run is byte-identical.

## Gotchas

- **Editing an adapter path** — `.agents/skills/` is canonical; `.claude/skills`, `.cursor/skills`,
  and `.codex/skills` are relative symlinks to it. Edit the source, never synchronize copies or
  reverse a link.
- **Skipping conventions for a small edit** — one frontmatter field can break discovery on every
  agent. Read `references/conventions.md` before any skill write.
- **Treating host-only fields as portable** — only six frontmatter fields belong to the standard,
  including experimental `allowed-tools`. A host accepting another field does not make it portable.
- **Putting positional shell placeholders in SKILL.md** — some clients template the body before an
  agent reads it, silently changing executable examples. Follow the safe placement and escaping
  rules in `references/conventions.md`.
- **Installing `skills-ref` during validation** — validation must not mutate the machine. Use it
  when present; otherwise report standard validation as unavailable and continue local checks.
- **Adding an activation router by reflex** — descriptions are the default router. Add an external
  rule only after repeated measured misrouting; absence alone is never a finding.
- **Applying a cross-check fix inline** — cross-check is strictly read-only. Record its finding,
  then invoke `fix <name>` in a separate operation.
- **Running cross-check before doctor** — malformed descriptions and missing constraints make D1
  and D4 noisy. Reach a clean doctor report first.

## Constraints

- Always read `references/conventions.md` before writing any skill file.
- Never overwrite an existing `SKILL.md` without an explicit create, fix, rename, or delete request.
- Use only standard frontmatter fields and keep local indexing data under `metadata`.
- Never install validation tooling implicitly.
- Never add an activation router without repeated behavioral evidence.
- Keep `cross-check` read-only; route every correction through `fix`.
- Run `sync-index` after any change that can affect index membership or content.
- Write all skill content in English.

## References

- [references/conventions.md](references/conventions.md) — canonical standard and local conventions
- [references/create.md](references/create.md) — scaffolding procedure
- [references/doctor.md](references/doctor.md) — individual and global audit
- [references/fix.md](references/fix.md) — findings and requested evolutions
- [references/cross-check.md](references/cross-check.md) — read-only inter-skill detectors
- [references/sync-index.md](references/sync-index.md) — deterministic README generation
- [references/evals.md](references/evals.md) — activation scenario format and evidence
