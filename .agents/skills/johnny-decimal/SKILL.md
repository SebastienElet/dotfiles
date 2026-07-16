---
name: johnny-decimal
description: >
  Organize files in ~/Documents with the Johnny Decimal and PARA hybrid system. Use when naming,
  filing, moving, or locating numbered personal documents. Make sure to use this skill whenever a
  request concerns Johnny Decimal categories, PARA mapping, or document filenames under
  ~/Documents, even if the user only asks where a document belongs.
metadata:
  category: ops
---

# Johnny Decimal + PARA Organization System

## Overview

Organize personal files under `~/Documents` with a hybrid of
[Johnny Decimal](https://johnnydecimal.com/) structure and
[PARA](https://fortelabs.com/blog/para/). Use it when filing, naming, moving, or locating a numbered
personal document. This skill governs only `~/Documents`; it does not govern `~/Brain`.

### Structure

```text
AC.ID — Title
│
├── AC   = Area (10-99) + Category (X0-X9)
└── ID   = Unique identifier within category (01-99)
```

### PARA Mapping

| JD Area | PARA      | Purpose                                 |
| ------- | --------- | --------------------------------------- |
| `10-19` | Projects  | Active projects with a defined outcome  |
| `20-29` | Areas     | Ongoing responsibilities to maintain    |
| `30-39` | Resources | Reference materials, topics of interest |
| `90-99` | Archives  | Completed or inactive items             |
| `Inbox` | —         | Items to process and organize           |

## Usage

```text
/johnny-decimal <filing, naming, moving, or locating request under ~/Documents>
```

Examples:

- `/johnny-decimal Where should I file this tax document under ~/Documents?`
- `/johnny-decimal Name a passport scan for its existing identity folder.`
- `/johnny-decimal Locate the numbered folder for an archived project.`

## Steps

1. Confirm that the request concerns `~/Documents`. If it concerns `~/Brain`, do not apply these
   filesystem rules.
2. Consult the Johnny Decimal index in Apple Notes using an available Apple Notes integration when
   one is present.
   - Use the existing indexed category and ID; never infer one from an example.
   - If the index is unavailable, ask the user for the category and ID. If a non-blocking answer is
     still useful, explicitly label the location as a provisional proposal and leave the ID
     unspecified.
3. Map the item to the indexed structure:
   - Categories subdivide areas (`11`, `12`, `13`, and so on), with at most 10 categories per area.
   - IDs (`11.01`, `11.02`, and so on) are unique items within a category.
   - Projects use `10.{YY}{NNN} — Project Title`, where `YY` is the start year and `NNN` is the
     sequential project number from the index. Example: `10.25001 — Website Redesign`.
   - Completed or inactive items move to the indexed `90-99` Archives area.
4. Keep the hierarchy flat. File the item directly in its indexed category or ID folder; do not add
   a year folder or other unindexed nesting.
5. Build the filename from the folder labels:

   ```text
   {Category} - {Subcategory} - {Date} - {Description}.{ext}
   ```

   - Inherit `Category` from the parent category label and `Subcategory` from the ID folder label.
   - Use ` - ` (space-dash-space) as the separator.
   - Add the ISO date `YYYY-MM-DD` only when the document's actual date is known.
   - Omit the date for undated reference documents, rules, and templates.
   - Write the description in Title Case; accents are allowed.
6. Return the indexed or clearly provisional path and filename. Keep one location per ID and use a
   descriptive title.

### Filename Examples

| Folder Path                            | File Name                                                  |
| -------------------------------------- | ---------------------------------------------------------- |
| `27 - Finances/27.05 - Fiscalité/`     | `Finances - Fiscalité - 2024-04-15 - Déclaration 2042.pdf` |
| `23 - Sport/23.01 - Tir à l'arc/`      | `Sport - Tir à l'arc - 2024-09-14 - Licence FFTA.pdf`      |
| `34 - Administratif/34.03 - Identité/` | `Administratif - Identité - Passeport.pdf`                 |

## Gotchas

- **Copying an ID from an example** — examples illustrate syntax, not the current index. Consult
  Apple Notes, ask the user, or leave the proposed ID unspecified.
- **Treating a tax or project year as a document date** — a year alone does not establish an ISO
  document date. Omit the date until the actual document date is known.
- **Adding a year directory for convenience** — extra nesting breaks the flat indexed structure.
  Keep the file directly in the indexed category or ID folder.
- **Applying these paths to `~/Brain`** — the two roots have different governance. Restrict this
  skill's filesystem decisions to `~/Documents`.

## Constraints

- Never invent, guess, or allocate a Johnny Decimal category, ID, or project sequence number.
- Always consult the Apple Notes index when an integration is available; otherwise ask the user or
  explicitly mark an ID-less proposal as provisional.
- Keep `~/Documents` flat below the indexed category or ID folder; do not create year folders.
- Filename category and subcategory must inherit the indexed folder labels.
- Include a filename date only when the document's actual date is known.
- Maintain one location per ID and avoid duplicate filing.
