---
name: skill-manager
description: Audit, create, and update Agent Skills for shared Codex, Claude, and Cursor workflows. Use when asked to review a skill, create a new skill, improve SKILL.md frontmatter or instructions, reduce context pollution, add scripts/references/assets, validate skill structure, or align skills with agentskills.io and Claude skill guidance.
---

# Skill Manager

Use this skill to make skills small, portable, and easy for agents to load only when useful.

## Workflow

1. Clarify the concrete workflow the skill should enable. Prefer 2-3 realistic trigger examples over broad capability lists.
2. Decide whether a skill is warranted. Suggest a command, script, repo instruction, or normal prompt when the workflow is one-off or too broad.
3. Keep `SKILL.md` lean. Put trigger language in frontmatter, core procedure in the body, and detailed/background material in `references/`.
4. Add `scripts/` only for repeatable or fragile operations that should be deterministic.
5. Add `assets/` only for files copied or reused in generated output.
6. Validate after edits and fix structural issues before finalizing.

## Audit Checklist

- `name` is kebab-case, matches the folder name, and is 64 characters or less.
- `description` explains what the skill does and when to use it in 1024 characters or less.
- `SKILL.md` is procedural, concise, and free of broad background that can live in references.
- File links are relative to the skill root and avoid deep reference chains.
- Optional resources have a clear reason to exist; no placeholder files remain.
- Tool requirements, network needs, and product-specific behavior are explicit when relevant.
- The skill composes with other skills and does not assume it is the only active instruction set.

## Creating Skills

Use this structure by default:

```text
skill-name/
|-- SKILL.md
|-- agents/openai.yaml
|-- references/
|-- scripts/
`-- assets/
```

Create only the optional directories needed for the workflow. For shared dotfiles skills, prefer `.agents/skills/<skill-name>` as the source path.

## Updating Skills

Make the smallest change that improves triggering, procedure, or reliability. Preserve existing behavior unless the user asks for a redesign.

When a skill grows, move detailed examples, specs, and checklists into `references/skill-quality.md` or a focused sibling reference. Keep the body as the operating procedure.

## Validation

Run the available validator when present:

```bash
skills-ref validate ./path/to/skill
```

If `skills-ref` is unavailable, check frontmatter, folder naming, resource purpose, and broken relative links manually. Read [references/skill-quality.md](references/skill-quality.md) when a deeper audit is needed.
