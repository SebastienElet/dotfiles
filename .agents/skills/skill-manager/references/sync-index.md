# Sync README Index

## Purpose

Keep `.agents/skills/README.md` as the authoritative, always-up-to-date index of all skills. Run
after any create, rename, or delete operation.

## Steps

1. **Scan all skill directories**

   ```bash
   ls .agents/skills/
   ```

   Exclude `README.md`. Each subdirectory is a skill.

2. **Read frontmatter from each `SKILL.md`**

   - Extract: `name`, `description`, `category`
   - Flag skills with missing `category` as `uncategorized`

3. **Group skills by category** (alphabetical within each group)

   - `dev` — development workflow
   - `support` — customer support
   - `product` — product / feature work
   - `ops` — infrastructure / tooling
   - `uncategorized` — missing or invalid `category` (requires follow-up)

4. **Rewrite `.agents/skills/README.md`** using the template below.

5. **Report changes**
   - Skills added to index
   - Skills removed from index (directory deleted)
   - Skills with missing/invalid frontmatter → list as action items

---

## README template

```markdown
# Shared Skills

This directory is the single source of truth for reusable agent skills.

## Conventions

- One skill per subdirectory.
- Each skill must include a `SKILL.md` file.
- Optional folders: `scripts/`, `references/`, `assets/`.
- Manage skills with `/skill-manager`.

## Dev

| Skill                         | Description |
| ----------------------------- | ----------- |
| `api-graphql-lobby-hexagonal` | ...         |
| `git`                         | ...         |
| `localize-graphql-hooks`      | ...         |
| `monorepo-package-creation`   | ...         |
| `pothos-migration`            | ...         |
| `prisma`                      | ...         |
| `review`                      | ...         |
| `testing`                     | ...         |

## Support

| Skill                         | Description |
| ----------------------------- | ----------- |
| `property-management-support` | ...         |

## Product

| Skill                 | Description |
| --------------------- | ----------- |
| `product-draft-issue` | ...         |

## Ops

| Skill           | Description |
| --------------- | ----------- |
| `cli-commands`  | ...         |
| `skill-manager` | ...         |
```

- Fill each `...` with the skill’s `description` from its `SKILL.md` frontmatter (truncate to ~100
  chars).
- Sort skills alphabetically within each category.
- If any skills have missing `category`, add an `## Uncategorized` section and list them with a note
  to fix their frontmatter.

---

## Rules

- **Do not manually edit** the skill table rows in README — always re-run sync-index.
- **Truncate descriptions** to ~100 characters in the table for readability.
- **Orphaned entries** (in README but no directory) must be removed.
- **Uncategorized skills** must be listed but also flagged as requiring a fix.
