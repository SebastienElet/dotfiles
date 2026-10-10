# Verdict cases

Behavioral scenarios, followed by a separately qualified execution record. Expected verdicts
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

## Case E — action reachability and deliberate prerequisites

A PR adds an organization declaration to an existing details screen. Its use case allows a record
with no personal contacts, but the parent returns an empty-state message when the contact list is
empty, before mounting the declaration control. API tests and isolated-control tests pass; an older
parent test still expects only the empty message.

Expected: _changes required_, naming the parent guard and the missing control in that valid starting
state. Lift: the enclosing screen makes the declaration usable in that state, with relevant evidence.
Green child/API tests and the inherited empty-state expectation do not establish the new workflow.

Variation: an existing authorization contract allows editors to declare organizations, but a
copied administrator-only guard hides the added control from editors. The PR is silent about roles.
Expected: a blocker at the guard; its presence in the control does not declare a legitimate
prerequisite. Ground eligibility in the independent authorization contract.

Counterexample: a PR deliberately limits a new contact-editing action to an editor at the editing
stage, with an existing contact. The action is usable from the screen when these stated prerequisites
hold. Expected: no reachability blocker for its absence with no contact, another role or another stage.
A read-only notice of missing data does not itself promise another editor on the details screen.
A declared gate is not an exemption when it still prevents the promised action after its prerequisites
hold. These scenarios specify decision criteria, not recorded executions.

## Case F — hostile URL inputs and display-output omissions

A PR shares a URL normalizer between installation and status. Its credential redactor uses
`[^/@]*@`, and tests cover one credential pair and the main error. Installation compares the
redacted string to a trusted marketplace URL. Host metadata also reaches report fields `version`,
`steps.*` and `deferrals` without presentation cleanup.

Expected: _changes required_, with two distinct findings. Class 11 traces a multiple-`@` input
leaving a credential suffix visible, and `https://evil.invalid?@github.com/...` becoming a trusted
identity after regex replacement. Class 12 traces hostile metadata through an uncovered textual
field into the displayed report. Require the hostile-input table and an inventory covering stdout,
stderr and every textual report field; cleaned error examples alone lift neither finding.

Counterexample: authority-aware parsing rejects foreign or malformed identities, and executed cases
cover multiple `@`, query/fragment `@`, backslashes, controls, newlines, bidirectional controls,
encoded delimiters and repeated/adjacent occurrences. Every reachable display output is exercised
with hostile values while raw metadata still drives internal decisions. Expected: no class 11 or
12 finding when these traces and observations establish the claimed boundary.

## Case G — unchanged public export claimed from shared source

A PR replaces a hand-written MCP schema with `mcpSchema(HandoffCapsuleSchema)` and claims
"unchanged public CLI/MCP contracts". Accepted-value fixtures and type checks pass. No byte
comparison of the actual base/head exports is supplied; the adapter can drop the published
`version` description even when the source schema retains it.

Expected: class 13 records the sweep outcome `unproven`, names the missing exported comparison and
never records `holds` from shared types or passing fixtures. If unchanged public compatibility is
essential to approval, the missing proof requires _changes required_, with its essentiality and lift
condition stated in the blocking paragraph. Otherwise retain the bounded reservation under phase 5.
Neither case records `broken by` or asserts that exported bytes differ without evidence.

Counterexample: the actual public exports from the named base and head, produced with the public
export command, each revision's pinned dependencies and a recorded environment, compare byte-for-byte
equal including descriptions.
Expected: class 13 holds on that observed comparison. A variant deleting only the exported
`version` description must fail equality even when accepted-value fixtures still pass; record
`broken by` with the observed metadata change contradicting the unchanged-contract promise. Restore
the metadata or declare and assess the contract change. These cases specify expected decisions,
not recorded executions or a complete six-phase workflow.

## Case H — Git remote identity (class 11)

### H1 — optional SCP user

**Defect:** requiring `user@` rejects `github.com:Owner/Repo.git` while accepting
`git@github.com:Owner/Repo.git` for the same repository. Report the traced rejection.

**Counterexample:** executed cases parse `[user@]host:path` and select the same repository with or
without the user. No finding on this identity rule.

### H2 — DNS host casing

**Defect:** literal host comparison rejects `git@GitHub.COM:Owner/Repo.git` as a different host.
Report the incorrect identity decision.

**Counterexample:** executed SCP and SSH cases establish host case equivalence. No host-casing finding.

### H3 — provider path casing

**Defect:** case-sensitive path comparison rejects GitHub `Owner/Repo` for `owner/repo`.
Report the valid alias rejected by the picker.

