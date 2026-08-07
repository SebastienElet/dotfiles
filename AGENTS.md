# Dotfiles Agent Instructions

This file is the single source of truth for all coding agents working in this repository.

> **Conflict rule:** if an agent-specific adapter disagrees with this document, `AGENTS.md` wins.

## Scope

- **Prefer small iterations.** Do only what was asked; avoid expanding to all similar items.
- **Keep changes minimal.** Reuse existing structures and ask when the intended scope is ambiguous.
- **Avoid broad refactors.** Do not migrate or reorganize unrelated configuration by default.

## Architecture Decisions

- `docs/adr/` records the structural decisions of this repository, indexed in `docs/adr/README.md`. Only decisions still in force are recorded.
- Before changing a structural choice (installer, shell, editor, package source, container runtime, agent instructions, skills layout), read the matching ADR. Its "Alternatives écartées" section usually already covers the option being reconsidered, with the reason it was dropped.
- Never contradict an ADR silently. Either follow it, or state the conflict, then deliver what was asked along with the ADR that would need superseding.
- A new structural decision gets a new ADR in the same MADR format, citing the commit hashes that carry it. A decision that replaces another one absorbs it: the superseded ADR is deleted, and its rationale moves to the new "Alternatives écartées" section.
- **Everything under `docs/`, ADRs included, is written in French**, by explicit exception to the English-documentation rule; `docs/AGENTS.md` holds the detail. Do not translate existing documents, and do not write the next one in English.
- Routine changes — adding a tool target, updating a lockfile, editing a skill — need no ADR.

## Personal Brain

- Before operating on content under `~/Brain`, read and follow `~/Brain/AGENTS.md`.
- Default durable memory target is `~/Brain`, not an agent-specific memory store: facts, decisions, and reference info worth recalling later belong there so every agent can reuse them. Only use an agent-specific memory system for things scoped to that agent's own behavior (e.g. its interaction preferences with this user).

## Shared Skills

- `.agents/skills/` is the single source of truth for reusable agent skills.
- Read `.agents/skills/README.md` for the current skill index.
- Use `skill-manager` whenever creating, migrating, editing, validating, or organizing skills.
- Agent-specific directories must not duplicate repository-wide rules or skill content.
