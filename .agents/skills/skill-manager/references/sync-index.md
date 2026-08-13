# Sync README Index

## Purpose

Rebuild `.agents/skills/README.md` deterministically from skill frontmatter. The README is a derived
index, never an independent source of metadata.

## Procedure

1. Enumerate every immediate directory under `.agents/skills/` and require a `SKILL.md`.
2. Read `name`, the complete folded `description`, and `metadata.category`.
3. Normalize folded-description whitespace to one ASCII space.
4. Use the first sentence, ending at the first period followed by whitespace or end-of-string.
   Preserve the period and do not apply an arbitrary character limit.
5. Escape a Markdown table pipe in the extracted sentence.
6. Group entries in `Dev`, `Support`, `Product`, `Ops`, then `Uncategorized` order.
7. Omit empty sections and sort slugs bytewise within every section.
8. Rewrite the complete README from the template below.
9. Format the README with Prettier.
10. Report added, removed, moved, and invalid entries.
11. Run the procedure again and require byte-identical output.

Invalid or missing `metadata.category` places a skill in `Uncategorized` and remains a doctor FAIL;
index generation must not hide it.

## Template

```markdown
# Shared Skills

This directory is the single source of truth for reusable agent skills.

## Conventions

- One skill per subdirectory.
- Each skill must include a `SKILL.md` file.
- Optional folders: `scripts/`, `references/`, `assets/`, `evals/`.
- Manage skills with `/skill-manager`.

## Dev

| Skill           | Description               |
| --------------- | ------------------------- |
| `another-skill` | <derived first sentence>. |
| `example-skill` | <derived first sentence>. |
```

Repeat the table for each non-empty category. Prettier owns table column widths, so generation must
not calculate or preserve manual padding.

## Idempotence check

After the first generation and formatting:

```bash
shasum -a 256 .agents/skills/README.md > /tmp/skills-index.sha256
```

Run sync again, then:

```bash
shasum -a 256 -c /tmp/skills-index.sha256
```

The check must pass. Identical frontmatter input must always produce identical README bytes.

## Constraints

- Never edit README table rows manually.
- Always read `metadata.category`, never a top-level category.
- Never truncate descriptions at an approximate character count.
- Never omit an invalid skill; index it under `Uncategorized` and report the finding.
- Never preserve orphaned rows whose directory no longer exists.
- Always verify a second generation is byte-identical.
