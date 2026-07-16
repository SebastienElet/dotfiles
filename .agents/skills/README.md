# Shared Skills

This directory is the single source of truth for reusable agent skills.

## Conventions

- One skill per subdirectory.
- Each skill must include a `SKILL.md` file.
- Optional folders: `scripts/`, `references/`, `assets/`.
- Manage skills with `/skill-manager`.

## Dev

| Skill    | Description |
| -------- | ----------- |
| `neovim` | Maintain this repository's Neovim and LazyVim configuration. Use when editing Lua files under… |

## Ops

| Skill            | Description |
| ---------------- | ----------- |
| `dotfiles`       | Apply this dotfiles repository's configuration and installation conventions. Use when changing… |
| `johnny-decimal` | Organize files in ~/Documents with the Johnny Decimal and PARA hybrid system. Use when naming,… |
| `skill-manager`  | Manage .agents/skills: create new skills with proper scaffolding, run doctor checks on existing skills… |
| `things-tasks`   | Manage Things 3 tasks, projects, and areas through the thangs CLI. Use when listing Things lists… |

## Uncategorized

| Skill     | Description |
| --------- | ----------- |
| `scripts` | Bash scripts conventions. Action: add a valid `metadata.category`. |
