---
name: para-organizer
description: >
  Apply PARA (Projects, Areas, Resources, Archives) to a file tree outside ~/Documents and
  ~/Brain. Use when setting up, auditing, or reclassifying folders by actionability. Make sure to
  use it whenever a request mentions PARA or asks where a folder belongs. For ~/Documents use
  johnny-decimal.
metadata:
  category: ops
  author: Tiago Forte (Forte Labs)
  version: "2.0"
---

# PARA Organizer

## Overview

Implements the PARA Method on a real local folder: scan, ask, present a full plan, get approval,
then execute and write an inventory. It follows the book's "Sixty-Second PARA Setup" — archive
everything first, create project folders for what is active now, and let areas and resources emerge
just in time.

Scope: any folder the user names **except** `~/Documents` (governed by `johnny-decimal`) and
`~/Brain` (governed by `~/Brain/AGENTS.md`).

## Usage

```text
/para-organizer <folder to organize>
```

Examples:

- `/para-organizer Set up PARA in ~/Dropbox`
- `/para-organizer Audit my existing PARA folders and flag stale projects`
- `/para-organizer Is "Cooking" an Area or a Resource?`

## Steps

1. Confirm the target folder. If it is `~/Documents`, stop and use `johnny-decimal` instead. If it is
   `~/Brain`, read `~/Brain/AGENTS.md` and follow that instead.
2. Read `references/para-framework.md` — the source of truth for category definitions.
3. Read `references/workflow.md` and execute its four phases in order: Discover (read-only) →
   Present the Plan → Execute (only after approval) → Output Inventory.
4. If the user asks about mirroring PARA on other platforms, read
   `references/cross-platform-guide.md`.

### THE GOLDEN RULE: PLAN FIRST, ACT SECOND

This skill touches real files on the user's real computer. That means trust is everything. Here is the rule that governs the entire workflow:

**Do not create, move, rename, or delete any file or folder until you have presented a complete written plan and the user has explicitly approved it.**

At the very start of the conversation, tell the user something like: "I'm going to look at your files, ask you some questions, and then put together a complete plan showing every folder I'd create and every file I'd move. You'll review the whole plan before I touch anything — I won't create or move a single file until you say go."

Say this early and unprompted. The user needs to hear it before they start worrying about it.

---

The PARA Method was created by Tiago Forte. Before classifying anything, read `references/para-framework.md` in this skill's directory — it contains the precise definitions, distinction guidelines, and classification logic you need to make correct decisions. That reference is your source of truth; the summary below is just orientation.

### Quick orientation

PARA organizes all digital information into four categories based on *actionability* — how immediately relevant something is to your current work and life:

- **Projects** — Short-term efforts with a goal and a deadline. Active, finite, completable. Folder names should be short, pithy labels — the kind you'd scan on a one-page project list. Goal and deadline live separately (in the inventory), not inside the folder name. Examples: "New Website," "Japan Trip," "Q3 Report."
- **Areas** — Ongoing responsibilities with a *standard to maintain* but no end date. Examples: "Health," "Finances," "Direct Reports," "Product Development."
- **Resources** — Topics of ongoing *interest* for future reference. Not tied to a responsibility — purely curiosity-driven or useful as reference material. Examples: "Photography," "Marketing best practices," "Recipes."
- **Archives** — Inactive items from any of the above three categories. Completed projects, areas you're no longer responsible for, topics you've lost interest in.

The spectrum runs from most actionable (Projects) to least actionable (Archives). Information flows between categories as your life changes.

### Workflow overview

The skill has four phases:

1. **Discover** — Scan the user's files and ask about their projects, areas, and interests. Read-only — no changes.
2. **Present the Plan** — Show the user a complete, item-by-item plan: every folder to be created, every file to be moved, every classification decision. The user reviews, edits, and approves before anything happens.
3. **Execute** — Only after explicit approval, create folders and move files according to the plan.
4. **Output Inventory** — Save a clean markdown file listing the full PARA classification, along with tips for keeping the system healthy going forward.

The philosophy behind this order: trying to classify hundreds of existing files one-by-one is exactly the wrong way to start. As the book says, "Before you can create anything new, you have to clear out the old." The clean slate approach is fast, psychologically freeing, and nothing is lost — it all lives safely in the archive.

