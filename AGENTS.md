# Dotfiles Agent Instructions

This file is the single source of truth for all coding agents working in this repository.

> **Conflict rule:** if an agent-specific adapter disagrees with this document, `AGENTS.md` wins.

## Scope

- **Prefer small iterations.** Do only what was asked; avoid expanding to all similar items.
- **Keep changes minimal.** Reuse existing structures and ask when the intended scope is ambiguous.
- **Avoid broad refactors.** Do not migrate or reorganize unrelated configuration by default.

## Code Structure

- Optimize for cohesion, not the fewest files. Minimal means the least code that remains easy to
  understand and change safely.
- Keep one responsibility per function and file. Parsing, orchestration, policy, I/O, and mutation
  are separate responsibilities unless their implementation is trivial.
- Treat 50 logical lines per production function and 250 lines per hand-written file as review
  triggers: split the unit or justify in the delivery note why keeping it intact is more cohesive.
- Do not extract a helper solely to satisfy a size trigger; every extracted unit must have a clear
  name and reason to change.
- These rules override any skill preference for the fewest files or the shortest diff. Before
  delivery, inspect every changed hand-written function and file against the triggers.

## Architecture Decisions

- `docs/adr/` records the structural decisions of this repository, indexed in `docs/adr/README.md`. Only decisions still in force are recorded.
- Never contradict an ADR silently. Either follow it, or state the conflict, then deliver what was asked along with the ADR that would need superseding.
- Routine changes — adding a tool target, updating a lockfile, editing a skill — need no ADR.

## Personal Brain

- Before operating on content under `~/Brain`, read and follow `~/Brain/AGENTS.md`.
- Default durable memory target is `~/Brain`, not an agent-specific memory store: facts, decisions, and reference info worth recalling later belong there so every agent can reuse them. Only use an agent-specific memory system for things scoped to that agent's own behavior (e.g. its interaction preferences with this user).

## Shared Skills

- Agent-specific directories must not duplicate repository-wide rules or skill content.
