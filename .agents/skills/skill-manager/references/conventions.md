# Skill Conventions

This file is the source of truth for every skill under `.agents/skills/`. It separates the
agentskills.io standard from this repository's stricter local quality and indexing rules.

## 1. Progressive disclosure

Skills expose three levels of context:

1. Frontmatter `name` and `description`, always available for discovery.
2. `SKILL.md`, loaded when the skill activates and kept below 500 lines.
3. `references/`, `scripts/`, `assets/`, and `evals/`, loaded or executed only when needed.

Keep core procedure and routing in `SKILL.md`. Move detailed variants, large tables, reusable
commands, fixtures, and assets to the appropriate resource directory.

## 2. Standard frontmatter

The agentskills.io standard allows exactly six top-level fields:

```yaml
---
name: example-skill
description: >
  Handle a specific reusable workflow. Use when its concrete trigger occurs. Make sure to use this
  skill whenever the implicit case appears, even if the user does not name the workflow.
license: MIT
compatibility: Requires a POSIX shell.
allowed-tools: Read Grep
metadata:
  category: ops
  version: "1.0"
---
```

Only `name` and `description` are required by the standard. This repository additionally requires
`metadata.category` for its index.

### Field constraints

| Field           | Type   | Constraint                                                                                                                  |
| --------------- | ------ | --------------------------------------------------------------------------------------------------------------------------- |
| `name`          | string | 1–64 lowercase ASCII letters, digits, and hyphens; no leading, trailing, or consecutive hyphen; equals the parent directory |
| `description`   | string | non-empty, at most 1024 characters; local target below 400 characters                                                       |
| `license`       | string | optional license name or reference                                                                                          |
| `compatibility` | string | optional, non-empty when present, at most 500 characters                                                                    |
| `allowed-tools` | string | optional space-separated pre-approved tools; experimental support varies by host                                            |
| `metadata`      | map    | optional string keys to string values; quote version-like values                                                            |

Fields such as `disable-model-invocation`, `user-invocable`, `paths`, `argument-hint`, `model`,
`context`, `hooks`, and top-level `category` are host-specific or non-standard. Remove them unless
the skill intentionally abandons portability, which requires explicit user approval and a revised
local convention.

Do not move an unknown field under `metadata` automatically. Preserve it there only when its meaning
is scalar metadata, its value is a string, and the user approves the semantic change.

### Description format

Descriptions determine discovery. Put the distinguishing case first and use this local pattern:

```text
<What it handles>. Use when <specific conditions>. Make sure to use this skill whenever <implicit
cases>, even if <the user does not name the domain>.
```

Aim below 400 characters because host skill lists truncate descriptions or omit entries before the
standard's 1024-character ceiling. Avoid generic descriptions, passive labels, and appended trigger
keyword dumps.

## 3. Canonical directory and adapters

Every project skill physically lives at:

```text
.agents/skills/<slug>/
  SKILL.md
  references/   optional detailed guidance
  scripts/      optional executable reusable logic
  assets/       optional output resources
  evals/        optional activation scenarios
```

Do not create a local `rules/` directory. Add only resource directories the skill actually needs.

The repository exposes the canonical directory through three relative symlinks:

| Consumer                    | Project path      | Repository state               |
| --------------------------- | ----------------- | ------------------------------ |
| Codex and compatible agents | `.agents/skills/` | canonical directory            |
| Claude Code                 | `.claude/skills`  | symlink to `../.agents/skills` |
| Cursor                      | `.cursor/skills`  | symlink to `../.agents/skills` |
| Codex compatibility adapter | `.codex/skills`   | symlink to `../.agents/skills` |

Never reverse these links, create parallel copies, or edit through multiple paths as if they were
independent. A project skill stays self-contained in the repository; a required external skill is
copied and becomes the repository's own version rather than linked outside the checkout.

## 4. Reference segmentation

Split reference files by scope only when behavior differs materially between packages or
applications. Use `<topic>-<scope>.md`, for example `integration-api.md` and
`integration-worker.md`. Keep identical rules in one shared reference.

When scoped siblings exist, route them explicitly from `SKILL.md`:

