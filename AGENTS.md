# Cursor rules and commands

This file points Codex to the Cursor Skills, Rules and Commands used in this repo.

## Agent Skills

Skills live in **`.cursor/skills/`**. Each skill is a folder with a `SKILL.md` file. Codex and Claude see them via symlinks (`.codex/skills` → `.cursor/skills`, `.claude/skills` → `.cursor/skills`).

- `cli-tools` — Prefer Rust-based CLI tools (eza, fd, rg, bat, etc.)

## Cursor Rules

- [rules/commits.md](.cursor/rules/commits.md)
- [rules/cursor.md](.cursor/rules/cursor.md)
- [rules/fish.md](.cursor/rules/fish.md)
- [rules/general.md](.cursor/rules/general.md)
- [rules/johnny-decimal.md](.cursor/rules/johnny-decimal.md)
- [rules/makefile.md](.cursor/rules/makefile.md)
- [rules/neovim.md](.cursor/rules/neovim.md)
- [rules/scripts.md](.cursor/rules/scripts.md)

## Cursor Commands

- [commands/code-review.md](.cursor/commands/code-review.md)
- [commands/git-commit.md](.cursor/commands/git-commit.md)
- [commands/git-pr.md](.cursor/commands/git-pr.md)
- [commands/git-push.md](.cursor/commands/git-push.md)
