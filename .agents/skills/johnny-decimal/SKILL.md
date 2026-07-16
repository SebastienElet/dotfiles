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
AA-BB - Area
CC - Category
CC.ID - Item
10.YYNNN - Project
```

An area is a ten-number range such as `20-29 - Areas`. A category is a two-digit number within that
range, such as `27 - Finances`. A regular item ID combines its category with a two-digit identifier,
such as `27.05 - Fiscalité`. Projects are the explicit exception: category `10` uses the five-digit
`YYNNN` suffix, such as `10.25001 - Website Redesign`.

### PARA Mapping

| JD Area | PARA      | Purpose                                 |
| ------- | --------- | --------------------------------------- |
| `10-19` | Projects  | Active projects with a defined outcome  |
| `20-29` | Areas     | Ongoing responsibilities to maintain    |
| `30-39` | Resources | Reference materials, topics of interest |
| `90-99` | Archives  | Completed or inactive items             |
| `Inbox` | None      | Items to process and organize           |

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
2. Resolve the destination without inventing index data:
   - If the user provides an authoritative indexed folder or path, use it directly.
   - If lookup or validation is needed, consult the Johnny Decimal index in Apple Notes using an
     available Apple Notes integration.
   - If the needed index is unavailable, ask the user for the category, ID, or destination. If a
     non-blocking answer is still useful, explicitly label it as provisional and leave unknown
     numbers unspecified.
3. Map the item to the indexed structure:
   - Each area range contains at most 10 two-digit categories.
   - Regular item IDs use `CC.ID - Title`, where `CC` is the category and `ID` is unique within it.
   - Projects use the exception `10.YYNNN - Project Title`, where `YY` is the start year and `NNN`
     is the sequential project number from the index.
   - For completed or inactive items, require an indexed destination in `90-99 - Archives`.
     Preserve the existing ID unless the index explicitly maps or renumbers it.
4. Keep the hierarchy flat. File the item directly in its indexed category or ID folder; do not add
   a year folder or other unindexed nesting.
5. Build the filename from folder labels exactly as `scripts/jdl` does:

   ```text
   Category-only:  {Category} - [Date - ]{Description}.{ext}
   ID/subcategory: {Category} - {Subcategory} - [Date - ]{Description}.{ext}
   ```

   - For a file directly in a category folder, inherit only that category label.
   - For a file in an ID folder, inherit the category and subcategory labels.
   - Use the exact ASCII separator ` - ` (space-hyphen-space).
   - Add the ISO date `YYYY-MM-DD` only when the document's actual date is known.
   - Omit the date for undated reference documents, rules, and templates.
   - Write the description in Title Case; accents are allowed.
6. Return the indexed or clearly provisional path and filename. Keep one location per ID and use a
   descriptive title.

### Filename Examples

| Folder Path                                              | Filing level | File Name                                                  |
| -------------------------------------------------------- | ------------ | ---------------------------------------------------------- |
| `20-29 - Areas/27 - Finances/`                           | Category     | `Finances - 2026-04-15 - Avis d'impôt.pdf`                |
| `20-29 - Areas/27 - Finances/27.05 - Fiscalité/`         | ID           | `Finances - Fiscalité - 2024-04-15 - Déclaration 2042.pdf` |
| `20-29 - Areas/21 - Bigfoot/21.03 - Meeting/`            | ID           | `Bigfoot - Meeting - 2024-05-14 - Retour Septeo.xlsx`      |
| `20-29 - Areas/23 - Sport/23.01 - Tir à l'arc/`          | ID           | `Sport - Tir à l'arc - 2024-09-14 - Licence FFTA.pdf`      |
| `30-39 - Resources/34 - Administratif/34.03 - Identité/` | ID           | `Administratif - Identité - Passeport.pdf`                 |

## Gotchas

- **Ignoring an authoritative path supplied by the user**: an unnecessary lookup can override
  verified index data. Use the supplied indexed folder directly.
- **Copying an ID from an example**: examples illustrate syntax, not the current index. Consult
  Apple Notes when lookup is needed, ask the user, or leave the proposed ID unspecified.
- **Treating a tax or project year as a document date**: a year alone does not establish an ISO
  document date. Omit the date until the actual document date is known.
- **Adding a year directory for convenience**: extra nesting breaks the flat indexed structure.
  Keep the file directly in the indexed category or ID folder.
- **Renumbering an archived item automatically**: moving an item does not authorize a new ID. Use
  the indexed archive destination and preserve its ID unless the index explicitly says otherwise.
- **Applying these paths to `~/Brain`**: the two roots have different governance. Restrict this
  skill's filesystem decisions to `~/Documents`.

## Constraints

- Never invent, guess, or allocate a Johnny Decimal category, ID, or project sequence number.
- Use an authoritative indexed folder or path supplied by the user without requiring another
  lookup.
- Consult the Apple Notes index only when lookup or validation is needed; if it is unavailable, ask
  the user or explicitly mark an ID-less proposal as provisional.
- Keep `~/Documents` flat below the indexed category or ID folder; do not create year folders.
- Use the exact ASCII ` - ` separator required by `scripts/jdl` in directory and file names.
- A filename must inherit the category label and, only when it is inside an ID/subcategory folder,
  the subcategory label.
- Include a filename date only when the document's actual date is known.
- Archive only to an indexed destination, preserving the existing ID unless the index explicitly
  maps or renumbers it.
- Maintain one location per ID and avoid duplicate filing.
