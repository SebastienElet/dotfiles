# Skill Conventions

This file is the SSOT for skill structure. All skills must conform to these rules.

**Status**: Aligned with [agentskills.io specification](https://agentskills.io/specification),
[Anthropic Complete Guide](https://resources.anthropic.com/hubfs/The-Complete-Guide-to-Building-Skill-for-Claude.pdf),
OpenAI Codex, Google Gemini CLI, and GitHub Copilot standards.

**Compatibility**: Skills written to these conventions work with all 5 agents:

- Cursor 2.4+
- Claude Code / Claude Projects
- OpenAI Codex
- Google Gemini CLI
- GitHub Copilot

---

## 1. Progressive Disclosure — 3 Levels

Skills are structured to reveal information progressively:

1. **Metadata** (~100 tokens)

   - `name` + `description` → always visible to the agent
   - The description is the primary trigger mechanism across all 5 agents
   - Must be "pushy" and include activation keywords (see below)

2. **Instructions** (< 5000 tokens)

   - SKILL.md body (required sections)
   - Loaded when the agent activates the skill
   - Include `## Gotchas` section for project-specific warnings

3. **Resources** (unlimited)
   - `references/`, `scripts/`, `assets/`, `evals/` directories
   - Loaded on-demand when a step references them
   - Keep SKILL.md lean; move rich content here

---

## 2. Frontmatter — Standard Format (agentskills.io)

```yaml
---
name: <slug> # required, kebab-case, matches directory
description: > # required, < 1024 chars, pushy format (see below)
  [What the skill does]. Use when [conditions]. Make sure to use this skill whenever [keywords +
  implicit cases], even if [where user might not name the domain].
license: MIT # optional
compatibility: <text> # optional, environment dependencies
metadata: # optional per agentskills.io spec; required for Bellman skills
  category: dev | support | product | ops # required (Bellman internal — used for README indexing)
  author: Bellman
  version: "1.0"
---
```

### Portability Rules (Critical)

❌ **Do NOT use custom fields at top-level**:

- No `disable-model-invocation: true` (Cursor-only, breaks portability)
- No `category` at top-level (move to `metadata.category`)
- No `paths` (Cursor-only; routing via `.cursor/rules/*.mdc` globs instead)

✅ **Allowed fields** (only these):

- `name` (required)
- `description` (required)
- `license` (optional)
- `compatibility` (optional)
- `metadata` (optional, object, unlimited fields inside)

### `description` Format — "Pushy"

The description is the **single trigger mechanism** across all 5 agents. Make it count:

**Pattern**:

```text
<What the skill does>. Use when <conditions/triggers>.
Make sure to use this skill whenever <keywords + implicit cases>,
even if <where user might not mention the domain>.
```

**Good example**:

> Diagnose discrepancies between landlord charges in Lobby and accounting records. Use when charges
> don't match or reconciliation fails. Make sure to use this skill whenever investigating charge
> discrepancies, charge balance issues, or missing invoices, even if the user doesn't explicitly
> mention "accounting" or "reconciliation".

**Bad examples**:

- ❌ Generic: `"Helps manage skills"` (no activation keywords)
- ❌ Passive: `"For creating and managing agent skills"` (no "Use when", no "Make sure")
- ❌ Explicit trigger list: `"Trigger terms: term1, term2"` (hardcoded list; use pushy format
  instead — keywords integrate naturally)

### `metadata.category` — Internal Convention

Categories are **not** in the agentskills.io spec; they're Bellman-internal for README organization
and filtering.

| Category  | Covers                                                              |
| --------- | ------------------------------------------------------------------- |
| `dev`     | git, PR, tests, code quality, migrations, architecture, performance |
| `support` | customer tickets, diagnostics, client responses, troubleshooting    |
| `product` | issue creation, feature scoping, roadmap, planning                  |
| `ops`     | CLI commands, infra, tooling, configuration, agent management       |

---

## 3. Directory Structure

Each skill lives in its own subdirectory under `.agents/skills/`:

```text
.agents/skills/<slug>/
  SKILL.md              ← required, < 500 lines, < 5000 tokens
  references/           ← optional, progressive disclosure (detailed guides, domain docs, templates)
    detailed-guide.md
    edge-cases.md
  scripts/              ← optional, reusable scripts (bash, etc.)
  assets/               ← optional, templates, images, data files
  evals/                ← optional, trigger-queries.json + evals.json (see section 7)
```

### Naming `references/` files — `<topic>-<scope>.md`

When a skill's behaviour varies by **package or application**, split `references/` files using the
`<topic>-<scope>.md` convention instead of a single monolithic file.

**When to split**:

- Rules differ structurally between two targets (e.g. different Prisma client, different seed
  helpers, different context types).
- An agent working in scope A would otherwise have to read and filter out scope B content.

**Naming**:

- `<topic>` = what the file covers: `unit`, `integration`, `mocking`, `seeds`, `context`, etc.
- `<scope>` = package or app slug: `pm` (property-management), `lm` (lease-management),
  `api-graphql`, `lobby`, `common`, etc.
- Example: `integration-pm.md`, `integration-lm.md`, `react-testing.md`, `mocking.md`

**When NOT to split** — keep a single file when:

- Rules are identical across all scopes (e.g. unit test structure is the same everywhere).
- The divergence is a single paragraph that can be gated with a heading.

**Routing in SKILL.md** — the `## Steps` section must tell the agent _which_ file to load based on
context:

```markdown
## Steps

1. Identify the test type (unit / integration / …).
2. Identify the target package:
   - `services/api-graphql/src/service/**` or `packages/prisma-pm/**` → read
     `references/integration-pm.md`
   - `services/api-graphql/src/lease-management/**` or `packages/prisma-lm/**` → read
     `references/integration-lm.md`
   - `services/lobby/**` → read `references/react-testing.md`
3. Follow the steps in that file.
```

> ⚠️ The paths above (`services/api-graphql/`, `packages/prisma-pm/`, `services/lobby/`, etc.) are
> **Bellman-specific examples**. Adapt them to your project's directory structure when applying this
> pattern in another codebase.

This "conditional routing" pattern ensures the agent loads **only** the relevant reference — not the
entire skill.

### `SKILL.md` Size Constraints

- **< 500 lines** — if longer, move content to `references/`
- **< 5000 tokens** — tight enough to fit in a context window with 15 other active skills
- Trim examples to essentials; link to `references/` for full docs

---

## 4. Required Sections in SKILL.md

Every SKILL.md must contain these sections in this exact order:

1. **Frontmatter** (YAML, top of file)
2. **`# <Title>`** — H1, human-readable name (e.g., "Skill Manager", "PR Create")
3. **`## Overview`** — 2–4 sentences: purpose + when to use
4. **`## Usage`** — invocation syntax with examples
5. **`## Steps`** — numbered, actionable procedure (or `## Workflow` if the skill is a workflow)
6. **`## Gotchas`** — project-specific warnings, edge cases, anti-patterns (**NEW, MANDATORY**)
7. **`## Constraints`** — hard must/must-not rules (minimum 3)

Optional:

- **`## References`** — navigation to `references/` files (if skill has progressive disclosure)
- **`## Examples`** — concrete end-to-end walkthroughs (if not covered in `## Steps`)

---

## 5. Writing Principles (Anthropic + OpenAI + Google + GitHub)

### Explain the Why

- Justify rules with reasoning, not just mandates
- Example: ✅ "Always read `conventions.md` first — it's the SSOT for structure; skipping it causes
  inconsistencies" vs. ❌ "Always read `conventions.md` first"

### Aim for Moderate Detail

- Don't cover every edge case; assume the user will ask follow-ups
- Keep explanations focused on the most common paths

### Provide Defaults, Not Menus

- Recommend **one** way, not multiple options
- If there are valid alternatives, explain when each applies (e.g., "Use Jest for `.service` files;
  Playwright for e2e")

### Favor Procedures Over Declarations

- Steps > abstract principles
- Imperative form: "Run X, then check Y" vs. "X should be run after Y"

### Bundle Repeated Scripts

- If a script appears in 2+ steps, move it to `scripts/`
- Reference it: "See `scripts/lint.sh` in this skill"

### Generalize From Examples, Don't Overfit

- Show the pattern, not every variation
- Let the user extrapolate to their case

### Front-Load Key Terms

- In `description` and `## Overview`, put the most important activation keywords first
- OpenAI/Codex note: descriptions may be truncated → essential info comes first

### Keep Each Skill Focused

- One job per skill (OpenAI best practice)
- If a skill does 3 things, split it into 3 skills + a meta-skill that links them

---

## 6. Gotchas Section (NEW)

Every skill must have a `## Gotchas` section listing project-specific pitfalls:

**Purpose**: Help users avoid common mistakes that are obvious to maintainers but not to users.

**Format**: Bulleted list, cause + fix

**Examples**:

```markdown
## Gotchas

- **Forgetting to doctor before syncing** — if skills look inconsistent, the index will capture the
  broken state. Always run `doctor` first, fix issues, then `sync-index`.
- **Modifying frontmatter custom fields** — Cursor won't recognize custom fields like
  `disable-model-invocation` after Phase 1. Use only standard agentskills.io fields.
- **Missing trigger terms in description** — if your description is generic, other agents won't
  activate the skill. Use the pushy format with specific keywords.
- **Over-long SKILL.md** — if a skill's SKILL.md exceeds 500 lines, move content to `references/`
  and reference it in a `## References` section.
```

---

## 7. Optional: Evals (Trigger-Queries)

For critical skills, add `.agents/skills/<slug>/evals/trigger-queries.json` to define activation
scenarios.

**Format**:

```json
{
  "skill": "<slug>",
  "version": "1.0",
  "queries": [
    {
      "query": "I need to create a new skill for X",
      "should_activate": true,
      "reason": "Keywords: 'create', 'new skill'"
    },
    {
      "query": "How do I use skill-manager?",
      "should_activate": true,
      "reason": "Implied: help on skill-manager"
    },
    {
      "query": "What's the best TypeScript pattern?",
      "should_activate": false,
      "reason": "Doesn't mention skills; out of scope"
    }
  ]
}
```

See `references/evals.md` in this skill for full details. Evals are validated in Phase 4.

---

## 8. Naming Conventions

- **Slug**: `kebab-case`, verb-noun or domain-noun
  - ✅ `git-commit`, `pr-create`, `support-charges-discrepancy`, `test-unit-create`
  - ❌ `helper`, `misc`, `tools`, `utils` (too generic)
  - ❌ `dev-git-commit` (don't repeat category in slug)

---

## 9. README Index Maintenance

After any create, rename, or delete operation, run `/skill-manager sync-index` to rebuild
`.agents/skills/README.md`.

The README uses `metadata.category` to organize skills into sections:

```markdown
## Dev

| Skill        | Description |
| ------------ | ----------- |
| `git-commit` | ...         |
| `pr-create`  | ...         |

## Support

| Skill                         | Description |
| ----------------------------- | ----------- |
| `support-charges-discrepancy` | ...         |
```

---

## 10. Checklist: Before Committing a Skill

- [ ] Frontmatter has `name`, `description`, `license` (optional), `compatibility` (optional),
      `metadata` (required for Bellman skills — must include at least `metadata.category`)
- [ ] No custom fields at top-level (no `disable-model-invocation`, `category`, `paths`)
- [ ] `metadata.category` is one of: `dev`, `support`, `product`, `ops`
- [ ] Description is "pushy" (includes "Use when" and/or "Make sure to use")
- [ ] Description is < 1024 characters
- [ ] SKILL.md is < 500 lines, < 5000 tokens
- [ ] All 7 required sections present (Overview, Usage, Steps, Gotchas, Constraints, + H1 +
      frontmatter)
- [ ] `## Gotchas` includes 3+ project-specific warnings
- [ ] `## Constraints` has 3+ hard must/must-not rules
- [ ] If SKILL.md is long, content is in `references/` with links in `## References`
- [ ] Skill is listed in `.agents/skills/README.md` under the correct category
- [ ] Slug matches directory name (kebab-case, no category prefix)
