---
name: harness-simplify
description: >
  Simplify a real harness workflow. Use when the user explicitly requests harness simplification
  to reduce conflicting instructions, excessive reading, redundant tools, unnecessary steps, or
  repeated human intervention. Make sure to use this skill for such requests even if no component
  is named; ordinary code cleanup or repeated failures alone do not trigger it.
metadata:
  category: ops
---

# Harness Simplify

## Overview

Reduce the complexity and cost of one real harness workflow while preserving useful outcomes and
governance. Start with a bounded diagnosis; significant changes wait for a validated checkpoint.
This skill coordinates existing maintenance procedures and does not replace them.

## Usage

`$harness-simplify simplify the repeated instruction reads in this review session`

`/harness-simplify reduce the manual steps in this harness workflow`

An explicit natural-language simplification request also qualifies. Inputs are a real session or
supported summary, a concrete cost, and the affected environment; output is a standalone checkpoint,
then a narrowly approved change and observation report.

## Steps

1. Confirm the explicit simplification request and name its concrete cost. Without an actionable
   example, ask exactly one question: "Which real session shows the cost, and what happened?"
   Wait for that example. Bound the first diagnosis to its workflow and implicated components;
   do not inventory the entire setup.
2. Reconstruct the relevant sequence: instructions loaded, routing decisions, skills and references
   read, tools called, checks executed, and results. Separate **declared behavior**, **documented
   mechanism**, and **observed execution**; cite the evidence and environment for each. Use existing
   supported summaries and measurements, not private telemetry formats or raw transcript harvesting.
   Mark missing observations and measurements as unknown, never zero or proof of absence.
3. Inspect canonical sources and affected consumers. Resolve symlinks, generated projections,
   indexes, and exports to their owning source before calling anything duplicated. Check each
   agent's actual discovery path and availability: a mentioned skill need not be installed or
   discoverable. When Claude Code or Codex behavior affects a decision, consult current official
   documentation and local configuration; record the URL, date, applicable version, and unresolved
   discrepancies. Documentation establishes a mechanism, not evidence that this installation ran it.
4. For each candidate, record its useful outcome, reason for existence, actual consumers, removal
   consequence, and evidence gaps. Try deletion first; otherwise reduce, merge cohesive responsibilities,
   or move to the appropriate mechanism. Do not add a mechanism to conceal an unnecessary one.
   Keep stable orientation in instructions, conditional procedures in skills, deterministic
   invariants in suitable checks, and durable knowledge in its source or governed memory.
5. Propose the smallest coherent change on one axis. Preserve a moved procedure's useful trigger,
   discoverability, and outcome for every affected consumer. Before removing an important check,
   establish either an explicit decision retiring its requirement or adequate remaining coverage
   through a named behavioral oracle. If neither is established, retain it and report the gap.
6. Present this standalone checkpoint and wait for user validation before a significant change:

   - **Established facts:** bounded workflow, cost evidence, canonical sources, consumers,
     declared/documented/observed distinctions, and existing review outcomes and evidence.
   - **Proposed decisions:** one axis, smallest coherent change, useful outcomes and governance
     preserved, removal consequences, and applicable canonical maintenance procedure.
   - **Hypotheses:** assumptions, falsifiers, missing measurements, and unverified residues.
   - **Deferred subjects:** independent axes with their still-useful decisions, hypotheses,
     review history, evidence, memory questions, and measurement questions retained explicitly.
   - **Next change and observation:** exact scope for approval, affected consumers, the real
     workflow to replay, expected observable difference, preservation checks, and rollback condition.

   Keep the checkpoint understandable without earlier chat. Preserve concise evidence references
   and their conclusions; never erase unresolved or adverse evidence to shorten it. A diagnostic
   request does not authorize the proposed edits; material scope changes require another checkpoint.

7. After validation, apply only that change through the canonical procedures: `skill-manager` for
   skills and indexes; `agent-instructions` for instructions and discovery or deployment;
   `harness-reflection` for repeated-failure learning; `memory-governance` for durable memory;
   and `enforcement-code` for check changes. Read only procedures required by the chosen axis and
   retain their approval, evidence, promotion, and verification requirements. If a procedure is not
   exposed as a skill for this agent, inspect its canonical source and documented projection;
   do not assume invocation succeeded or recreate it locally. Report a blocked dependency when
   its required source or capability is unavailable.
8. Replay the affected workflow and check each affected consumer independently, including useful
   failure paths and moved procedures' activation. Compare supported before/after observations.
   Run relevant existing native checks; use the bounded cases in [references/validation.md](references/validation.md)
   when validating this skill. Distinguish scenario reasoning from actual host execution. Without
   access to a real installation, report that limit and leave deployment or runtime behavior unverified.
9. Report briefly: cost actually removed, useful behaviors preserved and their evidence,
   uncertainties, and the next observation. A smaller declaration alone does not establish faster
   execution. Consider further acceleration or automation only after simplification, through
   `workflow-automation` in a separate iteration unless it is exactly the validated change.

## Gotchas

- **Counting symlink paths as maintained copies** — removing one can break a consumer; resolve
  ownership and discovery before proposing deletion.
- **Moving guidance to a skill only one agent has** — the other loses the procedure; verify its
  supported route and activation before retiring the old location.
- **Rarely seeing an automation or check** — session samples can miss background or failure-path
  value; inspect its requirement, consumers, and execution mechanism before judging usefulness.
- **Compressing away an earlier failed review** — unresolved risk disappears from the decision;
  retain review conclusions, evidence, residues, and independent memory or measurement subjects.

## Constraints

- Never begin a broad harness audit without a separately agreed scope.
- Never replace deterministic enforcement with a probabilistic instruction to be careful.
- Never edit third-party plugins or generated destinations directly; use their supported owning
  mechanism, or leave the unsupported change blocked.
- Never reimplement canonical maintenance, learning, memory, or automation policies here.
- Never add telemetry collection, an audit engine, or ad hoc instrumentation to this workflow.
- Never claim measured savings, successful activation, or verified deployment from declarations,
  unexecuted scenarios, or missing evidence.
