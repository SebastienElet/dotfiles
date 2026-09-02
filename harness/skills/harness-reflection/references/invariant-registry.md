# Invariant Registry

Use this reference only after the diagnostic class is `harness-gap`. The canonical registry is
`harness/invariants/registry.json`; it is the source of truth for named invariants, not a replacement
for factual `pr-feedback` evidence.

## Workflow

1. Preserve each `pr-feedback` occurrence as factual evidence. Search the registry by source and
   behavior before drafting a candidate.
2. Classify the registry cause as `not-applied`, `not-loaded`, `unknown`, `blind-spot`, or
   `judgment`. `judgment` may inform a `skip`, but cannot become a control.
3. Return exactly one decision:
   - `skip` when the evidence is insufficient, the cause is not reusable, or the class is `judgment`;
   - `link` when the finding belongs to an existing invariant and should add a source without a
     duplicate record;
   - `propose` when no invariant matches and two distinct PRs or high severity supports a candidate.
4. Keep `link` and `propose` session-local until explicit approval. No time pressure authorizes a
   registry, skill, instruction, or `pr-feedback` mutation.
5. After explicit approval, validate the chosen surface, all three consumers, and the oracle. Run
   `bun tooling/invariant-registry-cli.ts` before reporting the registry valid.

## Candidate model

Describe a proposal with its invariant statement, `controlKind`, cause class, severity, factual
sources, scope and exceptions, lifecycle, and verification state. An active record also needs an
approval. An enforceable active or verified invariant needs an oracle with its failure path and test
path. Keep evidence distinct from interpretation: `pr-feedback` records what review observed;
reflection evaluates whether that evidence warrants a control.

## Surface matrix

| Control kind | Compatible surface | Consumer declaration |
| --- | --- | --- |
| `probabilistic` | `always-loaded-instruction`, `conditional-skill`, `project-local-contract` | State how Claude, Codex, and Cursor load or do not support it. |
| `enforceable` | `hook`, `permission`, `lint`, `type`, `architectural-test` | State how Claude, Codex, and Cursor invoke or encounter the control. |

Every record declares `claude`, `codex`, and `cursor` as `supported` with a mechanism or
`unsupported` with a reason. Do not infer a consumer from another consumer's adapter.

## Verification and retirement

Candidates remain `unverified`. A measured or verified result records outcome, timestamp, and
environment. Promotion requires explicit approval plus two distinct pull requests or high severity.
Use the cheapest behavioral oracle that proves the stated invariant; a failing oracle must exercise
its declared failure path. A retired invariant records `retiredAt`, reason, and, when applicable,
`replacedBy`.

## CLI diagnostics

The CLI rejects invalid lifecycle transitions, missing approval, insufficient evidence, promotion of
`judgment`, incompatible surfaces, and missing or unsafe oracle paths. Resolve diagnostics in the
record or return `skip`; do not bypass them by weakening the validator or reclassifying evidence.