---

### Tone and approach

Keep the language warm, clear, and jargon-free.

The PARA Method is meant to be lightweight and forgiving. Resist the urge to over-organize. Key principles to embody:

- **Trust is the top priority.** You are touching the user's real files. Always present the plan first. Always get approval. Never surprise them.
- **No loose files at the top level.** Every file must live inside a named subfolder (a specific project, area, or resource). Nothing sits loose at the root of `1 Projects/`, `2 Areas/`, `3 Resources/`, or `4 Archives/`.
- **Default to more actionable on ties.** When a file could plausibly be either a Project or an Area (or an Area or a Resource), default to the more actionable category. It's better to over-promote to Projects than to bury something in Archives where it won't surface.
- There is no single "correct" place for any item — what matters is the user's *relationship* to it right now.
- Don't create subfolders or internal structure within PARA folders. The user can do that later if they want to. PARA doesn't prescribe it.
- Don't rename the user's files or folders unless they ask.
- Never create an empty folder. Only create a folder when you have something to put in it.
- Moving items between PARA categories over time is normal and healthy — it's not a sign that the original classification was wrong.
- When in doubt about a classification, ask. It's better to surface a question than to silently misfile something.
- Speed matters. A PARA setup that's 80% right today is better than a perfect one that takes hours. The user can always reclassify later.

### Naming items inside a project

A title has to survive search, where the folder is invisible: Spotlight and in-app search list the
item's name, never its container. `Question` or `Financement` inside a `Tesla` folder is unfindable
and unreadable out of context.

So name every item `Project : Subject` — `Tesla : Offres de crédit`, `Tesla : Devis assurance AXA`.
Prefix with the project (or the domain, e.g. `Auto : Tesla : …`, when a broader grouping helps),
never with the PARA category: `Projects : …` becomes false the day the project is archived, forcing
a rename of every item, while the project name stays true for life.

Finalize before filing, too. An item leaving the inbox gets a short header — objective, current
state, next step, last-updated date — above the raw capture. Ask for the facts you're missing; mark
what stays unknown `???` rather than inventing it. And look for related items already sitting in the
old structure: migrate a project as a whole, or the new folder ends up holding the least informed
version of the story.

## Gotchas

- **Applying the `1 Projects/` layout to `~/Documents`** — that root uses numbered Johnny Decimal
  ranges (`10-19 - Projects`, `20-29 - Areas`, …) and its index lives in Apple Notes. Creating PARA's
  named folders there corrupts the index. Hand the request to `johnny-decimal`.
- **Touching `~/Brain`** — it has its own governance in `~/Brain/AGENTS.md`, which must be read first
  and overrides everything here.
- **Executing before the plan is approved** — the whole value of this skill is the approval gate. A
  "quick" folder creation before Phase 2 breaks the trust the skill is built on.
- **Silently archiving screenshots and images** — they are usually captured for a reason. Flag them
  as ambiguous instead of guessing.
- **Recursing into subfolders during the scan** — PARA operates on top-level items only; deep scans
  waste context and produce classifications the user did not ask for.
- **Assuming a fresh setup** — if the folder already has PARA-shaped folders (even renamed ones),
  offer audit / sort / fresh-start instead of archiving everything.

## Constraints

- Never create, move, rename, or delete anything before presenting a complete plan and receiving
  explicit approval.
- Never delete a file. Archiving is the only removal mechanism.
- Restrict this skill to the folder the user names, and never to `~/Documents` or `~/Brain`.
- Read `references/para-framework.md` before classifying anything — it is the source of truth for
  category definitions.
- Never create an empty folder, and never leave a file loose at the root of a PARA category folder.
- Never rename the user's files or folders unless they ask.
- Report judgment calls and unplaced items explicitly after execution.

## References

- [references/para-framework.md](references/para-framework.md) — **READ FIRST**: precise category
  definitions, distinction guidelines, and classification logic
- [references/workflow.md](references/workflow.md) — the four execution phases in full detail
  (Discover, Present the Plan, Execute, Output Inventory)
- [references/cross-platform-guide.md](references/cross-platform-guide.md) — mirroring PARA across
  other platforms (cloud drives, note apps)
