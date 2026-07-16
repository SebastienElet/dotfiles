# Shared Skills

This directory is the single source of truth for reusable agent skills.

## Conventions

- One skill per subdirectory.
- Each skill must include a `SKILL.md` file.
- Optional folders: `scripts/`, `references/`, `assets/`.
- Manage skills with `/skill-manager`.

## Ops

| Skill           | Description |
| --------------- | ----------- |
| `dotfiles`      | Apply this dotfiles repository's configuration and installation conventions. Use when changing… |
| `skill-manager` | Manage .agents/skills: create new skills with proper scaffolding, run doctor checks on existing skills… |
| `things-tasks`  | Manage Things 3 tasks, projects, and areas through the thangs CLI. Use when listing Things lists… |

## Uncategorized

| Skill             | Description |
| ----------------- | ----------- |
| `johnny-decimal`  | Johnny Decimal + PARA organization. Action: add a valid `metadata.category`. |
| `neovim`          | Neovim LazyVim configuration conventions. Action: add a valid `metadata.category`. |
| `scripts`         | Bash scripts conventions. Action: add a valid `metadata.category`. |
