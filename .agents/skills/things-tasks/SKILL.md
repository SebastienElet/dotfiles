---
name: things-tasks
description: >
  Manage Things 3 tasks, projects, and areas through the thangs CLI. Use when listing Things lists,
  creating or editing tasks, completing or canceling tasks, managing projects or areas, filtering
  by project, area, or tag, or inspecting thangs options. Make sure to use this skill whenever the
  user asks about their todos, Today list, deadlines, or Things data, even if they do not explicitly
  mention Things 3 or thangs.
metadata:
  category: ops
---

# Things Tasks

## Overview

Manage Things 3 tasks, projects, and areas with the `thangs` CLI. Use this skill for reading or
changing Things data, including requests phrased generically as todos, current tasks, or deadlines.

## Usage

Invoke the skill with a natural-language Things request:

```text
$things-tasks list my Today tasks
$things-tasks add "Renew passport" due 2026-08-01
$things-tasks complete "Submit expenses"
```

Use these `thangs` commands:

```bash
thangs list [--list Today|Upcoming|Anytime|Someday] [--project <name>] [--area <name>] [--tag <name>] [--json]
thangs add <name> [--notes <text>] [--due YYYY-MM-DD] [--tags tag1,tag2] [--project <name>] [--area <name>] [--json]
thangs edit <task> [--name <newName>] [--notes <text>] [--due YYYY-MM-DD] [--tags tag1,tag2] [--project <name>] [--area <name>] [--json]
thangs complete <task> [--json]
thangs cancel <task> [--json]
thangs add-project <name> [--area <name>] [--notes <text>] [--deadline YYYY-MM-DD] [--json]
thangs add-area <name> [--json]
```

Inspect the installed CLI when a command or option is uncertain:

```bash
thangs --version
thangs --help
thangs <command> --help
thangs install-skill [--force] [--json]
```

## Steps

1. Identify the requested Things operation and the task, project, area, tag, or built-in list in
   scope.
2. Default to `thangs list --list "Today"` when the user asks what is due today or asks to manage
   current tasks.
3. Read current tasks with `--json` before editing, completing, or canceling by name so the exact
   target and existing fields are known.
4. Confirm intent before a mutation when the request is ambiguous, destructive, or could affect
   multiple tasks.
5. Run the narrowest matching `thangs` command, quoting names that contain spaces and preserving
   fields the user did not ask to replace.
6. Report the exact task, project, or area names changed and the command outcome.
7. For checklist items, use `things:///add` with a percent-encoded, newline-separated
   `checklist-items` parameter; `thangs` and the Things AppleScript dictionary do not support them.
   `things:///update` requires an auth token from Things settings.

   ```python
   from urllib.parse import quote

   url = "things:///add?" + "&".join(f"{key}={quote(value, safe='')}" for key, value in params.items())
   subprocess.run(["open", "-g", url], check=True)
   ```

## Gotchas

- **Editing by an ambiguous task name** — `thangs` may target the wrong item when names collide.
  List current tasks with `--json`, identify the exact task, and confirm with the user if needed.
- **Using `--tags` during an edit** — the option replaces the task's tags instead of merging them.
  Read the existing task first and include every tag that must remain.
- **Creating checklist items through `thangs` or AppleScript** — neither interface exposes
  checklist creation or editing. Use `things:///add` with at most 100 newline-separated items; to
  update without a token, recreate the task and trash the old one.
- **Encoding Things URLs with `urlencode` or `quote_plus`** — Things does not decode `+` as a space.
  Percent-encode each parameter with `quote(value, safe="")`.
- **Moving filtered AppleScript items while iterating** — mutating a `whose` result can raise
  `Invalid index` (-1719). Collect the item IDs first, then move each `to do id`.

## Constraints

- Use `--json` whenever output must be parsed or a task must be selected reliably.
- Use exact names and quote task, project, area, and tag values that contain spaces.
- Use ISO dates (`YYYY-MM-DD`) for `--due` and `--deadline`.
- Preserve existing task fields unless the user explicitly asks to replace them.
- Treat `complete` as marking a task done and `cancel` as moving it to trash.
- Do not mutate multiple or ambiguous tasks without confirming the intended scope.
