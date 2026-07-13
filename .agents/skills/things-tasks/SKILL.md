---
name: things-tasks
description: Manage Things 3 tasks, projects, and areas through the thangs CLI. Use when the user asks to list today's tasks or other Things lists, create or edit tasks, complete or cancel tasks, create projects or areas, filter by project/area/tag, or inspect thangs options.
---

# Things Tasks

## Workflow

Use `thangs` for Things 3 task operations. Prefer JSON output when parsing or selecting tasks:

```bash
thangs list --list "Today" --json
```

Default to `thangs list --list "Today"` when the user asks what is due today or asks to manage current tasks. Use exact names and quote task, project, area, and tag values that contain spaces.

Before mutating Things data, confirm intent when the request is ambiguous, destructive, or could affect multiple tasks. `complete` marks a task done; `cancel` moves it to trash.

## Commands

List tasks:

```bash
thangs list [--list Today|Upcoming|Anytime|Someday] [--project <name>] [--area <name>] [--tag <name>] [--json]
```

Create a task in the Inbox:

```bash
thangs add <name> [--notes <text>] [--due YYYY-MM-DD] [--tags tag1,tag2] [--project <name>] [--area <name>] [--json]
```

Edit a task:

```bash
thangs edit <task> [--name <newName>] [--notes <text>] [--due YYYY-MM-DD] [--tags tag1,tag2] [--project <name>] [--area <name>] [--json]
```

Complete or cancel a task:

```bash
thangs complete <task> [--json]
thangs cancel <task> [--json]
```

Create projects and areas:

```bash
thangs add-project <name> [--area <name>] [--notes <text>] [--deadline YYYY-MM-DD] [--json]
thangs add-area <name> [--json]
```

Inspect CLI help or install the bundled Claude skill:

```bash
thangs --version
thangs --help
thangs <command> --help
thangs install-skill [--force] [--json]
```

## Checklist Items (URL scheme only)

Neither `thangs` nor the Things AppleScript dictionary can create or edit checklist items (checkable sub-items inside a task). The only token-free way is `things:///add` with the `checklist-items` parameter (newline-separated, max 100 items). `things:///update` also supports it but requires an auth token (Things → Settings → General → Manage Things URLs).

**Encoding:** Things does NOT decode `+` as a space. Never use `urllib.parse.urlencode`/`quote_plus`; percent-encode instead:

```python
from urllib.parse import quote
url = "things:///add?" + "&".join(f"{k}={quote(v, safe='')}" for k, v in params.items())
subprocess.run(["open", "-g", url], check=True)
```

To add a checklist to an existing task without a token: recreate it via `things:///add` (same title/notes/when/area), then trash the old one.

## AppleScript Pitfalls

- Checklist items are not exposed; notes and other properties are.
- Never `move ... to list "Trash"` while iterating a `whose` filter — the list mutates and throws "Invalid index" (-1719). Collect `id`s first, then move by `to do id`.

## Practices

- Read current tasks first before editing, completing, or canceling by name.
- Use `--json` for reliable parsing and concise summaries for the user.
- Preserve existing task fields unless the user asks to replace them; note that `--tags` replaces tags on edit.
- Use ISO dates (`YYYY-MM-DD`) for `--due` and `--deadline`.
- Report the exact task names changed and the command outcome.
