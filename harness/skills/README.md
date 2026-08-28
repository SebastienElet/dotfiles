# User Skills

This directory is the canonical source for user-scoped agent skills.

## Conventions

- One skill per subdirectory.
- Each skill must include a `SKILL.md` file.
- Optional folders: `agents/`, `scripts/`, `references/`, `assets/`, `evals/`.
- Manage skills with `/skill-manager`.

## Dev

| Skill                        | Description                                                                                          |
| ---------------------------- | ---------------------------------------------------------------------------------------------------- |
| `agent-instructions`         | Maintain coding-agent instructions and their discovery paths.                                        |
| `claude-developer`           | Prepare manual implementation and correction prompts for Claude Code without invoking it.            |
| `codegraph`                  | Explore large repositories structurally with CodeGraph.                                              |
| `enforcement-code`           | Write code whose purpose is to refuse: hook, guard, validator, permission check, lint rule, CI gate. |
| `harness-reflection`         | Turn repeated agent failures into evidence-backed harness improvements.                              |
| `issue-creation`             | Draft, validate, review, and publish tracker issues across forges.                                   |
| `linear-start`               | Start or resume implementation of an assigned Linear issue in a Bitbucket repository.                |
| `linear-sync`                | Reconcile assigned Linear issues with Bitbucket pull-request reality without reviewing code.         |
| `linear-workflow`            | Apply the shared Linear and Bitbucket work invariants.                                               |
| `pr-feedback`                | Collect evidence-backed review feedback and reviewer-authored fixes from merged pull requests.       |
| `pr-fix`                     | Repair an open pull request after an independent merge review.                                       |
| `pr-verdict`                 | Deliver a PR verdict on an open pull request, yours or another author's.                             |
| `requirements-clarification` | Clarify requirements before implementation.                                                          |

## Product

| Skill               | Description                                                                          |
| ------------------- | ------------------------------------------------------------------------------------ |
| `linear-issue-spec` | Prepare implementation-ready Linear development issues as functional specifications. |

## Ops

| Skill                 | Description                                                                                      |
| --------------------- | ------------------------------------------------------------------------------------------------ |
| `handoff`             | Hand the current work to a fresh session instead of letting the context compact.                 |
| `obsidian-retrieval`  | Retrieve read-only knowledge from Obsidian vaults or local Markdown corpora.                     |
| `skill-manager`       | Manage user and project skills: create, doctor, fix, cross-check, and sync their README indexes. |
| `workflow-automation` | Turn evidenced repeated human or agent workflows into supported automation.                      |
