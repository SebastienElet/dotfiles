---
name: workflow-automation
description: >
  Turn evidenced repeated human or agent workflows into supported automation. Use when the same
  multi-step procedure recurs or measurement shows a stable action sequence. Make sure to use it
  whenever agents repeatedly perform the same operational work, even if nobody requests automation.
metadata:
  category: ops
---

# Workflow Automation

## Overview

Convert a stable recurring outcome, not a copied command trace, into the smallest supported
automation. Keep human authority only where a decision or irreversible external effect requires it.
Repetition identifies a candidate; it does not prove that the observed sequence is the right design.

## Usage

Use when the user reports a recurring workflow or when supported evidence shows the same operational
sequence across independent runs.

```text
$workflow-automation automate this recurring release preparation
$workflow-automation review these repeated agent runs for an automation candidate
```

## Steps

1. Establish recurrence from an explicit user report or evidence from at least three independent
   runs. Treat a single observed run as a candidate requiring more evidence.
2. Name the stable outcome, inputs, outputs, effects, authority, failure modes, and variable choices.
   Remove discovery, debugging, and recovery commands that are incidental to one run.
3. Search for an existing supported command, connector, configuration, task, or workflow that already
   provides the outcome. Standardize that primitive before creating another one.
4. Choose the narrowest durable implementation. Use Just for fixed command orchestration, Bun and
   TypeScript for small testable utilities, Rust for substantial or system-oriented CLIs, and CI or
   a supported scheduler for event-driven or timed execution.
5. Automate every safe deterministic step. Keep an explicit human checkpoint only for unresolved
   judgment, missing authority, or an irreversible external effect.
6. Make effects explicit and design for idempotence, inspection or dry-run, bounded retries,
   resumability, timeouts, and useful failure output where the workflow requires them.
7. Test the real entry point and its failure paths. Run the same supported command locally and in CI
   when both environments claim the guarantee, and name every supported platform not exercised.
8. If recurrence or a stable outcome cannot be established, return the candidate and the evidence
   still needed instead of writing automation.

## Gotchas

- **Automating the trace** — copied tool calls preserve incidental discovery and recovery work;
  model the stable outcome and its boundaries instead.
- **Generalizing one occurrence** — a speculative abstraction creates a new maintenance surface;
  gather independent evidence or an explicit user report of recurrence first.
- **Keeping a passive checklist** — manual toil remains hidden behind prompts; automate safe steps
  and reserve checkpoints for real authority or judgment.
- **Reading private telemetry formats** — internal schemas are unstable and may contain sensitive
  data; consume only supported summaries or queries.
- **Overlapping reflection** — repeated failures require `harness-reflection`; use this skill for a
  recurring workflow whose desired outcome is already understood.

## Constraints

- Never infer recurrence from one run without an explicit user report.
- Never parse private Arnes artifacts, transcripts, or telemetry schemas directly. Until Arnes
  exposes a stable repetition query, automatic cross-session detection is unavailable.
- Never automate irreversible or externally visible effects without established authority and an
  explicit checkpoint when that authority cannot be delegated safely.
- Reuse an existing supported primitive before introducing a new tool.
- Follow the applicable `scripts` boundary: Just orchestrates fixed commands, Bun and TypeScript own
  small utilities, and Rust owns substantial or system-oriented CLIs.
- Never store secrets or raw private prompts in automation evidence, fixtures, or logs.
- Test the shipped entry point rather than a duplicate implementation in the test harness.
