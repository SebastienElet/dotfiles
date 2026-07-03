# Skill Evals — Trigger-Queries (Phase 4)

**Status**: This file is a stub prepared for Phase 4 (Descriptions, Evals & Validation).

## Overview

Skills can include `evals/trigger-queries.json` to define activation scenarios. This helps validate
that:

1. The skill description correctly triggers in real-world queries
2. The skill doesn't false-positive on unrelated queries
3. The skill is discoverable across all 5 agents

## When to Create Evals

Evals are **recommended for critical skills** (dev, support, or ops workflows that many developers
rely on).

**Skills that should have evals**:

- `skill-manager` — foundational tooling
- `git-commit`, `pr-create` — daily workflows
- `testing` — testing patterns
- `support-*` — customer-facing diagnostics
- `pothos-migration`, `prisma` — critical patterns

**Skills that may skip evals**:

- Niche domain-specific skills (less frequently used)
- Experimental skills (under development)

## Format — `trigger-queries.json`

```json
{
  "skill": "skill-slug",
  "version": "1.0",
  "queries": [
    {
      "query": "I need to create a new skill for X",
      "should_activate": true,
      "reason": "Keywords: 'create', 'new skill'; matches description 'create new skills with proper scaffolding'"
    },
    {
      "query": "How do I use skill-manager?",
      "should_activate": true,
      "reason": "Implicit: help on skill-manager; matches 'managing .agents/skills'"
    },
    {
      "query": "What's the best TypeScript pattern?",
      "should_activate": false,
      "reason": "No mention of skills or skill-manager; out of scope"
    },
    {
      "query": "I want to check all skills in my repo",
      "should_activate": true,
      "reason": "Keywords: 'doctor', 'skills'; matches description 'run doctor checks on existing skills for quality'"
    },
    {
      "query": "Can you review my code?",
      "should_activate": false,
      "reason": "Generic review request; doesn't mention skills. (Use the 'review' skill instead.)"
    }
  ]
}
```

### Query Fields

- **`query`** (string): user input that should or shouldn't trigger the skill
- **`should_activate`** (boolean): expected outcome (true = should trigger, false = should not)
- **`reason`** (string): why this query should/shouldn't activate (for debugging)

## File Location

Place `trigger-queries.json` at:

```text
.agents/skills/<slug>/evals/trigger-queries.json
```

Optionally, add an `evals.json` file for more detailed test cases (Phase 4).

## How Evals Are Used (Phase 4)

In Phase 4, the review agent will:

1. Load each skill's `trigger-queries.json`
2. Simulate the queries with each of the 5 agents' activation mechanisms
3. Report which queries succeed / fail
4. Suggest description rewrites if activation rates are low

## Writing Good Trigger Queries

- **Include common use cases** — queries that developers would actually type
- **Include edge cases** — implicit needs that don't name the domain (e.g., "How do I doctor?"
  instead of only "How do I doctor skills?")
- **Include false-positives** — queries that should NOT trigger (helps ensure specificity)
- **Front-load keywords** — if a query starts with key terms, it's more likely to trigger
- **Be realistic** — use phrasing that matches how people actually ask questions, not how the
  documentation phrases things

## Examples by Skill

### Example: `skill-manager`

```json
{
  "skill": "skill-manager",
  "version": "1.0",
  "queries": [
    {
      "query": "I need to create a new skill",
      "should_activate": true,
      "reason": "'create', 'new skill'"
    },
    {
      "query": "How do I scaffold a skill?",
      "should_activate": true,
      "reason": "'scaffold' + 'skill'"
    },
    {
      "query": "Doctor all skills for quality",
      "should_activate": true,
      "reason": "'doctor', 'skills', 'quality'"
    },
    {
      "query": "Update the skills README",
      "should_activate": true,
      "reason": "'update', 'skills README' → matches sync-index operation"
    },
    {
      "query": "How do I write tests?",
      "should_activate": false,
      "reason": "Testing is a different domain; use the 'testing' skill"
    },
    {
      "query": "What categories can I use for my skill?",
      "should_activate": true,
      "reason": "Question about skill structure → skill-manager knowledge"
    }
  ]
}
```

### Example: `pr-create` (hypothetical future skill)

```json
{
  "skill": "pr-create",
  "version": "1.0",
  "queries": [
    {
      "query": "Create a pull request",
      "should_activate": true,
      "reason": "Direct: 'create' + 'pull request'"
    },
    {
      "query": "How do I open a PR?",
      "should_activate": true,
      "reason": "Implicit: 'PR' = pull request"
    },
    {
      "query": "I need to push my changes",
      "should_activate": false,
      "reason": "Push is a git operation; use 'git' skill, not 'pr-create'"
    },
    {
      "query": "What's the PR title format?",
      "should_activate": true,
      "reason": "Implicit: PR workflow question"
    }
  ]
}
```

## Next Steps (Phase 4)

1. For critical skills, author `evals/trigger-queries.json`
2. Run validation across Cursor, Claude, Codex, Gemini CLI, Copilot
3. Collect activation metrics (% of queries that correctly trigger)
4. Refine descriptions based on results
5. Document findings in Phase 4 exit state

---

**Phase 4 will detail**:

- Multi-agent eval framework
- Collecting metrics on description quality
- Automated description refinement suggestions
- Final validation checklist before graduation
