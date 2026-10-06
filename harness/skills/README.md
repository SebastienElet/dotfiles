# User Skills

This directory is the canonical source for user-scoped agent skills.

## Conventions

- One skill per subdirectory.
- Each skill must include a `SKILL.md` file.
- Optional folders: `agents/`, `scripts/`, `references/`, `assets/`, `evals/`.
- Manage skills with `/skill-manager`.

## Dev

| Skill                           | Description                                                                                          |
| ------------------------------- | ---------------------------------------------------------------------------------------------------- |
| `agent-instructions`            | Maintain coding-agent instructions and their discovery paths.                                        |
| `claude-developer`              | Prepare manual implementation and correction prompts for Claude Code without invoking it.            |
| `code-enforcement`              | Write code whose purpose is to refuse: hook, guard, validator, permission check, lint rule, CI gate. |
| `code-review`                   | Review a local diff for standards and requirement compliance.                                        |
| `code-simplify`                 | Simplify code to reduce understanding and maintenance costs.                                         |
| `codebase-design`               | Design cohesive modules with simple public interfaces.                                               |
| `design-claim-audit`            | Audit architectural and domain guarantees.                                                           |
| `diagnosing-bugs`               | Diagnose bugs and performance regressions.                                                           |
| `harness-reflection`            | Turn repeated agent failures into evidence-backed harness improvements.                              |
| `improve-codebase-architecture` | Survey architectural friction and present improvement candidates.                                    |
| `issue-creation`                | Draft, validate, review, and publish tracker issues across forges.                                   |
| `linear-start`                  | Start or resume implementation of an assigned Linear issue in a Bitbucket repository.                |
| `linear-sync`                   | Reconcile assigned Linear issues with Bitbucket pull-request reality without reviewing code.         |
| `linear-workflow`               | Apply the shared Linear and Bitbucket work invariants.                                               |
| `pr-feedback`                   | Collect evidence-backed review feedback and reviewer-authored fixes from merged pull requests.       |
| `pr-fix`                        | Repair an open pull request after an independent merge review.                                       |
| `pr-verdict`                    | Deliver a PR verdict on an open pull request, yours or another author's.                             |
| `proof-integrity-review`        | Review changes to verification mechanisms.                                                           |
| `requirements-clarification`    | Clarify requirements before implementation.                                                          |
| `tdd`                           | Develop owned behavior with test-driven development.                                                 |
| `visual-explanation`            | Explain relationships, diagnostics and changes visually.                                             |

## Product

| Skill               | Description                                                                          |
| ------------------- | ------------------------------------------------------------------------------------ |
| `grill-me`          | Challenge an idea through a requested design interview.                              |
| `grilling`          | Run a design interview by dependent decisions.                                       |
| `issue-simplify`    | Simplify GitHub or Linear issues and drafts.                                         |
| `linear-issue-spec` | Prepare implementation-ready Linear development issues as functional specifications. |
| `to-spec`           | Synthesize the current conversation into a functional specification.                 |
| `wayfinder`         | Map a large effort as dependent decisions across sessions.                           |

## Ops

| Skill                     | Description                                                                                               |
| ------------------------- | --------------------------------------------------------------------------------------------------------- |
| `disaster-recovery-plan`  | Fill a disaster recovery plan (PRA) from a Word template using repository evidence and operational facts. |
| `handoff`                 | Hand the current work to a fresh session instead of letting the context compact.                          |
| `harness-simplify`        | Simplify a real harness workflow.                                                                         |
| `memory-governance`       | Govern durable local agent memory.                                                                        |
| `obsidian-retrieval`      | Retrieve read-only knowledge from Obsidian vaults or local Markdown corpora.                              |
| `output-discipline`       | Shape responses so decisive information and next actions are easy to find.                                |
| `prose-edit`              | Revise existing prose while preserving the writer's voice and meaning.                                    |
| `security-assurance-plan` | Fill security assurance plans (PAS) from a DOCX template and traceable evidence.                          |
| `skill-manager`           | Manage user and project skills: create, doctor, fix, cross-check, and sync their README indexes.          |
| `skill-simplify`          | Simplify an identified skill's content.                                                                   |
| `workflow-automation`     | Turn evidenced repeated human or agent workflows into supported automation.                               |
