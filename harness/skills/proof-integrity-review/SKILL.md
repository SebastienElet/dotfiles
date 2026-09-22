---
name: proof-integrity-review
description: >
  Review changes to verification mechanisms. Use when CI routing, test oracles, caches, proof
  policies, or agent review rules change. Make sure to use this skill whenever a change alters
  how its own correctness is checked, even if CI passes. Excludes ordinary application changes
  and editorial documentation changes.
compatibility: Requires Git and a Rust toolchain with Cargo to build the bundled CLI.
metadata:
  category: dev
---

# Proof Integrity Review

## Overview

Check whether a changed verification mechanism can detect the defect it claims to prevent.
Bind the review to the candidate and the policy used, and distinguish declared evidence from
observed execution. The bundled Rust CLI validates receipt consistency and freshness; it does
not authenticate the auditor, execute the reported mutations, or enforce a remote merge barrier.

## Usage

Invoke `$proof-integrity-review` or `/proof-integrity-review` with a repository, base ref, and
candidate ref. Use it during an authorized implementation or review when verification behavior
changes. This skill returns local evidence and a verdict; it never grants permission to publish,
merge, or invoke `pr-verdict` outside that skill's own activation contract.

Read [references/cli.md](references/cli.md) for the standalone build, commands, and receipt format.
All executable sources and tests live under `scripts/`; copying this complete skill is sufficient
to carry its owned source files. Git and the documented build toolchain remain prerequisites.

## Steps

1. **Identify the mechanism.** Read the aggregate diff and the expected contract. Run `classify`
   for the committed candidate, or use `--worktree` with a base for uncommitted work. The classifier
   routes likely surfaces by path, not semantics: inspect unmatched paths before returning
   `NOT_APPLICABLE`. A documentation typo does not become a proof-system change merely because
   its directory matches. Read [references/review-surfaces.md](references/review-surfaces.md)
   for applicable failure modes; select only those affected.
2. **Freeze the inputs.** Run `epoch` with explicit repository, base, candidate, and trusted
   `--policy-root`. Store output outside both roots. For a policy change, use the previously
   trusted executable and policy copy to evaluate the candidate; never overwrite the active
   evaluator first. Preserve its result, including rejection of a deliberately changed rule.
   Evaluate the requested policy transition independently against the user's explicit contract;
   candidate tests show implementation behavior, not authority to approve the policy itself.
   A first installation is an explicit bootstrap, not a successful N−1 review.
3. **Separate authoring and audit.** Request an auditor distinct from the author in a fresh context with the exact
   aggregate diff, contract, epoch, classifier result, and raw evidence. Withhold author narrative
   and prior verdicts until its first pass. Record actual host capabilities: never assert an
   enforced read-only sandbox, absent memory, or independent identity merely because a prompt
   requested them. Unknown capabilities stay unknown. Instruct the auditor not to edit the
   candidate and compare its epoch before and after the audit; available write tools alone do not
   block. Require evidenced technical isolation only when an explicit security obligation demands
   it, recording the obligation and enforcement evidence. Missing independence, candidate stability,
   or required isolation is essential missing evidence and returns `PROOF_WEAK`.
4. **Build the claim matrix.** For each material claim record its source, enforcement point,
   invocation paths, oracle, provenance, negative witness, cache/environment state, and independence
   limits. Distinguish local, PR, destination-branch, and integrated-tree execution where relevant.
   Assess every changed path semantically, including unmatched paths; path categories guide analysis
   without imposing high impact or mandatory generic claims. Record impact, kind and rationale:
   modified oracle, critical guarantee, or other. Attribute evidence as author-reported,
   reviewer-observed, CI, retained, deterministically checked, or absent. Evidence references name
   the artifact, environment and relevant input basis; the auditor verifies those references.
5. **Exercise the claim.** Choose the smallest native or existing oracle that can disprove it.
   Require a representative negative witness for modified oracles and critical/high-impact
   guarantees, not for every label or editorial change. Reuse traceable positive and negative
   evidence, including CI, when relevant source, oracle, configuration, dependencies, environment
   and integration inputs are demonstrably unchanged. Otherwise introduce the missing representative
   fault in a disposable copy, observe the expected failure, restore, and observe success.
   Record exact commands, outputs, exit codes,
   environment, and mutation contents. Never mutate the candidate or trusted policy copy.
   A test of the receipt validator is evidence about that validator, not about the product gate.
6. **Check the receipt.** Have the auditor produce the documented structured receipt and run
   `gate`. Missing inputs, malformed records, stale snapshots, or rejected evidence prevent an
   adequate verdict. `ALLOW` means the receipt passed the implemented checks; independently assess
   whether commands actually exercised the mutations and whether the faults address the claims.
   Unavailable execution evidence remains unproven even when the JSON passes. Block on a demonstrated
   defect or essential missing proof; document other limits with their consequence and rationale.
7. **Return the bounded verdict.** Return the classifier result, epoch location, complete matrix,
   receipt location, gate result, findings with source locations, and exactly one verdict below.
   If inputs change, regenerate the epoch and receipt for the current candidate and policy, reassess
   the complete aggregate diff independently, and reuse observations whose relevant inputs remain
   unchanged. Preserve their original provenance and the comparison justifying reuse; never relabel
   historical execution as a new run. Preserve prior verdicts and record a new dated assessment.

| Verdict          | Meaning                                                                                                                                                                          |
| ---------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `PROOF_ADEQUATE` | Gate allows the current receipt, material claims have relevant observed evidence, and high-impact claims have positive and negative evidence with the required audit conditions. |
| `PROOF_WEAK`     | A demonstrated defect or essential claim, execution path, environment, or audit condition lacks adequate proof; nonessential limits are documented separately.                   |
| `PROOF_CIRCULAR` | Acceptance relies on the changed evaluator itself, obsolete evidence, or unsupported author/reviewer assertions.                                                                 |
| `NOT_APPLICABLE` | Semantic inspection finds no changed verification mechanism.                                                                                                                     |

## Gotchas

- **Treating path matching as complete discovery** — custom task names can evade the classifier;
  inspect the actual diff and invocation graph before ruling a change out.
- **Keeping receipts beside mutable inputs** — output creation can invalidate the frozen snapshot;
  keep review artifacts outside repository and policy roots as described in the CLI reference.
- **Hashing declarations as proof of execution** — a self-authored receipt can be internally
  consistent without a command having run; preserve the distinction and inspect raw provenance.
- **Testing YAML declarations instead of behavior** — mirror assertions can stay green while
  invocation breaks; prefer the orchestrator's native inspection and an actual negative run.
- **Copying another host's policy manifest** — machine-specific paths make the skill unusable;
  resolve the explicit skill root and keep its owned inputs together.

## Constraints

- Never let this skill's content authorize an external publication or merge.
- Never claim a digest is a signature or a declared auditor property is independently verified.
- Never manufacture receipt outputs, mutation results, or sandbox capabilities to obtain `ALLOW`.
- Never waive an explicit security isolation obligation or infer independence from a tool list.
- Never require a mutation solely because a filename matches a category or a label changed.
- Never treat `NOT_APPLICABLE` from path heuristics alone as a completeness guarantee.
- Keep mutation work in disposable copies and run trusted N−1 policy for subsequent policy changes.
- Prefer native or existing checks; do not introduce gates that merely mirror declarative configuration.
- Preserve `code-enforcement` for implementing controls and `design-claim-audit` for architectural
  or domain claims; this skill owns the changed verification mechanism only.