```markdown
1. Identify the target.
2. For `services/api/**`, read `references/integration-api.md`.
3. For `services/worker/**`, read `references/integration-worker.md`.
4. Follow the selected procedure.
```

A two-word filename without a same-topic sibling is an ordinary reference, not a scoped file.

## 5. Local body convention

The agentskills.io standard does not prescribe body sections. This repository requires these in
order:

1. Frontmatter
2. One H1 title
3. `## Overview`
4. `## Usage`
5. `## Steps` or `## Workflow`
6. `## Gotchas`
7. `## Constraints`

`## References` and examples are optional. `Gotchas` and `Constraints` each need at least three
concrete entries. This is a local quality barrier, not a portability claim.

## 6. Writing principles

- Explain non-obvious reasons inside a skill, where the procedure loads on demand.
- Prefer one default and state when an alternative applies.
- Write numbered actions instead of abstract declarations.
- Keep one job per skill.
- Show one strong neutral example instead of several organization-specific variants.
- Front-load search terms in the description and overview.
- Put repeated or deterministic executable logic in `scripts/`.
- Write all skill and reference content in English.

## 7. Positional shell placeholders

Some hosts template a `SKILL.md` body as a slash command before an agent reads it. Literal shell
tokens such as `$0` through `$9`, `$@`, and `$ARGUMENTS` can therefore be replaced by invocation
arguments. The rewritten command may still run and silently produce the wrong result.

Apply these rules:

- Put executable shell containing positional argument placeholders in `scripts/`.
- Let `SKILL.md` invoke the script without reproducing its internals.
- Escape a literal token in explanatory prose with one backslash immediately before the dollar
  sign.
- Do not double the backslash.
- References and scripts are read as files and are not body-templated.
- Treat named host substitutions as host-specific unless the standard adopts them.

Doctor searches `SKILL.md` bodies for unescaped positional placeholders. A match is a local FAIL
unless it is proven non-executable prose and correctly escaped.

## 8. Gotchas

Every skill lists at least three repository-specific failure modes. Each entry names the cause, its
consequence, and the correction. Avoid generic advice that a capable agent already knows.

```markdown
## Gotchas

- **Editing an adapter** — parallel copies diverge. Edit `.agents/skills/` only.
- **Skipping a failed probe** — the audit becomes a false PASS. Report the unavailable check.
- **Using a vague trigger** — the skill never loads. Name the concrete situation first.
```

## 9. Activation routing

The description is the default router. An absent external router rule is never a doctor or
cross-check finding.

Add a router rule only when identical realistic prompts, run at least three times without the rule,
reproduce a missed or wrong activation. Preserve the prompts and results, add the smallest
relational rule that distinguishes sibling skills, then rerun the same prompts. A rule that exists
only to prevent a broken skill from activating is evidence that the skill should be removed.

## 10. Evals

Critical skills may store `evals/trigger-queries.json`. When present, doctor requires:

- `skill`: string equal to the directory slug;
- `version`: string;
- `queries`: non-empty array;
- every query has string `query`, boolean `should_activate`, and string `reason`;
- at least one positive and one negative case.

The repository provides no automatic multi-agent eval runner. Scenario files are evidence only when
their prompts were actually executed and results recorded outside the repository or in an approved
design document.

## 11. README index

`.agents/skills/README.md` is derived from `name`, folded `description`, and
`metadata.category`. Never edit table rows manually. Run `sync-index` after create, fix, rename, or
delete, and require a second run to produce byte-identical output.

Categories are local indexing metadata:

| Category  | Scope                                                  |
| --------- | ------------------------------------------------------ |
| `dev`     | development workflow, tests, review, architecture      |
| `support` | customer or user support workflows                     |
| `product` | product and feature planning                           |
| `ops`     | tools, configuration, infrastructure, agent management |

## 12. Validation checklist

When `skills-ref` exists, run:

```bash
skills-ref validate ./.agents/skills/<slug>
```

When absent, report `Standard validation: unavailable (skills-ref not installed)`, continue local
checks, and never claim standard validation succeeded. Do not install it implicitly.

Before declaring a skill clean, verify:

- standard field names, types, and limits;
- local category and directory-name match;
- description pattern and local length target;
- required sections in order;
- at least three gotchas and constraints;
- fewer than 500 `SKILL.md` lines or appropriate progressive disclosure;
- no executable positional placeholders in the body;
- valid eval JSON when present;
- README membership and deterministic regeneration;
- adapter symlinks still point to `../.agents/skills`;
- no unapproved activation router rule.
