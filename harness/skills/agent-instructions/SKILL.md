---
name: agent-instructions
description: >
  Maintain coding-agent instructions and their discovery paths. Use when creating or changing
  AGENTS.md, CLAUDE.md, agent rules, or instruction projections. Make sure to use this skill whenever
  agent guidance or its deployment changes, even if the request names only one agent.
metadata:
  category: dev
---

# Agent Instruction Maintenance

## Overview

Keep agent guidance scoped, discoverable, and derived from one canonical source. Apply the shared
maintenance policy before changing instruction content, adapters, deployment, or verification.

## Usage

Use this skill for changes to global or project instructions, agent-specific rule files, and the
configuration that exposes them. For example: `$agent-instructions add a Codex instruction source`.

## Steps

1. Read `references/maintenance.md` completely.
2. Inspect the existing instruction hierarchy and every discovery path affected by the change.
3. Choose one canonical source and keep agent-specific locations as projections or adapters.
4. Update every installer, generated file, index, and verification barrier that consumes the source.
5. Verify each affected agent projection independently.

## Gotchas

- **Putting conditional guidance in always-loaded instructions** — every task pays the context cost;
  expose it as a skill for agents that support progressive disclosure.
- **Treating identically named mechanisms as equivalent** — Claude Markdown rules and Codex command
  rules have different semantics; verify the consumer's discovery contract.
- **Updating one projection only** — another agent silently keeps stale behavior; enumerate every
  declared consumer and test its actual destination.

## Constraints

- Maintain one canonical source for shared instruction behavior.
- Never inject a conditional rule into an always-loaded instruction file as a compatibility shim.
- Never report a projection healthy unless its real deployed destination was checked.
