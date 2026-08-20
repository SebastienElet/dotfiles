# Verdict cases

Two end-to-end cases. Each names the PR to point the skill at, the verdict it must reach, and the
criteria that decide pass or fail. They are assigned to different forges so that both command sets in
`references/forges.md` get exercised.

Run them against real pull requests — a case that never touched a forge proves nothing about a skill
whose first phase is anchoring. Use a scratch repository if no suitable PR exists; the domain does not
matter, the shape of the diff does.

## Case A — changes required (Bitbucket)

**Diff under review.** An endpoint that reads a state, decides from it, and writes. The read sits
outside the transaction that performs the write, the retry loop wraps only the write, and no unique
index backs the uniqueness the service checks in code. Unit tests exist and pass, sequentially.

**Expected verdict:** _changes required_.

**Pass criteria**

- The verdict is anchored on the head SHA, and the marker
  `<!-- merge-verdict:<pr>:<head-sha-12> -->` is the first line.
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
one regression test covering the reported input. A second input path reaches the same function and is
not covered, and the error code the endpoint now returns is not documented.

**Expected verdict:** _approved with reservations_.

**Pass criteria**

- The verdict names the reservation as a bounded consequence: the uncovered second path and the
  undocumented code (failure class 6), with what would lift each.
- The barrier paragraph gives counts and states that coverage is limited to the reported input path.
- No blocker is raised. Neither an uncovered path nor an undocumented code is a mechanism that loses
  data, so promoting either to a block fails the case.
- The closing sentence states the merge criterion rather than forbidding the merge.

**Fail signals**

- Blocking to be safe, on coverage or on documentation.
- Approving flatly, with the two reservations dropped or buried as prose.
- "Everything is green" with no counts.
- A barrier paragraph with no statement of what the single test does not cover.

## Execution record

Both cases were run once, on 2026-08-11, phases 1 to 5 only — publication was deliberately not
reached. A third run, on 2026-08-12, was a real review rather than a case, and went through phase 6.

- **Case A**, on a Bitbucket PR (`bkt`), reached _changes required_ as expected, on the three
  mechanisms of classes 1, 2 and 3. Two lessons went back into the skill: the reported base was the
  integration branch while the branch was stacked — the opposite of what the Gotcha then claimed —
  and the run that mattered most was the first, which produced no verdict at all because the
  barrier could not install its dependencies. Refusing to rule from that is the skill working, and
  it is why phase 4 precedes phase 5.
- **Case B**, on a GitHub PR (`gh`), ran end to end but produced _changes required_ rather than the
  expected reservations: the target held a measured mechanism that destroys an unversioned file and
  still reports success. The expectation was wrong about the target, not about the skill — the
  _approved with reservations_ path therefore remains unexercised.
- **Third run**, on GitHub, phases 1 to 6, first publication the skill has ever performed: a general
  comment written through a body file, plus a fix ticket and the re-review ticket the procedure then
  required. Verdict _changes required_, on classes 4 and 5 only. Five gaps came back into the skill.
  The requester was the
  author, so GitHub refused the native blocking state and the comment had to carry the enforcement
  alone — a configuration the skill described only for Bitbucket. The head moved between the metadata
  read and the barrier, which phase 1's SHA anchoring caught, so the guard is now exercised rather
  than merely asserted. The repository-wide lint script and the pipeline's gate turned out to be
  different commands over different file sets, and only the second measures the head. And the
  non-blocking section grew until it rivalled the blocking one, which is where the three-line cap
  comes from. Last, the request anticipated a PR that was not open yet, a case phase 1 assumed away.

Nothing has yet validated: the marker's idempotent update, the duplicate-verdict guard, or Bitbucket
comment publication.

Keep this record free of anything belonging to the reviewed repositories — no PR numbers, commit
SHAs, branch names, build counts or defect details. A skill file is committed and published; the
work it was exercised on usually is not.

## Declared gaps in these cases

`gh pr review --request-changes` — GitHub's native blocking state — is exercised by neither case,
since the blocking case is assigned to Bitbucket. The third run attempted it and GitHub refused,
the account being the PR's author; that measures the refusal, not the success path. Run case A a
second time on GitHub, on a PR the account did not write, or the enforcement path stays unverified.

_Approved with reservations_ has never been produced: both cases that reached a verdict landed on
_changes required_. The state that requires stating a bound rather than forbidding a merge is
therefore the least exercised part of the skill.

Do not let the runs that went green imply either gap is covered: that is the same substitution of a
green for a guarantee that phase 4 exists to prevent.
