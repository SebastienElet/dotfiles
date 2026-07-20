# Dotfiles Agent Instructions

This file is the single source of truth for all coding agents working in this repository.

> **Conflict rule:** if an agent-specific adapter disagrees with this document, `AGENTS.md` wins.

## Scope

- **Prefer small iterations.** Do only what was asked; avoid expanding to all similar items.
- **Keep changes minimal.** Reuse existing structures and ask when the intended scope is ambiguous.
- **Avoid broad refactors.** Do not migrate or reorganize unrelated configuration by default.

## Personal Brain

- Before operating on content under `~/Brain`, read and follow `~/Brain/AGENTS.md`.
- Default durable memory target is `~/Brain`, not an agent-specific memory store: facts, decisions, and reference info worth recalling later belong there so every agent can reuse them. Only use an agent-specific memory system for things scoped to that agent's own behavior (e.g. its interaction preferences with this user).

## Shared Skills

- `.agents/skills/` is the single source of truth for reusable agent skills.
- Read `.agents/skills/README.md` for the current skill index.
- Use `skill-manager` whenever creating, migrating, editing, validating, or organizing skills.

## Agent Adapters

- `CLAUDE.md` delegates repository instructions to this file.
- `.claude/skills`, `.codex/skills`, and `.cursor/skills` expose the canonical shared skills.
- Agent-specific directories must not duplicate repository-wide rules or skill content.
