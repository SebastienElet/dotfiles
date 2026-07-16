---
name: scripts
description: >
  Create and maintain portable Bash scripts in this repository. Use when adding or editing
  standalone scripts, shebangs, error handling, or executable tooling. Make sure to use this skill
  whenever a change touches scripts/ or introduces shell automation, even if the request calls it a
  helper or hook.
metadata:
  category: dev
---

# Scripts

## Overview

Maintain executable Bash tooling under `scripts/` without introducing host-specific assumptions.
New scripts must work on both macOS and Linux and remain easy to invoke from the Makefile or Git
hooks.

## Usage

Invoke this skill with a script-related request:

```text
$scripts add an extensionless cleanup helper
$scripts make this hook portable across macOS and Linux
```

## Steps

1. Inspect neighboring scripts and every caller before changing names, arguments, or exit codes.
2. Use Bash for standalone scripts and `#!/usr/bin/env bash` for new files.
3. Give executable scripts descriptive, extensionless names such as `git_hook_assert_eslint`.
4. Add explicit validation and meaningful non-zero exit codes for invalid input or failed work.
5. Prefer commands available on both macOS and Linux; gate unavoidable platform differences with
   an explicit `uname` check.
6. Preserve the executable bit and verify syntax with `bash -n scripts/<name>`.
7. For destructive behavior, default to inspection or dry-run and refuse unsafe paths.

## Gotchas

- **Adding a `.sh` extension** — executable helpers in this repository are invoked by their command
  name — use an extensionless filename and preserve executable mode.
- **Depending on modern Bash features** — macOS may provide an older Bash — avoid features such as
  associative arrays unless the script explicitly verifies a compatible runtime.
- **Using GNU-only utilities** — flags supported on Linux may fail on macOS — prefer portable syntax
  or branch explicitly by platform.
- **Testing destructive commands against real data** — a correct script can still remove the wrong
  path — validate with temporary fixtures and a dry-run before real execution.

## Constraints

- Use Bash, not Fish, for standalone repository scripts.
- Use `#!/usr/bin/env bash` for every new executable script.
- Keep executable script names extensionless and descriptive.
- Never ignore command failures that affect correctness.
- Never assume GNU-specific commands or flags are available on macOS.
- Do not rename existing scripts or change their interface without checking all callers.
