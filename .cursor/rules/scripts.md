---
description: Shell scripts conventions
globs: scripts/*
---

# Scripts

- Use **Bash** for portability (not Fish) in standalone scripts
- Scripts should be executable and have no extension
- Use descriptive names (e.g., `git_hook_assert_eslint`)
- Include error handling with meaningful exit codes
- Ensure scripts work on both macOS and Linux (avoid `gdate`, prefer portable alternatives)
- Use `#!/usr/bin/env bash` shebang for portability
