# Cursor rules and commands

This file points Codex to the Cursor Skills, Rules and Commands used in this repo.

## Agent behavior (scope)

- **Prefer small iterations.** Do only what was asked; avoid expanding to "all" or "everything" unless the user explicitly says so.
- **Minimal scope.** If the request could apply to one item or many (e.g. "migrate the X rule"), assume **one** and change only that item. If in doubt, ask.
- **No big refactors by default.** Do not refactor or migrate other similar items in the same change; the user will ask for further steps if they want them.

## Agent Skills

Skills live in **`.cursor/skills/`**. Each skill is a folder with a `SKILL.md` file. Codex and Claude see them via symlinks (`.codex/skills` → `.cursor/skills`, `.claude/skills` → `.cursor/skills`).

- `cli-tools` — Prefer Rust-based CLI tools (eza, fd, rg, bat, etc.)
- `commits` — Conventional Commits format and scopes
- `cursor` — Cursor editor config (settings, keybindings, extensions)
- `fish` — Fish shell conventions
- `general` — Dotfiles guidelines (English, minimal config, Makefile for installs)
- `johnny-decimal` — Johnny Decimal + PARA organization
- `makefile` — Makefile structure and install workflow
- `neovim` — LazyVim config and plugin layout
- `scripts` — Bash scripts conventions

## Cursor Rules

- [rules/commits.md](.cursor/rules/commits.md)
- [rules/cursor.md](.cursor/rules/cursor.md)
- [rules/fish.md](.cursor/rules/fish.md)
- [rules/general.md](.cursor/rules/general.md)
- [rules/johnny-decimal.md](.cursor/rules/johnny-decimal.md)
- [rules/makefile.md](.cursor/rules/makefile.md)

## Cursor Commands

- [commands/code-review.md](.cursor/commands/code-review.md)
- [commands/git-commit.md](.cursor/commands/git-commit.md)
- [commands/git-pr.md](.cursor/commands/git-pr.md)
- [commands/git-push.md](.cursor/commands/git-push.md)
