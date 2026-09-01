---
name: apple-notes
description: >
  Write to Apple Notes with AppleScript: create, move, rename notes and folders, HTML bodies,
  attachments. Use when adding, filing, or triaging Notes content. Make sure to use it whenever a
  request writes to Notes, even if AppleScript is unnamed — the apple-notes MCP server is read-
  only.
license: MIT
compatibility: macOS with the Notes app; osascript. Requires Automation permission for the calling terminal.
metadata:
  category: ops
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
.agents/skills/apple-notes/scripts/notes.sh move <src/path> <title> <dst/path> [new-title] [account]
.agents/skills/apple-notes/scripts/notes.sh attachments <folder/path> <title> <dir> [account]
```

- `account` (optional): defaults to `iCloud`. List accounts with
  `osascript -e 'tell application "Notes" to get name of accounts'`.
- `move` takes slash-separated nested paths, creates the missing destination levels, refuses to move
  out of a shared folder, and — with `new-title` — renames the note as it files it. On a note with
  attachments it renames via `set name` instead of rewriting the body, so the photos survive (the
  body's first line then still shows the old title — fix that by hand in the app if it matters).
- `attachments` exports every attachment of a note to `<dir>` as `<index>-<filename>`, the only way
  to get a copy of the photos out of Notes. Use it before anything that touches a body.
- Examples:
  - `notes.sh folder Recipes` → creates `Recipes` if missing (idempotent)
  - `printf '<div>Buy milk</div>' | notes.sh note Recipes 'Shopping list'`
  - `notes.sh move Notes 'Company : Hiring : Template' '3 Resources/Hiring' 'Hiring : Rejection email template'`

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

5. **Filing a note out of the inbox** (the default `Notes` folder) — use `notes.sh move` and pass a
   `new-title`: a title has to survive search, where the folder is invisible, so name it
   `Domain : Subject` matching the destination folder and drop stale prefixes (see the
   `para-organizer` skill). Add the header (below) by hand before moving.

   `Domain : Subject` is two segments, and the segment count mirrors the destination's depth: a note
   in `2 Areas/Company` is `Company : Subject`, and only a three-level destination like
   `20-29 - Areas/21 - Company/21.08 - Hiring` earns the legacy three-segment
   `Company : Hiring : Subject`. Copying a legacy title's shape into a two-level PARA folder is
   what makes a batch of freshly filed notes look inconsistent.

   The `Subject` is the actual topic, not the shape of the note: no generic label like `Meeting`,
   `Produit`, or `Points`, no attendee name standing in for the content, and no trailing date —
   Notes already stores the creation date, and a date in the title only pushes the searchable words
   past the truncation point. Keep the whole title under ~66 characters: beyond that Notes truncates
   the name shown in the list with an ellipsis, and that truncated string is the note's real name for
   AppleScript lookups.

6. **For anything the script does not cover** (updating an existing note) write the AppleScript
   inline — see the patterns below.

### Note header

Two lines, right after the title block, in French like the rest of the notes:

```text
Objectif : <why this note is kept> — <search terms absent from the title>
Mis à jour : AAAA-MM-JJ
```

`Objectif` is a search hook, not a summary: Notes searches the body, so it earns its place only by
carrying the words the owner would actually type and the title does not already hold. Rephrasing the
title (`IA : Artificial Analysis benchmarks de modèles` → "garder la référence Artificial Analysis
(benchmarks de modèles)") adds no retrievable term and is the failure mode to avoid.

Nothing else. No `État` (nearly every note is a fresh capture, so the field carries no information
and rots on relecture), no `Prochaine étape` (an open action belongs in Things 3 — see the
`things-tasks` skill — and the field otherwise sits empty). Never write `???` as a value: omit the
line instead.

### Triaging one inbox note

The inbox is the default `Notes` folder. Process it oldest-first, one note per round:

1. Read the oldest note with the MCP server — `list_notes` with `folder: "Notes"`,
   `sort: "date-created"`, then re-query the target with `title_contains` and
   `include_content: true`. Ignore rows whose `folderName` is not exactly `Notes`: the folder filter
   is a partial match.
2. Decide the destination among `1 Projects`, `2 Areas`, `3 Resources`, `4 Archives` — never a legacy
   numbered folder — plus a subfolder named after the domain. Reuse an existing subfolder when one
   fits; ask when the call is genuinely ambiguous.
3. Add the two-line header (`Objectif`, `Mis à jour`) documented above — never invent an objective,
   ask when the reason for keeping the note is not recoverable from its content. When the note reports
   attachments, list their names first (`get name of attachments of note …`): emoji glyphs
   (`1f3af.png`) are safe to rewrite over, real files (`photo.jpg`, `IMG_1048.heic`) are not — skip
   the header on those, since inserting it means rewriting the body.
4. File it with `notes.sh move Notes '<title>' '<dst/path>' '<Domain : Subject>'`.
5. Report the destination and the remaining inbox count, then stop — one note per round.

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
  body's first line and _also_ prepends the `name:` value, so the title appears twice inside the
  note. Put the title as the first line of the body (`<div><h1>Title</h1></div>`), which is what
  `notes.sh note` does.
- **Renaming a note means rewriting the body's first line, and only that** — `set name of n` renames
  the list entry, but doing it on top of a body rewrite blanks that first line, and doing it alone
  leaves the body contradicting the title. Notes also converts the `<h1>` into
  `<b><span style="font-size: 24px">…</span></b>` on save, so replace the whole first `<div>…</div>`
  block, not the tag. `notes.sh move` does this — except on a note with attachments, where a body
  rewrite is the greater evil and `set name` alone is the right trade (see below). The rename does
  stick: the MCP server reads the new name back out of the CoreData store.
- **`get body` is inconsistent about attachments, so any `set body` round-trip can destroy them** —
  on some notes the HTML carries the image inline as `<img src="data:image/jpeg;base64,…">`, on others
  a note reporting `attachmentCount: 1` comes back with no trace of it, and writing that version back
  deletes the photo for good (the note lands in Recently Deleted attachment-less, so there is nothing
  to restore). Never rewrite the body of a note with attachments: `notes.sh move` retitles those with
  `set name` instead, which leaves the body untouched. Export first with `notes.sh attachments` if you
  must touch a body anyway.
- **Attachments are read-only and cannot be created by script** — every property of the `attachment`
  class is `access="r"` in `Notes.sdef` and there is no `make new attachment`, so an attachment
  removed by a body rewrite cannot be put back. `save attachment … in file` is the only handle you
  get; `notes.sh attachments` wraps it. Two quirks: the `save` must run outside a `tell account`
  block (inside it, `file (POSIX file …)` resolves against the account and fails with `-1728`), and
  an unnamed inline image reports `name` as `missing value` and refuses to save at all (`-10000`).
- **Writing a body that contains a base64 `<img>` creates a real attachment** — Notes ingests the
  data URI and `count of attachments` goes up. Useful to know, but not a re-attach path: what you can
  read back out of a body is not reliably what you put in.
- **`attachmentCount` counts emoji, not just photos** — a note whose text uses 🎯💸📦 reports eight
  attachments that are 300 B–2 KB PNG glyphs named after their codepoint (`1f3af.png`). Read
  `get name of attachments of note …` before concluding a note is unsafe to rewrite: an emoji-only
  note takes the normal body-rewrite retitle and header without risk.
- **`duplicate` is not supported** — `duplicate note …` fails with `-1717` ("Notes can not be
  copied"), so you cannot test a destructive operation on a copy. Recreate a note from an exported
  body in a scratch folder instead.
- **A body-rewrite pipeline that fails writes an empty body and wipes the note, with no error** —
  `set body of n to ""` succeeds, the note keeps its place but loses everything and shows up as
  `New Note` in the list (the title lives in the body's first line). Always check the rewritten HTML
  is non-empty before writing it back; `notes.sh` now refuses the write instead. Recovery means
  retyping the content, so read the note before rewriting it.
- **A header has to go on line 2, never line 1** — retitling replaces the body's whole first
  `<div>…</div>` block, so a header prepended above the title is silently eaten by the next retitle.
  Move and retitle first, then insert the header after the title block.
- **A link attachment exposes its target via `get URL of`** — `get URL of attachment 1 of note …`
  returns the shared URL, so a note whose only attachment is a Safari link preview can be rewritten
  as a plain `<a href="…">` and then retitled — recover the URL by hand first, and only then rewrite
  the body. Notes counts that link preview as an attachment, so `notes.sh move` takes the `set name`
  route on it and never rewrites the body on its own.
- **A prefix env assignment in front of a pipeline only reaches the first command** —
  `VAR=x osascript … | perl -e '…$ENV{VAR}…'` gives perl an empty value, which silently produces an
  empty insertion rather than an error. `export` it instead; `notes.sh` does this at its retitle step.
- **A note cannot be addressed by the id the MCP server reports** — `note id "x-coredata://…/p3662"`
  fails with `-1728`. Address notes by name within their folder.
- **Literal newlines break AppleScript string literals** — a multi-line HTML body inlined into an
  `osascript` heredoc is a syntax error. `notes.sh` strips newlines (HTML ignores them); do the same
  in hand-written scripts, or concatenate with `& return &`.
- **`log` inside a `tell application "Notes"` block prints the specifier, not the value** — `log
(name of n)` outputs `name of note id x-coredata://…`. Force evaluation with `get`: `log (get name
of n)`.
- **`delete folder` sends notes to Recently Deleted, not oblivion, but it is still destructive** —
  never delete a folder or note without explicit user confirmation; there is no undo from the
  script's side.
- **Moving a note out of a shared folder revokes the collaborator's access** — and sharing cannot be
  scripted, only done from the app's UI. Check `get shared of folder "…"` before moving anything, and
  have the destination shared by hand first.
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
