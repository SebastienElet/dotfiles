# Verdict cases

Three behavioral scenarios, followed by a separately qualified execution record. Expected verdicts
and pass criteria describe what a case should demonstrate; they are not execution evidence.

Cases A and B target Bitbucket and GitHub respectively. Only an observed forge operation can support
a claim about anchoring or publication. The revised Case B package and Case C isolate phase-5
decisions on supplied evidence; they do not execute the represented tests or publish a verdict.
Nothing in this register authorizes another review, approval or publication.

## Case A — changes required (Bitbucket)

**Diff under review.** An endpoint that reads a state, decides from it, and writes. The read sits
outside the transaction that performs the write, the retry loop wraps only the write, and no unique
index backs the uniqueness the service checks in code. Unit tests exist and pass, sequentially.

**Expected verdict:** _changes required_.

**Pass criteria**

- The verdict is anchored on the head SHA, and the marker
  `<!-- pr-verdict:<pr>:<head-sha-12> -->` is the first line.
- At least one blocker is stated as an ordered sequence ending in a broken invariant — the
  out-of-transaction read, the stale retry, or the missing constraint. Failure classes 1, 2 and 3.
- The barrier paragraph gives counts _and_ states that the passing tests are sequential and therefore
  say nothing about the concurrent interleaving that motivates the blockers.
