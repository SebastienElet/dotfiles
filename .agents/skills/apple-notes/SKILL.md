---
name: apple-notes
description: >
  Create folders and notes in Apple Notes with AppleScript, including HTML-formatted bodies and
  nested folders. Use when adding, writing, or filing content into Apple Notes from the command
  line. Make sure to use this skill whenever a request mentions creating a note, a Notes folder,
  saving something "in my Notes", or scripting Notes with osascript, even if AppleScript is never
  named — the apple-notes MCP server is read-only and cannot write.
license: MIT
compatibility: macOS with the Notes app; osascript. Requires Automation permission for the calling terminal.
metadata:
  category: ops
  author: Bellman
---

# Apple Notes

## Overview

Apple Notes has no write API other than AppleScript. The `apple-notes` MCP server configured in this
repository only exports and lists — creating anything goes through `osascript`. This skill provides a
tested wrapper script for the two common cases (create a folder, create a note) and the raw
AppleScript patterns for everything else.

### Current structure (iCloud account)

`1 Projects`, `2 Areas`, `3 Resources`, `4 Archives` are the active PARA folders — file new notes
there. Every other top-level folder (the numbered Johnny Decimal ranges `10-19 - Projects`,
`20-29 - Areas`, `30-39 - Resources`, `90-99 - Archives` and their children) is **legacy**: kept
read-only while the owner migrates notes manually through the inbox. Never create, move, or file
anything into them.

## Usage

```bash
.agents/skills/apple-notes/scripts/notes.sh folder <name> [account]
.agents/skills/apple-notes/scripts/notes.sh note <folder> <title> [account]   # HTML body on stdin
```

- `account` (optional): defaults to `iCloud`. List accounts with
  `osascript -e 'tell application "Notes" to get name of accounts'`.
- Examples:
  - `notes.sh folder Recipes` → creates `Recipes` if missing (idempotent)
  - `printf '<div>Buy milk</div>' | notes.sh note Recipes 'Shopping list'`

## Steps

1. **Check the target exists** — list folders before writing so you don't create a near-duplicate
   of an existing one:

   ```bash
   osascript -e 'tell application "Notes" to tell account "iCloud" to get name of folders'
   ```

2. **Create the folder** with `notes.sh folder <name>`. It is a no-op when the folder already exists.

3. **Create the note** with `notes.sh note <folder> <title>`, piping the body as HTML on stdin. Use
   `<div>` per line, `<b>`/`<i>`/`<ul><li>` for formatting. Plain text works too but loses line
   breaks.

4. **Verify** by reading the note back:

   ```bash
   osascript -e 'tell application "Notes" to tell account "iCloud" to get name of notes of folder "Recipes"'
   ```

5. **For anything the script does not cover** (nested folders, moving, updating an existing note),
   write the AppleScript inline — see the patterns below.

### Nested folders

```applescript
tell application "Notes" to tell account "iCloud"
  if not (exists folder "Parent") then make new folder with properties {name:"Parent"}
  if not (exists folder "Child" of folder "Parent") then ¬
    make new folder at folder "Parent" with properties {name:"Child"}
end tell
```

### Appending to an existing note

```applescript
tell application "Notes" to tell account "iCloud"
  set n to note "Shopping list" of folder "Recipes"
  set body of n to (body of n) & "<div>New line</div>"
end tell
```

## Gotchas

- **Never pass both `name:` and `body:` when creating a note** — Notes derives the title from the
  body's first line and *also* prepends the `name:` value, so the title appears twice inside the
  note. Put the title as the first line of the body (`<div><h1>Title</h1></div>`), which is what
  `notes.sh note` does.
- **Literal newlines break AppleScript string literals** — a multi-line HTML body inlined into an
  `osascript` heredoc is a syntax error. `notes.sh` strips newlines (HTML ignores them); do the same
  in hand-written scripts, or concatenate with `& return &`.
- **`log` inside a `tell application "Notes"` block prints the specifier, not the value** — `log
  (name of n)` outputs `name of note id x-coredata://…`. Force evaluation with `get`: `log (get name
  of n)`.
- **`delete folder` sends notes to Recently Deleted, not oblivion, but it is still destructive** —
  never delete a folder or note without explicit user confirmation; there is no undo from the
  script's side.
- **`notes.sh folder` only creates root-level folders** — it takes a name, not a path, so asking it
  for a nested folder silently creates a second folder at the account root instead. Use the nested
  AppleScript pattern above (`make new folder at folder "Parent"`) for anything below the root.
- **`delete folder` on a non-empty folder silently fails and iCloud syncs it back** — a folder with
  notes or subfolders looks deleted, then reappears seconds later. Empty it first: delete each
  subfolder by name (`delete folder "Sub" of folder "Parent"`), then the notes bottom-up
  (`repeat with i from (count of notes of f) to 1 by -1`), then the folder itself. Also note that
  `folders of f` and `notes of f` raise `-1728` on some folders — enumerate by index or address
  children by name instead of iterating the collection.
- **First run prompts for Automation permission** — the calling terminal (or Claude Code) must be
  granted control of Notes in System Settings → Privacy & Security → Automation, otherwise
  `osascript` fails with error -1743.
- **The `apple-notes` MCP server cannot create anything** — its tools (`list_folders`, `list_notes`,
  `export_notes`) are read-only. Use it to inspect, this skill to write.

## Constraints

- **Always check whether the folder already exists** before creating one — Apple Notes happily
  creates two folders with the same name in the same account.
- **Never delete or overwrite an existing note or folder** without explicit user confirmation;
  appending (`set body of n to (body of n) & …`) is the safe default.
- **Always quote-escape user-supplied text** before interpolating it into AppleScript (`\` then `"`).
  `notes.sh` does this via its `as_quote` helper; reuse the same approach inline.
- **Default to the `iCloud` account** unless the user names another one.
