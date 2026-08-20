# PR verdict template

Fill this into `verdict.md`, then publish with the commands in `references/forges.md`. A slot marked
REQUIRED that cannot be filled means the phase it belongs to was not done — go back to it instead of
publishing an incomplete verdict. This file is English; the verdict itself follows the language of
the PR.

## Skeleton

```text
<!-- merge-verdict:<pr>:<head-sha-12> -->
## Independent verdict — <changes required | approved with reservations | approved>

<Anchor sentence — REQUIRED. Whose work, on which head SHA, against which base: the real one,
naming the parent PR when the branch is stacked. Then the CI state, open tasks and conflicts,
or "none observed".>

<Blocking paragraph — one clause per blocker: the mechanism, then the invariant it breaks. Close
with one sentence stating what must become true to lift them. Omit this paragraph entirely when
the verdict is "approved".>

<Barrier paragraph — REQUIRED. Open with "Authenticated local validation on this exact head:"
and give counts, never adjectives. Then, in the same paragraph, REQUIRED: what this evidence does
not cover. The verdict is invalid without that second half. Evidence the author supplied — an
attachment, a pasted output — is named as theirs: never counted as measured here, never reported
as absent.>

<Non-blocking remarks — three lines at most, one per remark, each prefixed "Non-blocking:". Keep
the ones that would change a reviewer's decision, drop the rest: past three, the section is a
second review competing with the verdict for attention. Drop it entirely rather than pad it.>

Fix: <url>                REQUIRED when the verdict blocks
Initial review: <url>     REQUIRED when a previous verdict exists on this PR

<Closing sentence — REQUIRED, one line, executable:
  changes required          → "Do not approve or merge this head."
  approved with reservations → "Mergeable once <criterion>."
  approved                   → "Approved on this head."
When the verdict blocks and the forge carries no native blocking state — Bitbucket, or your own
PR on GitHub — the same line says that this comment is the only thing holding the merge.>
```

## Filled example

A _changes required_ verdict. Every identifier and figure below is invented — a committed skill must
carry nothing from the repositories it was exercised on. The example is English because these files
are; a real verdict is written in the language of its PR. Note what the barrier paragraph does: it
gives numbers, then immediately spends a sentence dismantling its own green.

```text
<!-- merge-verdict:1042:a1b2c3d4e5f6 -->
## Independent verdict — changes required

Review of PR #1042 on a1b2c3d4e5f6, stacked base feat/ledger-read-side@9f8e7d6c5b4a.
Pipeline #318 green, no task and no conflict observed.

Blockers: the snapshot and the controls both run before the closing transaction, so a concurrent
write can vanish from the successor; two simultaneous closes can create two successors, because the
retry never re-reads the winning result and the uniqueness constraints that would refuse the second
one do not exist. Lift: concurrent tests against PostgreSQL, an explicit contract for the deferred
controls and for tenant authorization, then a rebase once the parent PR merges.

Authenticated local validation on this exact head: global lint green (18/18 builds, 0 errors, the
145-warning threshold respected), typecheck green on both touched packages, 7/7 close unit tests
green. Those tests remain sequential: they cover none of the concurrent interleaving that motivates
the blockers above.

Non-blocking: the documented 409/412 codes no longer match the real behavior.

Fix: https://tracker.example/ISSUE-158
Initial review: https://forge.example/pull-requests/1042/comments/155

Do not approve or merge this head.
```

## Self-check before publishing

- The marker is the first line, and its SHA is the head you actually checked out.
- Every clause in the blocking paragraph names a sequence of steps, not a quality judgement.
- The barrier paragraph contains digits, and a sentence saying what those digits do not prove.
- The closing sentence tells the reader what to do, not how the reviewer feels.
- The barrier paragraph names the command it ran, and that command is the one CI runs.
- Every attachment in the description and the comments was opened, and evidence the author supplied
  is attributed to them rather than reported missing.
- No lift criterion asks for a run the PR already shows; it names the control the repository lacks.
- At most three non-blocking lines; a fourth means the section is competing with the verdict.
- No re-review ticket or `Re-review:` slot; the head-specific verdict is the re-review record.
- Total under about thirty lines. Past that, preferences have leaked into the blocking section.
