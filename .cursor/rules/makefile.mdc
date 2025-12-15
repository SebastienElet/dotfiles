---
description: Makefile structure and conventions
globs: Makefile
---

# Makefile Conventions

Structure targets to be self-documenting — the target name should clearly indicate its purpose, avoiding the need for comments.

## Sections (respect this organization)

- `terminal`: Shell and terminal tools (fish, tmux, nvim, wezterm...)
- `work`: Development tools (cursor, docker, gh, terraform...)
- `personal`: Personal apps (obsidian, rust, spotify...)
- `utils`: Utility applications (cleanshot, keycast...)

## Target Patterns

- Tool targets depend on `brew` and the binary path
- Config targets create symlinks from `~/.dotfiles/` to `~/.config/` or `~/`
- Use `${BREW_BIN}` and `${APP_BIN}` variables for paths
- GUI apps (`.app`) are macOS-only; CLI tools should work on both macOS and Linux

## Example

```makefile
newtool: brew ${BREW_BIN}/newtool
${BREW_BIN}/newtool:
	brew install newtool
```

Note: Homebrew works on both macOS and Linux. For Linux-only alternatives, consider adding conditional targets.
