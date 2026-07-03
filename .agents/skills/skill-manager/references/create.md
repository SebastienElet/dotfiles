# Create a New Skill

## Steps

1. **Gather requirements** (ask the user if unclear)

   - What is the skill's purpose? (1 sentence)
   - What triggers it? (slash command name, domain keywords, situations)
   - What are the inputs and expected outputs?
   - What hard rules must it enforce?
   - What category: `dev` / `support` / `product` / `ops`?
   - Does it need reference files (SQL, templates, markdown)?
   - **Do the rules vary by package or application?** (e.g. different behaviour for `api-graphql` vs
     `lobby` vs `lease-management`) → if yes, plan `<topic>-<scope>.md` files and conditional
     routing in `## Steps` (see `conventions.md § 3`).

2. **Choose slug**

   - `kebab-case`, verb-noun or domain-noun
   - Check for conflicts:

     ```bash
     ls .agents/skills/
     ```

3. **Scaffold directory**

   ```bash
   mkdir -p .agents/skills/<slug>
   # If rules needed (progressive disclosure):
   mkdir -p .agents/skills/<slug>/rules
   # If references needed:
   mkdir -p .agents/skills/<slug>/references
   ```

4. **Write SKILL.md** using the template below

   - Fill all required sections (including `## Gotchas`)
   - Write description in pushy format (not ending with "Trigger terms: ...")
   - Keep steps actionable and numbered
   - Keep SKILL.md < 500 lines (move rich content to `references/`)

5. **Sync README**
   - Follow `references/sync-index.md` to add the skill to `.agents/skills/README.md`

## SKILL.md Template

```markdown
---
name: <slug>
description: >
  [What the skill does]. Use when [conditions]. Make sure to use this skill whenever [keywords +
  implicit cases], even if [edge case where user might not mention the domain].
license: MIT
compatibility: <optional; environment dependencies>
metadata:
  category: dev | support | product | ops
  author: Bellman
---

# <Title>

## Overview

<2–4 sentences on purpose and when to use.>

## Usage
```

/<slug> [options]

```text

- `-x` / `--option` (optional): description (default: `value`)
- Examples:
  - `/<slug>` → default behavior
  - `/<slug> --option value` → behavior with option

## Steps

1. **Step one**
   - Sub-action
   - Sub-action

2. **Step two**
   - Sub-action

## Gotchas

- **First gotcha** — cause + how to fix
- **Second gotcha** — cause + how to fix
- **Third gotcha** — cause + how to fix

## Constraints

- **Constraint 1**: ...
- **Constraint 2**: ...
- **Constraint 3**: ...

## References (if using progressive disclosure)

For detailed guidance on specific topics, see:

- [references/topic-1.md](references/topic-1.md)
- [references/topic-2.md](references/topic-2.md)

## Example Workflow (optional)

```

User: /<slug>

Agent:

1. ...
2. ...
3. ...

```text

## Quality Checklist Before Saving

See [conventions.md § 10](conventions.md#10-checklist-before-committing-a-skill) for the canonical checklist (SSOT — do not duplicate here).
```