- A fix ticket is linked, and no ticket exists solely to request or record the re-review.
- The closing sentence is "do not approve or merge this head" (in the PR's language). On Bitbucket
  this sentence is the entire enforcement; its absence fails the case even if everything else is
  right.

**Fail signals**

- "The tests pass, so the concurrency looks fine."
- A blocker phrased as a risk ("this could be racy") with no sequence.
- Naming and structure remarks inside the blocking paragraph.
- Two comments published, or a second comment added when one already carries the same
  `<pr>:<sha>`.

## Case B — approved with reservations (GitHub)

**Diff under review.** A small, correct bug fix: a null-handling defect repaired at its cause, with
one regression test covering the reported input and a controlled faulty variant that reproduces its
test-first RED. A second, unchanged input path reaches the same function and is not covered, and the
pre-existing error code is not documented.

**Expected verdict:** _approved with reservations_.

**Pass criteria**

- The changed-behavior ledger records the repaired null input, its passing regression test on the
  exact head, and the reproduced test-first RED as its negative witness.
- The verdict names the reservation as a bounded consequence: the unchanged uncovered second path
  and the pre-existing undocumented code (failure class 6), with what would lift each.
- The barrier paragraph gives counts and states that coverage is limited to the reported input path.
- No blocker is raised. Neither an uncovered path nor an undocumented code is a mechanism that loses
  data, so promoting either to a block fails the case.
- The closing sentence states the merge criterion rather than forbidding the merge.

**Fail signals**

- Blocking to be safe, on coverage or on documentation.
- Approving flatly, with the two reservations dropped or buried as prose.
- "Everything is green" with no counts.
- A barrier paragraph with no statement of what the single test does not cover.

## Case C — critical guarantees without negative witnesses

**Evidence package under review.** A rolling-deployment change claims four observable contracts:
historical cursors preserve their original ordering, create replay returns the original resource
without a second allocation, a schema migration publishes atomically, and the public contract
declares the conflict returned for an identical in-progress request. The exact head passes a large
aggregate barrier, but the review record contains no test-first RED or controlled faulty variant for
any of the four contracts.

**Expected verdict:** _changes required_.

**Pass criteria**

- The changed-behavior ledger retains all four contracts as separate rows.
- Aggregate barrier results are not substituted for behavior-level evidence.
- Every missing negative witness is recorded as `absent`.
- The verdict is _changes required_ for missing sensitivity evidence on ordering compatibility,
  replay uniqueness and atomicity. The conflict label alone needs no mutation; assess its actual
  protocol behavior and justify whether its proof is essential.

**Fail signals**

- Either approval verdict because the aggregate barrier is green.
- A summary paragraph that drops one or more ledger rows.
- A passing regression test described as a negative witness without an observed failing counterpart.

## Case D — reusable CI and bounded editorial changes

An independent fresh-context auditor has write-capable tools and unknown persistent memory.
Candidate snapshots match before and after. No security obligation requires technical isolation.
Traceable CI exercises the relevant tests on unchanged source, oracle, dependency, environment and
integration inputs; one additional label edit has relevant rendering evidence. Modified oracles
have representative observed negative witnesses.

Expected: approval is possible with these facts attributed and limits documented. Do not invent
read-only enforcement, demand a local rerun, or mutate every label. If relevant inputs differ,
reuse requires a new applicability assessment; missing essential proof blocks. If an explicit
security obligation requires isolation and enforcement is unknown, approval is blocked. An older
same-head rejection remains historical; record a new assessment instead of changing its verdict.

## Execution record

Reconciled on 2026-09-19 against accessible sources. **Reported** means a historical account was
read, but its primary execution trace was not verified here. **Verified observation** means the
specific tool exchange was inspected; it does not certify the whole review or the current skill.
The [source record](execution-evidence.md) gives provenance, retained observations and limitations.
An inaccessible trace is unverified here, not nonexistent.

| Record                                    | Decision evidence                                                                                                                                                                        | Forge publication                                                                                                                           | Status and source                                                                                                                                     |
| ----------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| Case A, 2026-08-11                        | Historical account reports _changes required_ on Bitbucket, and an earlier attempt that returned no verdict when dependencies could not be installed.                                    | Account says phases 1–5 only; publication deliberately not reached.                                                                         | Reported; primary trace unavailable in this audit. [E1](execution-evidence.md#e1--original-cases-a-and-b).                                            |
| Original Case B, 2026-08-11               | Historical account reports _changes required_: the real target contained a blocking mechanism, so it did not match the intended reservations scenario.                                   | Account says phases 1–5 only, not an end-to-end publication run.                                                                            | Reported; primary trace unavailable in this audit. [E1](execution-evidence.md#e1--original-cases-a-and-b).                                            |
| Third historical run, 2026-08-12          | Account reports _changes required_, a moved head detected, and refusal of GitHub's native blocking review on a self-authored PR.                                                         | Account reports a general GitHub comment and tickets, including a re-review ticket required by the procedure then.                          | Reported; neither the original tool responses nor the corresponding comment were verified here. [E2](execution-evidence.md#e2--third-historical-run). |
| Revised Case B package, 2026-08-28        | Account reports 3/3 _approved with reservations_ decisions with the completed ledger retained, from supplied positive and negative evidence.                                             | Not part of the package evaluation.                                                                                                         | Documented decision-only result; individual outputs not recovered here. [E3](execution-evidence.md#e3--supplied-evidence-packages).                   |
| Case C package, 2026-08-28                | Account reports candidate retention 3/3; first integrated wording blocks 3/3 but retains rows 0/3; revised wording retains all four rows and blocks 3/3, with witnesses marked `absent`. | Not part of the package evaluation.                                                                                                         | Documented decision-only results; individual outputs not recovered here. [E3](execution-evidence.md#e3--supplied-evidence-packages).                  |
| Additional GitHub publication, 2026-08-23 | Published text says _approved with reservations_; this predates the behavior ledger and does not establish current approval criteria.                                                    | A `gh pr comment` request with a `pr-verdict` marker returned a comment URL and exit 0; the public comment was read back during this audit. | Verified publication observation. [E4](execution-evidence.md#e4--github-comment-publication).                                                         |
| Bitbucket repair comment                  | A `pr-fix` repair record is not a `pr-verdict` decision.                                                                                                                                 | Comment presence was confirmed by read-back after the posting pipeline returned an error.                                                   | Verified adjacent operation, excluded from verdict-publication coverage. [E5](execution-evidence.md#e5--bitbucket-repair-comment).                    |

The E3 counts remain attributed historical results, not newly reproduced measurements. The supplied
barrier and negative witnesses test the decision made from that package, not whether the agent
actually ran those commands. No previously documented experiment was repeated for this audit.

## Current scope and remaining gaps

These records predate the later measurement, composition and proof-integrity requirements; see the
[revision boundary](execution-evidence.md#revision-boundary). They do not validate today's complete
six-phase workflow, runtime skill selection, measurement emission, or an applicable proof audit.

The retained sources do not establish:

- Publication of a `pr-verdict` comment on Bitbucket. E5 establishes a repair comment only; other
  historical publication traces mentioned in the issue remain unverified here.
- Updating an existing verdict with the same marker, avoiding a duplicate on replay, or preserving
  the old verdict when a new head receives a new comment. A search before a single post proves none
  of these outcomes.
- Successful GitHub native blocking review. E2 reports a refusal; E4 is a general comment.
- An unconditional _approved_ decision supported by a complete behavior ledger, or a native forge
  approval. E3 reports the reservations decision only.
- Independent recovery of the E1–E3 execution outputs, including the claimed moved-head detection
  and the E3 per-run ledger rows.

These are limits of the retained evidence, not claims that the paths have never run. One successful
comment or a reported 3/3 result is not a reliability guarantee. Keep any future evidence minimal
and anonymized; do not retain private identities, repository coordinates, ticket IDs or full logs.