**Counterexample:** executed cases accept GitHub aliases and retain case distinctions for a separate
GitLab fixture whose local contract requires exact matching. No finding; do not impose one path
case rule on every provider.

### H4 — default SSH port

**Defect:** preserving `:22` as a different identity rejects
`ssh://git@github.com:22/Owner/Repo.git` for a repository accepted without the port.
Report the traced rejection.

**Counterexample:** executed cases equate omitted and explicit port 22 while preserving or rejecting
non-default endpoints with diagnostics. No port-identity finding.

### H5 — final suffix and slash

**Defect:** the declared identity contract equates `Repo`, `Repo.git`, `Repo/` and `Repo.git/`,
but the picker rejects `Repo.git/`. Report that observed mismatch.

**Counterexample:** executed cases accept all four final forms and preserve interior path segments.
No suffix-normalization finding.

### H6 — distinct or unsupported endpoint

**Defect:** a normalizer replaces a foreign host with `github.com` and selects a trusted repository.
Report the trust-boundary violation.

**Counterexample:** executed cases keep foreign and non-default endpoints distinct or reject them
with diagnostics, and diagnose malformed SCP and unsupported local paths. No endpoint-confusion finding.

An absent executed identity or hostile-input case is `unproven`, not a demonstrated rejection or leak.

## Case I — selection accounting (class 14)

### I1 — missing expected target

**Defect:** an expected project is missing; filtering returns empty success and skips its required
suite. Report the missing-project/unchanged conflation and omitted check.

**Counterexample:** native inspection validates expected targets before filtering and fails explicitly
on this missing project. A successful empty selection diagnoses every exclusion. No target-loss finding.

### I2 — required prerequisite

**Defect:** selection follows only source changes; an execution-prerequisite change skips the suite
that depends on it. Report the omitted required suite.

**Counterexample:** native graph observations select that suite for the prerequisite change and
reject a variant that omits it. No prerequisite-selection finding; no new mirror gate is required.

### I3 — ambiguous candidate

**Defect:** extraction finds two candidates, then discards the ambiguous one before classification.
Report its traced disappearance, even if the filtered input and output counts agree.

**Counterexample:** both occurrences remain identifiable as retained or diagnosed rejections;
ambiguity stays visible until resolution or rejection, and an unclassifiable input fails explicitly.
No candidate-loss finding.

### I4 — incomplete rejection diagnostic

**Defect:** the contract requires both an unavailable-dependency reason and a forbidden-dependency
reason, but the rejection reports only the latter. Report the missing required reason as a bounded
reservation unless that proof is essential to approval.

**Counterexample:** one diagnostic includes both required reasons and identifies the rejected input.
No diagnostic finding; complete reasons do not require separate warning counts.

### I5 — wrong eligibility

**Defect:** the independent contract excludes parents with unfinished children, but the selector
retains a parent with a blocked child. Report the violated eligibility rule.

**Counterexample:** the observed partition rejects that parent with its reason and retains eligible
parents exactly once. No eligibility finding; derive the expected set from the contract.

### I6 — unsupported destination

**Defect:** an update of an existing record is routed to a creation-only handler, preventing the
promised update. Report the unsupported route.

**Counterexample:** observed routing sends the update to an authorized editor. No routing finding;
correct accounting alone would not establish this result.

Missing accounting without an observed loss or wrong decision is `unproven`; never invent a dropped input.

## Case J — comparison evidence (class 15)

### J1 — rewritten instruction

**Defect:** a rewritten mandatory instruction is installed after source/projection checks alone.
Its behavior has no base/head comparison. Record `unproven` and block adoption pending that proof.

**Counterexample:** exact base/head texts are exercised in fresh contexts on identical violating and
safe scenarios, with recorded inputs/environment, observations and a criterion fixed beforehand and
met. Class 15 holds within that measured scope; no effectiveness gain follows from decision equality.

### J2 — inconclusive required budget

**Defect:** correctness tests pass, but benchmark noise leaves a required performance budget
unqualified. Record `unproven` and block merge; infer neither a regression nor a gain.

**Counterexample:** named base/head executables, pinned inputs and the same workload/environment
meet the previously declared budget with noise and order effects accounted for. Class 15 holds for
that budget.

### J3 — unsupported optional speed claim

**Defect:** a PR claims faster startup without comparative observations, and no performance budget
is required. Record the optional claim as `unproven`; require its removal, qualification or measurement.

**Counterexample:** the optional claim is removed. No benchmark is required for it; any independent
instruction-adoption criterion or required budget still applies.

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
