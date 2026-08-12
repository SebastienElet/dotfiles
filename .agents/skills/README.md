# Shared Skills

This directory is the single source of truth for reusable agent skills.

## Conventions

- One skill per subdirectory.
- Each skill must include a `SKILL.md` file.
- Optional folders: `scripts/`, `references/`, `assets/`, `evals/`.
- Manage skills with `/skill-manager`.

## Dev

| Skill               | Description |
| ------------------- | ----------- |
| `enforcement-code`  | Write code whose purpose is to refuse something: hook, guard, validator, permission check, lint rule, CI gate. |
| `merge-verdict`     | Deliver a merge verdict on an open pull request, yours or another author's. |
| `neovim`            | Maintain this repository's Neovim and LazyVim configuration. |
| `scripts`           | Create and maintain portable Bash scripts in this repository. |

## Ops

| Skill               | Description |
| ------------------- | ----------- |
| `apple-notes`       | Write to Apple Notes with AppleScript: create, move, rename notes and folders, HTML bodies, attachments. |
| `do-nothing-script` | Turn a repeated manual procedure into a do-nothing script (Slimmon): one function per step, printed then awaiting the operator, automated one step at a time. |
| `dotfiles`          | Apply this repository's conventions for configuration, symlinks, platform differences, and tool installation. |
| `handoff`           | Hand the current work to a fresh session instead of letting the context compact. |
| `johnny-decimal`    | Organize ~/Documents with the Johnny Decimal and PARA hybrid. |
| `para-organizer`    | Apply PARA (Projects, Areas, Resources, Archives) to a file tree outside ~/Documents and ~/Brain. |
| `skill-manager`     | Manage .agents/skills: create, doctor, fix, cross-check, and sync the README index. |
| `things-tasks`      | Manage Things 3 tasks, projects, and areas through the thangs CLI. |
