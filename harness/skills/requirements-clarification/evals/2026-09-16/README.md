# Requirements clarification evaluations — 2026-09-16

Evidence for [#223](https://github.com/SebastienElet/dotfiles/issues/223). This experiment changes
only the skill's description and Usage, preserving its seven-step procedure and all six original
prompts. Two synthetic artifact-cleanup prompts extend the corpus. No global instructions, other
skills, router, hook, validator, or reusable runner are changed.

The behavioral contract is: before implementation, activate for unresolved decisions with material
consequences in any domain, research available facts, and ask no question about discoverable facts,
settled decisions, or style without material consequences.

## Current evidence

The baseline misses skill activation in all three artifact-retention runs, although the agent asks
material questions. This is a routing failure, not evidence that Codex cannot recognize ambiguity.
All six original cases retain their expected routing and clarification behavior in the baseline.

The first [candidate](candidate-skill.json) fixes the material-positive route but incorrectly
loads the skill in two of three discoverable-fact runs. Those failures remain in
[run 1](candidate/case-3-1.json) and [run 2](candidate/case-3-2.json). A second
[refinement](refined-skill.json) requires a user decision left open after inspection, but still
loads merely to assess clarity in [run 1](refined/case-3-1.json) and
[run 3](refined/case-3-3.json). A third [alternatives-based trigger](alternatives-skill.json)
increases that false-positive rate to 3/3 in its focused trial. All unsuccessful variants remain
available for comparison; none is deployed or treated as a successful correction.

The retained `policy` variant uses the original trigger's concrete form, broadened to undecided
product or operational policy, including authentication boundaries, migration compatibility and
data retention. It explicitly excludes implementing or testing established requirements. Usage
distinguishes known contracts from missing policy. The focused discoverable-fact trial passes 3/3;
the remaining cases use exactly the same skill bytes. No change to the seven-step protocol was
justified.

Independent review also found that the original new negative prompt requested automatic cleanup
without specifying when it runs. Its six baseline/candidate executions remain preserved as
NOT_SCORED, not discarded or counted as passes. The corrected prompt explicitly requests only a
function, with invocation and scheduling out of scope. Its three new baseline runs use the original
skill; the refined and policy variants use the same corrected prompt. The six original prompts and the
new material-positive prompt are identical throughout.

Claude Code and Cursor are **not verified**: native authentication probes report no logged-in
session. No model request or behavioral repetition was performed on either host. Their required
three repetitions per case remain outstanding; the Codex-only exception for #246 does not apply
to #223. This experiment does not establish completion of #223.

## Routing results and execution index

Every cell counts correct routing decisions, not activations: a negative passes only without a skill read. Each numbered link is one independent execution.

| Case                           | Expected        | Baseline                                                                                                            | Candidate                                                                                             | Refined                                                                                  | Alternatives                                                                                            | Retained policy                                                                       |
| ------------------------------ | --------------- | ------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| S1: Authentication             | Activate        | 3/3 ([1](baseline/case-1-1.json), [2](baseline/case-1-2.json), [3](baseline/case-1-3.json))                         | 3/3 ([1](candidate/case-1-1.json), [2](candidate/case-1-2.json), [3](candidate/case-1-3.json))        | 3/3 ([1](refined/case-1-1.json), [2](refined/case-1-2.json), [3](refined/case-1-3.json)) | Not run                                                                                                 | 3/3 ([1](policy/case-1-1.json), [2](policy/case-1-2.json), [3](policy/case-1-3.json)) |
| S2: Migration                  | Activate        | 3/3 ([1](baseline/case-2-1.json), [2](baseline/case-2-2.json), [3](baseline/case-2-3.json))                         | 3/3 ([1](candidate/case-2-1.json), [2](candidate/case-2-2.json), [3](candidate/case-2-3.json))        | 3/3 ([1](refined/case-2-1.json), [2](refined/case-2-2.json), [3](refined/case-2-3.json)) | Not run                                                                                                 | 3/3 ([1](policy/case-2-1.json), [2](policy/case-2-2.json), [3](policy/case-2-3.json)) |
| S3: Discoverable test facts    | Do not activate | 3/3 ([1](baseline/case-3-1.json), [2](baseline/case-3-2.json), [3](baseline/case-3-3.json))                         | 1/3 ([1](candidate/case-3-1.json), [2](candidate/case-3-2.json), [3](candidate/case-3-3.json))        | 1/3 ([1](refined/case-3-1.json), [2](refined/case-3-2.json), [3](refined/case-3-3.json)) | 0/3 ([1](alternatives/case-3-1.json), [2](alternatives/case-3-2.json), [3](alternatives/case-3-3.json)) | 3/3 ([1](policy/case-3-1.json), [2](policy/case-3-2.json), [3](policy/case-3-3.json)) |
| S4: Local rename               | Do not activate | 3/3 ([1](baseline/case-4-1.json), [2](baseline/case-4-2.json), [3](baseline/case-4-3.json))                         | 3/3 ([1](candidate/case-4-1.json), [2](candidate/case-4-2.json), [3](candidate/case-4-3.json))        | 3/3 ([1](refined/case-4-1.json), [2](refined/case-4-2.json), [3](refined/case-4-3.json)) | Not run                                                                                                 | 3/3 ([1](policy/case-4-1.json), [2](policy/case-4-2.json), [3](policy/case-4-3.json)) |
| S5: Neighboring timeout        | Do not activate | 3/3 ([1](baseline/case-5-1.json), [2](baseline/case-5-2.json), [3](baseline/case-5-3.json))                         | 3/3 ([1](candidate/case-5-1.json), [2](candidate/case-5-2.json), [3](candidate/case-5-3.json))        | 3/3 ([1](refined/case-5-1.json), [2](refined/case-5-2.json), [3](refined/case-5-3.json)) | Not run                                                                                                 | 3/3 ([1](policy/case-5-1.json), [2](policy/case-5-2.json), [3](policy/case-5-3.json)) |
| S6: Import style               | Do not activate | 3/3 ([1](baseline/case-6-1.json), [2](baseline/case-6-2.json), [3](baseline/case-6-3.json))                         | 3/3 ([1](candidate/case-6-1.json), [2](candidate/case-6-2.json), [3](candidate/case-6-3.json))        | 3/3 ([1](refined/case-6-1.json), [2](refined/case-6-2.json), [3](refined/case-6-3.json)) | Not run                                                                                                 | 3/3 ([1](policy/case-6-1.json), [2](policy/case-6-2.json), [3](policy/case-6-3.json)) |
| S7: Undecided retention        | Activate        | 0/3 ([1](baseline/case-7-1.json), [2](baseline/case-7-2.json), [3](baseline/case-7-3.json))                         | 3/3 ([1](candidate/case-7-1.json), [2](candidate/case-7-2.json), [3](candidate/case-7-3.json))        | 3/3 ([1](refined/case-7-1.json), [2](refined/case-7-2.json), [3](refined/case-7-3.json)) | Not run                                                                                                 | 3/3 ([1](policy/case-7-1.json), [2](policy/case-7-2.json), [3](policy/case-7-3.json)) |
| S8: Specified cleanup function | Do not activate | 3/3 ([1](baseline-revised/case-8-1.json), [2](baseline-revised/case-8-2.json), [3](baseline-revised/case-8-3.json)) | NOT_SCORED ([1](candidate/case-8-1.json), [2](candidate/case-8-2.json), [3](candidate/case-8-3.json)) | 3/3 ([1](refined/case-8-1.json), [2](refined/case-8-2.json), [3](refined/case-8-3.json)) | Not run                                                                                                 | 3/3 ([1](policy/case-8-1.json), [2](policy/case-8-2.json), [3](policy/case-8-3.json)) |

The original ambiguous baseline S8 remains in [1](baseline/case-8-1.json), [2](baseline/case-8-2.json), and [3](baseline/case-8-3.json), all NOT_SCORED. The baseline S8 cell above links to its corrected-prompt replay.

There are 102 completed executions: 24 initial baseline, 24 candidate, 3 corrected-prompt baseline, 24 refined, 3 alternatives-only trials, and 24 retained-policy trials. Six original-S8 executions are excluded from scoring because of the prompt defect. No unsuccessful run was deleted; no agent process timed out or failed. All scored executions ask zero unnecessary questions. The retained policy passes all 24 routing and clarification verdicts; the baseline composite passes 21/24 routing verdicts.

These totals can be reproduced from the per-record `phase` and `verdict` fields. The effective baseline combines S1–S7 under `baseline` with S8 under `baseline-revised`; original S8 records are never silently replaced.

## Acceptance status

| #223 requirement                                                              | Status                                                                                     |
| ----------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------ |
| Positive outside authentication/migration, three fresh executions per host    | Codex 3/3; Claude Code and Cursor NOT VERIFIED                                             |
| Discoverable facts and style, zero questions in three fresh contexts per host | Codex satisfied on the retained corpus; other hosts NOT VERIFIED                           |
| Negative scenarios avoid activation, three times per host                     | Codex 15/15; other hosts NOT VERIFIED                                                      |
| Authentication/migration preserved                                            | Codex 6/6; other hosts NOT VERIFIED                                                        |
| Individual evidence and traceable aggregate                                   | All 102 executions retained and linked above                                               |
| Modified-surface validation                                                   | Local checks documented in [validation.json](validation.json); remote CI belongs to the PR |

The cross-agent criteria remain incomplete. This PR references #223 without closing it.

## Reproduction and retained evidence

This reuses the isolated manual method delivered by
[#246 / PR #342](https://github.com/SebastienElet/dotfiles/blob/65275906e0878aea48323c5b3563e5bc0cea3c70/harness/skills/linear-workflow/evals/2026-09-16/README.md).
These records belong beside the skill, not under `harness/evals/evidence/`, whose Arnes schema
describes a different experiment. No existing runner was generalized.

Each execution uses a new process, temporary home, Codex home, and workspace; no conversation is
resumed. Materialize every path/content entry in [fixture.json](fixture.json) inside the workspace,
including its exact `AGENTS.md`. Install only this user skill at
`/fixture/home/.agents/skills/requirements-clarification/SKILL.md`: use the exact `content` in
[baseline-skill.json](baseline-skill.json) for `baseline` and `baseline-revised` runs, the
[candidate snapshot](candidate-skill.json) for `candidate`, the [refined snapshot](refined-skill.json)
for `refined`, the [alternatives snapshot](alternatives-skill.json) for its focused trial, or
[SKILL.md](../../SKILL.md) for `policy`.
The unrelated example skill in the fixture is synthetic task data, not an installed user skill.
Codex's bundled system skills remain available; each execution records their hashes.

Submit each exact `query` in [trigger-queries.json](../trigger-queries.json) on stdin, three times
per variant, with at most three concurrent processes and a 240-second timeout. For the exploratory
original case 8, use its exact prompt retained in those execution records rather than the corrected
scenario file:

```text
codex exec --json --ephemeral --ignore-user-config --ignore-rules
  --skip-git-repo-check --sandbox read-only --model gpt-6-astra
  -c 'model_reasoning_effort="high"' --cd /fixture/workspace -
```

The actual command is an argument array in every record. Only PATH, TMPDIR and LANG are inherited
when present; HOME and CODEX_HOME point to that execution's fixture, and VOLTA_HOME retains the
installed CLI. Existing Codex authentication is copied privately for the request, removed after
execution, and never supplied as task data or retained in evidence. The workspace, user settings,
rules, plugins, hooks, MCP configuration, expected answers, previous outputs and other repository
instructions are not inherited. The ordinary product system prompt remains in effect.

Each record preserves date, prompt and hash, fixture and skill hashes, agent/version, requested
model/effort, command, exit status, duration, exact agent messages, tool commands and outputs,
diagnostics, usage, mutation observations and a manual verdict naming the facts researched.
Temporary roots become `/fixture`; account paths become `/host-user`; session identifiers are
omitted. Otherwise agent text is unchanged. All task content is synthetic, with no private corpus.
The provider's resolved model revision and random seed are unavailable, not inferred.

## Verdict criteria

- Positive routing requires a successful observed read of the skill and a response governed by
  its procedure. An announcement or a good clarification response alone is insufficient.
- Negative routing requires no read or invocation of this skill. Discovery metadata alone does
  not count as activation.
- Authentication and migration must still research the fixture and retain their unresolved
  trust/compatibility decisions. Artifact retention must expose an unresolved material choice
  before proposing implementation, despite the request to ship quickly.
- Discoverable facts, local renaming, existing timeout and import order must produce zero user
  questions. A question about execution permission also counts; no historical exemption for
  execution invitations is carried forward. The specified cleanup case must not reopen its
  explicit retention, cutoff or failure policy.
- The specified cleanup case covers the function's supplied policy, not scheduler integration.
  This scope is explicit in the corrected prompt supplied to the evaluated agent.
- Behavior and routing are assessed separately. No-mutation observations are supported by a
  workspace comparison and tool transcript, but read-only instructions and sandbox also prevent
  writes independently. These are decision assessments, not implementation or write-safety tests.
- The clarification verdict does not certify exhaustive retrieval or every factual statement.
  Candidate style runs [1](candidate/case-6-1.json) and [2](candidate/case-6-2.json) miss the hidden
  style file and incorrectly state no import configuration exists. They still ask no question
  and do not activate the skill; this factual limitation remains recorded separately.
- A timeout, unsuccessful agent process or incomplete transcript is INVALID, never PASS. Failed
  optional Git or absent-directory probes are retained; they do not invalidate later successful
  source inspection and a complete answer.

## Historical observations and unverified platforms

The [2026-08-24 report](../../../../../docs/requirements-clarification-validation.md) reported
unnecessary Claude questions and negative Cursor activations. Its aggregate rows were reread;
its original individual raw traces were not re-executed or audited here. These remain historical
observations, not measurements of the currently installed versions.

Current host/probe details are in [environment.json](environment.json). The supported agents remain
Codex, Claude Code and Cursor. This experiment runs on macOS arm64; Linux behavior is not verified.
The full deployed skill collection, competing plugins, real external documentation and production
repositories are outside this synthetic experiment. Three repetitions characterize these prompts
and versions only; they do not establish future deterministic routing.

## Procedural and deterministic checks

The baseline doctor is recorded in [environment.json](environment.json). The final scoped
cross-check covers D1–D6 against the user skill collection: no conflict involving this skill;
description token overlap peaks at 9.43%, below the 40% warning threshold. Procedure and constraints
remain unchanged. There is no duplicate project slug, dead skill reference or scoped reference
conflict. This static assessment is not behavioral evidence.

The derived user-skill index is regenerated twice and remains byte-identical to the repository
version because the description's first sentence is unchanged. Standard validation is unavailable
(`skills-ref` is not installed); no validator was installed. Existing Arnes scenario validation
checks the scenario contracts, not the manual behavioral verdicts. Formatting and spelling checks
cover the authored surfaces; exact agent responses are retained in JSON without prose correction.
No code comments were added.

The completed local checks are recorded in [validation.json](validation.json): seven targeted deployment tests, TypeScript lint and types, native scenario validation, repository formatting, spelling, and evidence integrity. These are macOS observations; they do not replace the missing Claude Code/Cursor runs or remote CI.
