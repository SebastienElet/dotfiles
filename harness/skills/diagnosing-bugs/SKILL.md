---
name: diagnosing-bugs
description: >
  Diagnose bugs and performance regressions. Use when behavior fails, throws, hangs, becomes slow,
  or a test is intermittent. Make sure to use this skill whenever a fix depends on understanding
  an unexpected failure, even if the user suggests a cause.
license: MIT
metadata:
  category: dev
---

# Diagnosing Bugs

## Overview

Establish a reproduction that can detect the reported symptom, reduce it, then test falsifiable
hypotheses one variable at a time. A suggested cause is a claim to investigate. Distinguish facts,
hypotheses and missing execution evidence throughout the diagnosis.

Adapted from [diagnosing-bugs by Matt Pocock](https://github.com/mattpocock/skills/tree/d81f3a183412e71a5b1e84ca21bc1a35eea03a60/skills/engineering/diagnosing-bugs).
The upstream license is preserved in [LICENSE](LICENSE).

## Usage

Use `$diagnosing-bugs <symptom>` or `/diagnosing-bugs <symptom>` when investigating a failure.
For example: `Diagnose why concurrent invoice creation sometimes succeeds twice.` Inspect source
and existing probes freely; permission to diagnose does not authorize production mutation.

## Steps

1. Locate the affected flow and its callers. Read relevant ADRs, domain terminology, recent
   changes and local dependency configuration. Verify official dependency documentation before
   compensating for assumed external behavior. Redact secrets from every shared command or artifact.
2. Find the cheapest existing or native feedback loop: a failing test, public CLI invocation,
   HTTP request, browser interaction, trace replay or profiler. State the exact symptom it detects.
   Execute it and retain the command, result and environment. A process exiting successfully is
   insufficient when the symptom is a wrong value, missing side effect or slow operation.
3. If the reproducing environment is unavailable, continue source inspection and prepare the
   probe. Label explanations as unverified hypotheses; ask only for the missing access or redacted
   artifact needed to execute it. Do not claim reproduction, confirmed cause or verified repair.
4. Reduce the failing scenario one input, caller or configuration element at a time, preserving
   the symptom. For intermittent failures, pin the controllable conditions and record reproduction
   frequency. Do not convert an intermittent bug into a passing check with retries or skips.
5. Rank plausible hypotheses with a prediction for each. Test one variable at a time, using a
   debugger, focused temporary instrumentation or bisection. For performance, measure a baseline
   and compare the same workload. Keep instrumentation bounded and remove it after diagnosis.
6. Once execution confirms the cause, use `tdd` to turn the reproduction into the smallest
   regression test at the boundary owning the failure, then apply the repair. If no adequate test
   boundary exists, report that gap and the available direct reproduction evidence explicitly.
7. Run the original scenario and the relevant required checks. Remove temporary instrumentation
   and disposable artifacts. Report the demonstrated cause, checks, execution environment and
   any supported environments left untested.

## Gotchas

- **Treating the user's suggested cause as established** — the wrong path gets patched; reproduce
  the symptom and test the prediction before choosing a fix.
- **Stopping all reading when remote access is missing** — useful preparation is lost; inspect the
  owned flow and prepare a probe while retaining the execution limitation.
- **A fast loop checking a different failure** — a green result hides the original bug; rerun the
  unreduced scenario after the fix.
- **Adding a debugging harness by reflex** — a temporary investigation becomes a maintained tool;
  prefer existing probes and follow repository policy before adding validation infrastructure.

## Constraints

- Never call a hypothesis a confirmed cause without relevant execution evidence.
- Never expose credentials or captured authentication headers in commands or reports.
- Never mutate production or external state without the relevant authorization.
- Never use retries, skips or relaxed assertions to hide an intermittent failure.
- Preserve independent authorized work when one environment blocks the reproduction.
