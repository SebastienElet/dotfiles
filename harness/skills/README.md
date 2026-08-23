# User Skills

This directory is the canonical source for user-scoped agent skills.

## Conventions

- One skill per subdirectory.
- Each skill must include a `SKILL.md` file.
- Optional folders: `agents/`, `scripts/`, `references/`, `assets/`, `evals/`.
- Manage skills with `/skill-manager`.

## Dev

| Skill                | Description                                                                                          |
| -------------------- | ---------------------------------------------------------------------------------------------------- |
| `agent-instructions` | Maintain coding-agent instructions and their discovery paths.                                        |
| `claude-developer`   | Prepare manual implementation and correction prompts for Claude Code without invoking it.            |
| `codegraph`          | Explore large repositories structurally with CodeGraph.                                              |
| `enforcement-code`   | Write code whose purpose is to refuse: hook, guard, validator, permission check, lint rule, CI gate. |
| `pr-fix`             | Repair an open pull request after an independent merge review.                                       |
| `pr-verdict`         | Deliver a PR verdict on an open pull request, yours or another author's.                             |

## Product

| Skill               | Description                                                                          |
| ------------------- | ------------------------------------------------------------------------------------ |
| `linear-issue-spec` | Prepare implementation-ready Linear development issues as functional specifications. |

## Ops

| Skill           | Description                                                                                      |
| --------------- | ------------------------------------------------------------------------------------------------ |
| `handoff`       | Hand the current work to a fresh session instead of letting the context compact.                 |
| `skill-manager` | Manage user and project skills: create, doctor, fix, cross-check, and sync their README indexes. |
