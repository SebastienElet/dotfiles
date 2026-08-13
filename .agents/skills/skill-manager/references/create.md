# Create a Skill

## Inputs

Gather before writing:

- one-sentence purpose;
- concrete explicit and implicit triggers;
- inputs, outputs, and hard constraints;
- local category: `dev`, `support`, `product`, or `ops`;
- required `agents/`, `references/`, `scripts/`, `assets/`, or `evals/` resources;
- scopes whose behavior genuinely differs.

Ask one question at a time when an unknown answer changes behavior. Do not invent domain rules.

## Procedure

1. Read `conventions.md` completely.
2. Normalize the requested slug and verify it is 1–64 lowercase letters, digits, or hyphens, with
   no leading, trailing, or consecutive hyphen.
3. Confirm `.agents/skills/<slug>/` does not already exist. Never overwrite it.
4. Create `.agents/skills/<slug>/` and only the resource directories established by the inputs:
   `agents/`, `references/`, `scripts/`, `assets/`, or `evals/`.
5. Write `SKILL.md` from the minimal template below.
6. Add optional frontmatter fields only when they have a real valid value.
7. Put executable shell with positional argument placeholders in `scripts/`, never in `SKILL.md`.
8. Route scoped sibling references from `## Steps` when behavior differs by scope.
9. Validate standard rules with `skills-ref` when available, then run local doctor.
10. Validate any eval JSON, run its scenarios when required, and run `sync-index` twice.

## Minimal template

```markdown
---
name: <slug>
description: >
  <specific case first>. Use when <concrete conditions>. Make sure to use this skill whenever
  <implicit cases>, even if <the user does not name the domain>.
metadata:
  category: dev | support | product | ops
---

# <Title>

## Overview

<Purpose, boundary, and core principle in two to four sentences.>

## Usage

<Invocation syntax and at least one realistic example.>

## Steps

1. <First complete action.>
2. <Second complete action.>
3. <Verification action.>

## Gotchas

- **<Specific cause>** — <consequence and correction>.
- **<Specific cause>** — <consequence and correction>.
- **<Specific cause>** — <consequence and correction>.

## Constraints

- <Hard must or must-not rule.>
- <Hard must or must-not rule.>
- <Hard must or must-not rule.>
```

Optional `license`, `compatibility`, and `allowed-tools` fields go before `metadata` and are omitted
unless established. `allowed-tools` is experimental and must be a space-separated string.

## Validation

If `skills-ref` is available:

```bash
skills-ref validate ./.agents/skills/<slug>
```

Otherwise report standard validation unavailable and continue doctor. Creation is complete only
when doctor passes, eval JSON passes when present, the skill is indexed, and a second `sync-index`
run changes no byte.

## Constraints

- Never overwrite an existing skill.
- Never create `rules/` or an unused resource directory.
- Never emit an empty optional frontmatter field.
- Never install validation tooling implicitly.
- Never add an activation router without repeated behavioral evidence.
- Always run doctor and deterministic `sync-index` before completion.
