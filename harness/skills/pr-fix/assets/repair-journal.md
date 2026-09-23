# PR repair journal template

Keep one local journal per canonical PR URL at the path defined in `SKILL.md`. Reload it across
sessions, update it after each correction or external result, and retain it after publication.
This is operational state, not durable agent memory or a file to commit to the contributor's branch.

## Skeleton

```markdown
# Repair journal

PR: <canonical URL>
Source: <repository and ref>
Initial head: <SHA>
Current head: <SHA>
Status: <in progress | pending validation | pending approval | pending publication | published>
Pending: <remaining correction, check, decision or publication error; none when complete>
Independent verdict: <result and reviewed SHA, or pending>
Native approval: <state from references/native-approval.md, account, SHA and time>
PR state: <forge state read with the approval, e.g. OPEN>
Comment: <ID and URL, or not published>
Published head: <SHA, or not published>

## Corrections

- <problem>: <failure mechanism and broken invariant> → <final behavior>.
  Commits: <SHAs>; delivery: <prepared | pushed | reverted | superseded>.
  Evidence: <failing observation, then passing command and counts; tested SHA and environment>.
  Limits: <what the evidence does not cover>.

## Passes

- <date, starting SHA → resulting SHA>: <corrections and any superseded attempts>.
  Barrier: <tier, commands, counts, environment and evidence limits>.
  Independent verdict: <result and reviewed SHA, or pending>.
  Required remote CI: <result URL and SHA, pending, unavailable or none required>.
  Native approval: <each state reached in this pass, with account, SHA and time, or not reached>.

## Deliberate omissions

- <finding and reason, or none>
```

## Resume checks

- Match the stored PR URL and source to forge metadata before using the journal.
- Reconcile prepared commits with the remote after an interrupted push; local preparation is not
  proof of delivery.
- Preserve earlier evidence with its original SHA; a new head requires a current verdict and checks.
- After an uncertain publication, find the stable PR marker remotely before retrying; a missing
  local comment ID does not prove that the comment was never created.
- If history is incomplete, record the gap and recover only verifiable facts before summarizing.
- After `requested` or `uncertain`, read the forge's approvals before calling approve again; a
  missing local confirmation does not prove that the approval was never recorded.
- A new head invalidates `confirmed` for the record: renew verdict, CI and native approval on it.
