# Execution evidence reconciliation

Source audit for [the verdict register](cases.md), performed on 2026-09-19 for
[issue #226](https://github.com/SebastienElet/dotfiles/issues/226). This is a read-only reconciliation
of past executions, not a new review or evaluation campaign.

## Source boundaries

- Public dotfiles history and forge artifacts are linked below. A PR description or committed
  execution account proves that a result was reported, not that its underlying commands succeeded.
- Locally accessible Codex session archives were inspected for tool calls and corresponding
  responses. Copies of a skill or registry in a tool output were not counted as new executions.
  Relevant searches also covered local Claude and Cursor transcript stores without recovering the
  original E1–E3 outputs. This is a bounded search, not an inventory of all historical storage.
- The remote session links attached to the E1/E2 source PRs were inaccessible through the fetch
  tool used here. No conclusion about their existence or contents follows from that failure.
- E4 has both a local tool exchange and a public comment. E5 has a local tool exchange only; its
  normalized extract deliberately omits private coordinates. Readers cannot independently recover
  the private source from this committed extract.
- No full transcript, local session identifier, private repository name, person, internal ticket,
  branch, reviewed commit, build count or defect detail is copied here. Dates identify historical
  observations; placeholders replace target and comment identifiers. Public links concern this
  dotfiles repository only.

## E1 — original cases A and B

**Source inspected:** the [initial committed execution account](https://github.com/SebastienElet/dotfiles/blob/881ebcacce74922a240a4f23539a2d070a47a062/.agents/skills/merge-verdict/references/cases.md)
and [its source PR](https://github.com/SebastienElet/dotfiles/pull/56).

The account dates both cases to 2026-08-11 and limits them to phases 1–5. It reports a Bitbucket
_changes required_ decision for A after an earlier dependency-installation failure prevented a
verdict. B also returned _changes required_: the real GitHub target did not satisfy the intended
non-blocking scenario. The account expressly excludes publication for both.

**Status:** historical report verified as a document; the execution itself remains unverified here.
The original requests, barrier outputs and verdict responses were not recovered. The account does
not establish either a successful reservations scenario or a forge publication. No precise runtime
or model is inferred from a commit's co-author metadata.

## E2 — third historical run

**Source inspected:** the [committed account added after the third run](https://github.com/SebastienElet/dotfiles/blob/4be4fd658e042ba7bfb6cba9a10f2fe777aaa42e/.agents/skills/merge-verdict/references/cases.md)
and [its source PR](https://github.com/SebastienElet/dotfiles/pull/80).

The account dates the run to 2026-08-12 and reports phases 1–6: a _changes required_ decision,
GitHub general-comment publication, ticket creation, native blocking-review refusal because the
account authored the PR, and detection of a changed head before the barrier. The re-review ticket
belongs to the procedure then, not today's contract.

**Status:** reported, not independently verified execution. Neither the original responses nor the
corresponding published comment were identified in this audit. E4 is a different, later run and
cannot substantiate this one's refusal or head-change detection. The former claim that E2 was the
skill's first-ever publication is not retained: the available sources do not establish that ordering.

## E3 — supplied evidence packages

**Source inspected:** the [versioned account introduced with the behavior ledger](https://github.com/SebastienElet/dotfiles/blob/352eab4fdc4d50e69b6700a535eab2b97d4b23a1/harness/skills/pr-verdict/references/cases.md)
and [its source PR](https://github.com/SebastienElet/dotfiles/pull/259).

The account dates the evaluations to 2026-08-28 and specifies fresh Codex subagent contexts,
GPT-5.4, high reasoning effort. Those environment details and counts are reported by that source:

| Package / wording                   | Reported decision                   | Reported ledger result                                  |
| ----------------------------------- | ----------------------------------- | ------------------------------------------------------- |
| C, candidate guidance               | No separate decision count stated   | Four rows retained in 3/3                               |
| C, first integrated wording         | _changes required_ in 3/3           | Rows retained in 0/3; wording rejected                  |
| C, revised template-ordered wording | _changes required_ in 3/3           | Four rows retained, negative witnesses `absent`, in 3/3 |
| B, revised evidence package         | _approved with reservations_ in 3/3 | Completed row retained in 3/3                           |

The source reports no contradictory result for the revised B and C runs. **Status:** documented
decision-only results; individual prompts and responses were not recovered in the accessible local
archives. Repeated copies of this account are not independent corroboration. The counts remain
attributed, not promoted to newly verified runs and not erased as though the evaluations never ran.

These packages supply the positive barrier and negative-witness facts to the decision step. Even
with every response available, that experiment would not prove actual test execution, forge
anchoring, publication, native approval, or reliability outside those prompts and model settings.
No rerun was made merely to replace an unavailable historical trace.

## E4 — GitHub comment publication

**Sources inspected:** a local Codex tool exchange from 2026-08-23 and the resulting
[public verdict comment](https://github.com/SebastienElet/dotfiles/pull/206#issuecomment-5388157939),
retrieved through GitHub's comment API on 2026-09-19. The API creation and update timestamps both
identify 2026-08-23; no edit is inferred from the retrieval.

Normalized observations from the local exchange:

1. A body file was created successfully. Its first line carried
   `<!-- pr-verdict:<pr>:<head-sha-12> -->`; its heading was
   `Independent verdict — approved with reservations`.
2. `gh pr comment <pr> --repo <repository> --body-file <file>` returned a comment URL. The matching
   command-completion event recorded exit 0. Subsequent head metadata matched the marker prefix.
3. This audit read the returned public comment and confirmed the marker and verdict text. It did
   not execute or reproduce the validation claims inside that comment.

The local trace records Codex CLI 0.149.0, `codex-tui`, model `gpt-5.6-sol`, medium reasoning
effort, and `/bin/zsh` command execution. The comment's platform statement is author-supplied,
not an independently verified host measurement in this audit.

**Verified observation:** publication of one GitHub general comment with an _approved with
reservations_ verdict. It predates the mandatory behavior ledger; no filled ledger belongs to
this publication. It proves neither current decision correctness nor native GitHub approval or
blocking state. A successful single post and matching head do not demonstrate marker replay,
duplicate prevention, a moved-head refusal, or comment replacement.

## E5 — Bitbucket repair comment

**Source inspected:** a local Codex tool exchange from 2026-08-26. The source records Codex CLI
0.149.1, `codex-tui`, model `gpt-5.6-sol`, and `zsh`. Only the operation needed to distinguish
repair publication from verdict publication is retained:

| Order | Normalized observed operation                           | Observed response                                                                                        |
| ----- | ------------------------------------------------------- | -------------------------------------------------------------------------------------------------------- |
| 1     | Search existing comments for `pr-fix:<pr>:<head>`       | No matching comment returned; command exit 0                                                             |
| 2     | `bkt pr comment` followed by a `jq` projection          | Projection reports a parse error; pipeline exit 5, so this response alone does not establish publication |
| 3     | Read comments again and select the same `pr-fix` marker | Comment identifier and content returned; command exit 0                                                  |

**Verified observation:** a repair comment was present after the posting attempt, despite the
pipeline error. There was no second posting attempt in this exchange. This is a `pr-fix` record,
not a `pr-verdict` or legacy `merge-verdict` publication. A successful read-back does not prove
idempotence, global uniqueness or correct verdict reasoning. No private forge request was made
during this audit, and the extract does not establish the comment's current remote state.

The historical Bitbucket verdict-publication traces mentioned by issue #226 remain unverified
here. E5 is not silently substituted for them; unavailable traces are not declared nonexistent.

## Revision boundary

The historical observations must be read against their then-current procedure:

- [PR #259](https://github.com/SebastienElet/dotfiles/pull/259), merged 2026-08-28, added the behavior
  ledger and reproduced positive/negative evidence requirement. E4 predates it; E3 evaluates the
  decision from supplied facts only.
- [PR #319](https://github.com/SebastienElet/dotfiles/pull/319), merged 2026-09-08, added structured
  measurement; [PR #329](https://github.com/SebastienElet/dotfiles/pull/329), merged 2026-09-10,
  permitted `pr-fix` composition stopping at phase 5. None of E1–E5 validates those integrations.
- [PR #341](https://github.com/SebastienElet/dotfiles/pull/341), merged 2026-09-18, added the
  conditional proof-integrity audit and its verdict requirements. Its CLI tests and reported
  composition scenarios do not certify runtime skill selection or complete forge execution.
  Receipt consistency is not execution authentication. None of the older cases establishes this
  audit, its freshness handling, or the resulting approval refusal paths.

This reconciliation changes the evidence record only. It does not re-review those PRs, change
their verdicts, certify their implementation, or alter the skill's current decision rules.
