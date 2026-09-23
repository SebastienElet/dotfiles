# Native approval

Read this in step 8, after the approval gate holds and before building the repair record. A
repository's own forge skill wins over these raw commands. `pr-verdict` never runs this procedure:
native approval belongs to the `pr-fix` invocation, not to the composed review.

## Gate

Approve only when every condition holds on one freshly read head SHA:

- the latest independent verdict is exactly `approved` and concerns that SHA;
- the required remote CI succeeded on that SHA, or the journal records that none is required;
- no correction, push or reconciliation remains pending;
- the user has not forbidden approval in this invocation or an earlier message about this PR;
- the authenticated account is not the PR author.

A failed condition records `Native approval: not performed — <condition>` and skips the forge call.
`approved with reservations`, pending or failed CI and a moved head keep the repair pending exactly
as step 8 requires; a user prohibition or own PR is a final outcome, not a pending one.

## Commands

| Step               | GitHub (`gh`)                                                                           | Bitbucket Cloud (`bkt`)                                                                       |
| ------------------ | --------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------- |
| Authenticated user | `gh api user --jq .login`                                                               | `bkt api /2.0/user` → `.uuid`                                                                 |
| PR author          | `gh pr view <n> --json author --jq .author.login`                                       | `bkt pr view <n> --json` → `.pull_request.author.uuid`                                        |
| Existing approval  | `gh api repos/{owner}/{repo}/pulls/<n>/reviews` → `.user.login`, `.state`, `.commit_id` | `bkt pr view <n> --json` → `.pull_request.participants[]` `.user.uuid`, `.approved`, `.state` |
| Approve            | `gh pr review <n> --approve --body-file approval.md`                                    | `bkt pr approve <n>`                                                                          |
| Withdraw           | not available to the reviewer; report instead                                           | `bkt api -X DELETE /2.0/repositories/{ws}/{repo}/pullrequests/<n>/approve`                    |
| PR state           | `gh pr view <n> --json state`                                                           | `bkt pr view <n> --json` → `.pull_request.state`                                              |

Observed on 2026-09-23 with `bkt` 0.32.1 against Bitbucket Cloud: the participant payload exposes
`role`, `approved` and `state: "approved"` per user, and the PR `state` stayed `OPEN` after the
reviewer's approval. The withdraw endpoint is the documented Cloud REST `DELETE …/approve`
(response 204); this skill has not exercised it. GitHub documents that authors cannot approve their
own pull requests and may dismiss an approval when a new commit lands. Bitbucket Data Center has not
been exercised: read `bkt pr view --help` and the actual payload before claiming an equivalent.

Pass the GitHub approval body through a file, one sentence in the PR's language naming the SHA and
pointing to the repair record; it is not a verdict. Bitbucket approval carries no body.

## Procedure

1. Re-read head SHA, required checks, PR state and the authenticated user. Evaluate the gate.
2. Read existing approvals. An approval by the authenticated account that binds the current SHA —
   GitHub `APPROVED` with `commit_id` equal to it, or Bitbucket `approved: true` read after the
   head was confirmed unchanged — is `already present`: record it and do not call the forge again.
   Bitbucket does not bind an approval to a SHA; record that the approval may predate this head.
   A GitHub approval on an older `commit_id` does not count; submit a new one.
3. Record `Native approval: requested on <SHA>` in the journal before the forge call.
4. Call the native approve command once.
5. Read approvals and the head again, whatever the exit code. Record `confirmed` only when the
   authenticated account's approval is present and the head is still the approved SHA. On
   Bitbucket, record the reviewer approval and the PR state separately: `approved` describes the
   participant, while the PR stays `OPEN` until a separate merge.
6. A refusal records the forge's message: missing permission, own PR, closed or merged PR, or a
   merge check. Never publish a comment, task or reaction as a substitute for the refused action.
7. An ambiguous result — timeout, network error, unparseable output — is resolved by reading
   approvals before any retry. Retry the approve call only when the read shows no approval. If the
   read itself fails, keep `Native approval: uncertain` and pause publication; never report it.
8. If the head moved between the gate and the confirmation, the approval may attach to an
   unreviewed head. On Bitbucket, withdraw it, verify `approved: false` and re-anchor; a failed
   withdrawal or unreadable result is `uncertain` and is reported to the user as an approval that
   may cover an unreviewed head. On GitHub, the review stays bound to its `commit_id`; report that
   it concerns the older SHA. Either way the moved head keeps the repair pending.

## Journal states

`requested`, `confirmed`, `already present`, `not performed — <reason>`, `refused — <forge
message>`, `uncertain — <failed read>` and `withdrawn — <reason>`. Each entry carries the account,
the SHA and the observation time. Only `confirmed` and `already present` may appear as an approval
in the repair record; every other state is stated as the absence of a native approval.
