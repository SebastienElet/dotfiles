# Cursor rules and commands

This file points Codex to the Cursor Skills, Rules and Commands used in this repo.

## Agent behavior (scope)

- **Prefer small iterations.** Do only what was asked; avoid expanding to "all" or "everything" unless the user explicitly says so.
- **Minimal scope.** If the request could apply to one item or many (e.g. "migrate the X rule"), assume **one** and change only that item. If in doubt, ask.
- **No big refactors by default.** Do not refactor or migrate other similar items in the same change; the user will ask for further steps if they want them.

## Agent Skills

Shared skills live in **`.agents/skills/`**. Each skill is a folder with a `SKILL.md` file. Keep `.cursor/skills/` for existing Cursor-specific skills until they are migrated explicitly.

- `skill-manager` — Audit, create, and update shared agent skills
- `cli-tools` — Prefer Rust-based CLI tools (eza, fd, rg, bat, etc.)
- `commits` — Conventional Commits format and scopes
- `fish` — Fish shell conventions
- `dotfiles` — Project guidelines (English, minimal config, Makefile for installs)
- `johnny-decimal` — Johnny Decimal + PARA organization
- `makefile` — Makefile structure and install workflow
- `neovim` — LazyVim config and plugin layout
- `scripts` — Bash scripts conventions

## Cursor Commands

- [commands/code-review.md](.cursor/commands/code-review.md)
- [commands/git-commit.md](.cursor/commands/git-commit.md)
- [commands/git-pr.md](.cursor/commands/git-pr.md)
- [commands/git-push.md](.cursor/commands/git-push.md)
