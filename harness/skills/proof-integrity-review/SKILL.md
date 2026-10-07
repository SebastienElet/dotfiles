---
name: proof-integrity-review
description: >
  Review changes to verification mechanisms. Use when CI routing, test oracles, caches, proof
  policies, or agent review rules change. Make sure to use this skill whenever correctness checks
  change, even if CI passes. Excludes application, installation, and editorial changes that leave
  verification unchanged.
compatibility: Requires Git and the affected repository's native checks; the optional receipt CLI requires Cargo.
metadata:
  category: dev
---

# Proof Integrity Review

## Overview

Audit whether a changed verification mechanism can detect the defect it claims to prevent. Use
observed evidence, an independent reviewer, and the exact candidate and policy under review.
The default result is a concise evidence report; a structured receipt is optional and validates
record consistency, not execution truth or permission to merge.

## Usage

Invoke `$proof-integrity-review` or `/proof-integrity-review` with a repository, base and candidate,
or compose it during an authorized review of changed verification behavior. For example:
`Audit the CI change that removes a test-discovery step.` Ordinary installer readiness checks
remain outside this audit when they leave verification unchanged.

Use `assertledger` for requested qualification of a regression test against a declared fault.
Read [references/cli.md](references/cli.md) only when a structured receipt is explicitly requested
by the user or an established external workflow. The bundled CLI remains available under `scripts/`;
its absence does not block the default audit.

## Steps

1. **Identify the changed mechanism.** Inspect the complete diff, consumers and contract, including
   paths a classifier would miss. Select the affected cases in
   [references/review-surfaces.md](references/review-surfaces.md). Return `NOT_APPLICABLE` only
   after semantic inspection; a task name, filename or `condition` check alone is insufficient.
2. **Anchor the review.** Record the base and candidate SHAs, worktree changes when included,
   relevant policy and native checks. Preserve the exact diff and compare candidate state before
   and after the audit. Store evidence outside the candidate and trusted policy roots. When policy
   changes, assess it with the previously trusted instructions or evaluator and the user's explicit
   contract; the proposed policy cannot authorize its own acceptance.
3. **Use an independent reviewer.** Delegate to a fresh context distinct from the author, supplying
   the diff, contract and raw evidence before author conclusions. Keep the candidate untouched and
   experiments in disposable copies. Record unknown host capabilities as unknown; require evidenced
   technical isolation only when an explicit security obligation requires it.
4. **Exercise the claims.** For each changed mechanism record the claim, source, invocation path,
   oracle, observed result, environment and evidence provenance. Account for every changed path,
   grouping related paths where useful. Modified oracles and critical guarantees require a relevant
   negative witness: the actual check rejects a representative fault, then passes after restoration.
   Use native or existing checks; receipt validation cannot replace this observation.
5. **Reuse only relevant evidence.** Compare source, oracle, configuration, dependencies, environment
   and integration inputs before reusing CI or retained runs. Preserve their original provenance.
   A cache hit, a SUCCESS badge or an empty discovery result does not demonstrate execution. Missing
   essential evidence blocks an adequate verdict; document other limits with their consequences.
6. **Return the bounded result.** Name the anchored candidate, changed mechanisms, commands and
   results, required negative witnesses, findings and remaining limits. Return exactly one verdict
   below. If inputs change, refresh their bindings and reassess affected evidence. For an explicitly
   requested structured receipt, also follow `references/cli.md` and report its actual gate result.
   A detected missing, stale or rejected required receipt returns `PROOF_WEAK`; claiming acceptance
   on that invalid evidence instead is `PROOF_CIRCULAR`.

| Verdict          | Meaning                                                                                                              |
| ---------------- | -------------------------------------------------------------------------------------------------------------------- |
| `PROOF_ADEQUATE` | Relevant observed evidence supports the changed mechanisms, with required negative witnesses and independent review. |
| `PROOF_WEAK`     | A demonstrated defect or essential evidence, stability or audit condition is missing.                                |
| `PROOF_CIRCULAR` | Acceptance relies on the changed evaluator itself, stale inputs or unsupported declarations.                         |
| `NOT_APPLICABLE` | Semantic inspection finds no changed verification mechanism.                                                         |

## Gotchas

- **Treating an installer condition as a proof gate** — operational readiness can trigger needless
  ceremony; trace whether it changes verification before activating this audit.
- **Treating a receipt digest as execution evidence** — declarations can be internally consistent
  without a test running; inspect the native observation and its input basis.
- **Counting a green aggregate as coverage** — selection or caching may skip the changed mechanism;
  verify the actual invocation and relevant fault rejection.
- **Reusing evidence after policy changes** — the old assessment no longer binds current inputs;
  preserve history and compare the changed policy independently.

## Constraints

- Never manufacture execution results, auditor independence, sandbox capabilities or receipt fields.
- Never let this skill authorize publication, approval, merge or another skill's activation.
- Never let the candidate policy approve itself or overwrite the trusted evaluator before review.
- Never replace a relevant negative witness with a declaration mirror or receipt-schema test.
- Never require a CLI, epoch or receipt for the default audit; require them only for the explicitly
  requested structured path, without weakening its validation.
- Preserve `code-enforcement` for implementing controls and `design-claim-audit` for architectural
  or domain claims; this skill owns changed verification mechanisms only.
