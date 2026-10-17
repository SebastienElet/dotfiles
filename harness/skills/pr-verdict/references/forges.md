# PR forges

Read this before the first CLI call of phase 1.

## Detection

```sh
git remote get-url origin
```

- contains `github.com` → `gh`
- contains `bitbucket.org` (Cloud) or a Bitbucket Data Center host → `bkt`

**A repository's own forge skill wins over this file.** A validated wrapper encodes house PR
conventions (required reviewers, ticket prefixes, merge policy) that these raw commands do not know
about. Look for it before falling back here.

## Command parity

| Step                     | GitHub (`gh`)                                                                                                                                                                                     | Bitbucket (`bkt`)                                                                                                                                   |
| ------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| Metadata                 | `gh pr view <n> --json headRefOid,headRefName,headRepository,headRepositoryOwner,isCrossRepository,maintainerCanModify,baseRefName,mergeStateStatus,title,body`                                   | `bkt pr view <n> --json`                                                                                                                            |
| Head SHA                 | `.headRefOid`                                                                                                                                                                                     | Cloud `.pull_request.source.commit.hash` (12-character abbreviation), DC `.fromRef.latestCommit` — confirm the path against the actual payload      |
| Fetch the head           | `gh pr checkout <n>`                                                                                                                                                                              | `bkt pr checkout <n>`                                                                                                                               |
| CI state                 | `gh pr checks <n>`                                                                                                                                                                                | `bkt pr checks <n>`                                                                                                                                 |
| Diff                     | `gh pr diff <n>`                                                                                                                                                                                  | `bkt pr diff <n>`                                                                                                                                   |
| Existing comments        | `gh api --paginate --slurp repos/{owner}/{repo}/issues/<n>/comments?per_page=100` and `gh api --paginate --slurp repos/{owner}/{repo}/pulls/<n>/reviews?per_page=100`                             | `bkt pr comments <n> --json`                                                                                                                        |
| Publish                  | `gh pr comment <n> --body-file verdict.md`                                                                                                                                                        | `bkt pr comment <n> --text "$(cat verdict.md)"`                                                                                                     |
| Update                   | Issue comment: `gh api -X PATCH /repos/{owner}/{repo}/issues/comments/<id> -F body=@verdict.md`; review summary: `gh api -X PUT /repos/{owner}/{repo}/pulls/<n>/reviews/<id> -F body=@verdict.md` | Cloud: `bkt api -X PUT /2.0/repositories/{ws}/{repo}/pullrequests/<n>/comments/<id> --input "$(jq -n --rawfile b verdict.md '{content:{raw:$b}}')"` |
| Changes-requested review | `gh pr review <n> --request-changes --body-file verdict.md` — refused on your own PR; use a marker-free pointer body when updating a marked verdict without that native state on this head        | no reliable equivalent                                                                                                                              |
| Clear own change request | `gh pr review <n> --approve --body-file review-signal.md` — only after the publishing user authorizes native approval and only on another author's PR                                             | no verified Bitbucket equivalent                                                                                                                    |

`bkt` covers both Bitbucket Cloud and Data Center; the two differ on JSON shape and on some flags
(`bkt pr comments --state` is Cloud-only). Verified against `gh` 2.97 and `bkt` as installed by this
repository — re-check `--help` when a command is rejected rather than guessing a flag.

## Reading the author's evidence

The description and the comments carry attachments: a screenshot of an output, a pasted log, a
recording. The JSON body returns them as raw markup, so list the uploads and open each one.

```sh
gh pr view 1042 --json body,comments --jq '.body, (.comments[].body)' \
  | grep -oE 'https://github\.com/user-attachments/assets/[0-9a-f-]+' | sort -u
curl -sSL -H "Authorization: token $(gh auth token)" -o attachment-1.png <url>
```

Read the downloaded file as an image. An unopened attachment is unread evidence, and phase 4 must
attribute it to the author instead of calling it absent. Verified with `gh` 2.97 against a private
repository: the token is required, and the upload answers `200 image/png`. Older bodies embed
`https://<host>/<owner>/<repo>/assets/<id>` instead, which the same command reaches. The Bitbucket
attachment endpoint has not been exercised from this skill — read `bkt pr view --help` before
claiming an equivalent rather than assuming one.

## Checking out the exact head

Neither `gh pr checkout` nor `bkt pr checkout` guarantees the recorded SHA: both land on a local
branch that tracks a ref which may have moved between the metadata read and the checkout. Detach on
the SHA and prove it:

```sh
gh pr checkout 1042           # or: bkt pr checkout 1042 — brings the objects local
git checkout a1b2c3d4e5f6     # the SHA recorded in phase 1
test "$(git rev-parse HEAD)" = "$(git rev-parse a1b2c3d4e5f6)" || echo "head moved: re-anchor"
```

If the SHA is gone from the local objects, the head was force-pushed while you were reading it. Stop
and re-anchor: everything measured so far belongs to a commit nobody will merge.

Bitbucket Cloud returns only the first 12 characters of the head hash (`bkt` 0.32.1). Require the
value to be non-empty and resolve it with `git rev-parse` as above. Comparing it to
`git rev-parse HEAD` as a string rejects the right commit.

## Resolving the repair target

`pr-fix` pushes to the PR source, which is not necessarily `origin`. Parse the same metadata payload
used for the head anchor and require every source value before editing:

- GitHub: `.headRepository.nameWithOwner`, `.headRefName` and `.headRefOid`.
  `.maintainerCanModify` describes the author's fork setting, not the authenticated token's complete
  permission set; a normal push is the permission check.
- Bitbucket Cloud: `.pull_request.source.repository.full_name`,
  `.pull_request.source.branch.name` and `.pull_request.source.commit.hash`.
- Bitbucket Data Center: `.fromRef.repository.project.key`, `.fromRef.repository.slug`,
  `.fromRef.id` and `.fromRef.latestCommit`.

Validate the branch with `git check-ref-format --branch`, quote every parsed value, and obtain the
source repository's clone URL from the same forge payload or repository endpoint. Add a dedicated
temporary remote and push `HEAD` normally to that source branch. An absent value, invalid ref,
changed SHA, unavailable clone URL or rejected push stops the repair; never fall back to `origin` or
a force option.

## Finding the real base

The base the forge reports is the merge target, not the review base. Ask git what the PR actually
carries before reading a single line of diff:

```sh
git log --oneline <dest>..<head>          # every commit the merge would bring in
git diff --stat <parent-head>..<head>     # the PR's own work, when a parent PR is named
```

When the PR description names a parent PR, resolve that parent's head through the forge and diff
against it. When the two counts disagree — the forge's diff much larger than the commit range that
belongs to this author's feature — the branch is stacked and the reported diff includes the parent's
work. Say so, and require the rebase or the change of target once the parent merges.

## Pass the body through a file, never inline

A verdict contains apostrophes, backticks and accented text. Inline in a shell argument, they break
quoting or silently truncate the comment. Write `verdict.md`, then pass it:

- `gh`: `--body-file verdict.md` natively; for the API, `-F body=@verdict.md` (`-F` reads `@<path>`;
  `-f` does not).
- `bkt`: no `--body-file`. Use command substitution — `--text "$(cat verdict.md)"` in bash/zsh,
  `--text (cat verdict.md)` in fish — or build the JSON body with `jq`, which never lets the text
  reach the shell as syntax. `bkt api --input` accepts a JSON **string**, not a file or stdin marker:

```sh
bkt api -X PUT /2.0/repositories/{ws}/{repo}/pullrequests/1042/comments/1001 \
  --input "$(jq -n --rawfile b verdict.md '{content: {raw: $b}}')"
```

Data Center takes `{"text": …, "version": <n>}` instead, and rejects the update without the
comment's current `version` — read it from `bkt pr comments <n> --json` first.

## Finding a previous verdict

Build the complete first-line marker from the anchored PR number and the first 12 characters of
the anchored head SHA, for example `<!-- pr-verdict:1042:a1b2c3d4e5f6 -->`. The complete first line
must equal that marker; a prefix, another PR, another SHA or a marker later in the body is not a
match. Identify the account through the same authenticated forge context that will publish, before
reading comments. Require a nonempty stable account ID, a complete comment listing and a valid ID
for every exact match. A failed API call, malformed listing or missing author ID on an exact match
stops publication, even if another owned match is present. Do not turn an uncertain read into an
empty result.

For GitHub, `gh api user --jq .id` identifies the publishing account. Read **both** paginated
collections; `gh pr review --request-changes` creates a review summary, which is not an issue
comment. The REST objects expose `.user.id`, `.body` and `.id`; review summaries also expose
`.state` and `.commit_id`. Use the same repository and PR established in phase 1:

```sh
gh api user --jq '.id'
gh api --paginate --slurp 'repos/{owner}/{repo}/issues/1042/comments?per_page=100' |
  jq --arg marker '<!-- pr-verdict:1042:a1b2c3d4e5f6 -->' \
    'if length > 0 and all(.[]; type == "array") and
        all(.[][]; type == "object" and (.body | type == "string") and (.id | type == "number")) then
       [.[][] | select((.body // "" | split("\n")[0]) == $marker) | {id, author_id: .user.id}]
     else error("invalid issue-comment listing") end'
gh api --paginate --slurp 'repos/{owner}/{repo}/pulls/1042/reviews?per_page=100' |
  jq --arg marker '<!-- pr-verdict:1042:a1b2c3d4e5f6 -->' \
    'if length > 0 and all(.[]; type == "array") and
        all(.[][]; type == "object" and (.body | type == "string") and (.id | type == "number")) then
       [.[][] | select((.body // "" | split("\n")[0]) == $marker) | {id, author_id: .user.id, state, commit_id}]
     else error("invalid review listing") end'
```

Configure bash or zsh to propagate a failed pipeline stage so a failed paginated fetch cannot look
like an empty list. Compare the numeric `author_id` with the numeric viewer ID, and keep the source
(`issue comment` or `review summary`) with each match. A GitHub login name or
`author_association` is not the ownership key. Immediately before an update, read the selected
object again by ID and recheck its author and exact first-line marker. Update a review summary with
the review endpoint in the command table; updating an issue comment through that endpoint fails.
The review update changes its body, not its native review state. Use the **full**, chronological
GitHub review listing, not only the marker-filtered matches, to find the publishing account's last
opinionated review (`APPROVED` or `CHANGES_REQUESTED`) and its `commit_id`. Ignore later `COMMENTED`
reviews when deciding the opinionated state. If review state is dismissed or cannot be established,
stop before publishing a conflicting verdict. GitHub's REST references document
[issue comments](https://docs.github.com/en/rest/issues/comments) and
[review summaries](https://docs.github.com/en/rest/pulls/reviews) as separate resources.

```sh
publisher_id=$(gh api user --jq '.id')
gh api --paginate --slurp 'repos/{owner}/{repo}/pulls/1042/reviews?per_page=100' |
  jq --arg publisher_id "$publisher_id" '
    def known_state:
      . == "APPROVED" or . == "CHANGES_REQUESTED" or . == "COMMENTED" or
      . == "PENDING" or . == "DISMISSED";
    def valid_account_review:
      (.id | type == "number") and (.state | known_state) and
      (if .state == "APPROVED" or .state == "CHANGES_REQUESTED" then
         (.commit_id | type == "string") and (.commit_id | length > 0)
       else true end);
    if length > 0 and all(.[]; type == "array") and
       all(.[][]; type == "object") and
       all(.[][] | select((.user.id | @json) == $publisher_id); valid_account_review) then
      [.[][] | select((.user.id | @json) == $publisher_id)
       | select(.state == "APPROVED" or .state == "CHANGES_REQUESTED" or .state == "DISMISSED")
       | {id, state, commit_id}] | last
    else error("invalid review listing") end'
```

GitHub documents chronological review listing. A `null` result means this account has no
opinionated review. Validate the numeric viewer ID before running this query; an uncertain or
dismissed state stops the transition.

For a blocking reassessment of someone else's PR, skip another request-changes review only when
that last opinionated review is `CHANGES_REQUESTED` **and** its `commit_id` is the anchored head.
Otherwise submit a marker-free request-changes review linking the updated marked verdict, then
verify that the publishing account's latest opinionated review on this head is
`CHANGES_REQUESTED`. A historical change request followed by an approval does not satisfy this
check. For a new head, the newly created marked request-changes review itself establishes the
state.

For an approval verdict on someone else's PR, check whether the latest opinionated review by this
account is `CHANGES_REQUESTED`, even when it belongs to an older head. If so, native approval is
needed to clear the existing request. Name that action when asking to publish; if it is not
authorized, return the verdict unpublished instead of announcing approval over an active native
block. Use a pending marked body first: keep any earlier assessment intact and add a visible line
saying the new approval assessment awaits native approval and is not yet published. On a new head,
create a marked pending comment containing that status. Reconcile the pending object's author,
marker and URL before the next write. Submit one marker-free `gh pr review --approve` review whose
body links to that object, then verify through a fresh review listing that this account's latest
opinionated review is `APPROVED`. Only then replace the pending body with the complete approved
`verdict.md` and read it back. If native approval fails or is uncertain, do not write the final body;
re-read the reviews and pending object, then report only their observed states. If the final body
update fails or is uncertain, re-read the object by ID: a confirmed final body completes the
publication, a confirmed pending body leaves it incomplete, and an unreadable body leaves its state
unknown. Stop and reconcile before retrying; never repeat a native approval already observed.
If no active change request exists, retain the existing comment-only approval publication. Never
put the verdict marker in a state-only review. A state review does not by itself prove a mechanical
merge block or unblock; branch rules and bypasses still matter.

For Bitbucket Cloud, `bkt api /2.0/user` returns the publishing account's `.uuid`.
`bkt pr comments 1042 --json` returns all comments in `.comments` (`null` when empty), including
inline comments and replies. Match the exact first line of `.content.raw` and compare `.user.uuid`
with the authenticated UUID; do not use display names. Exclude a deleted comment from updates.
Only a top-level global comment, with both `.parent` and `.inline` absent, is eligible for update.
An exact marker in an owned reply or inline comment is misplaced: stop rather than updating it or
publishing another verdict. An unknown author on such a match also stops; a known foreign reply or
inline comment does not mask publication by the current account.
`bkt` 0.34.0 follows Cloud pagination; a CLI or API failure stops publication. Check that
`.comments` is an array or `null`, then read the selected comment through
`bkt api /2.0/repositories/{ws}/{repo}/pullrequests/1042/comments/<id>` and verify its author and
marker before using the Cloud update endpoint in the command table.
The Cloud API documents `.user`, `.content.raw`, `.deleted` and the update endpoint. If a live
payload omits `.user.uuid`, `.content.raw` or `.id`, stop and inspect the actual response.
The [Cloud user API](https://developer.atlassian.com/cloud/bitbucket/rest/api-group-users/)
identifies the current account; the [pull request comments API](https://developer.atlassian.com/cloud/bitbucket/rest/api-group-pullrequests/)
defines the comment data and update operation.

```sh
bkt api /2.0/user --json --jq '.uuid'
bkt pr comments 1042 --json |
  jq --arg marker '<!-- pr-verdict:1042:a1b2c3d4e5f6 -->' \
    'if has("comments") and (.comments == null or (.comments | type == "array")) and
        all((.comments // [])[]; type == "object" and (.id | type == "number") and
            (.deleted == true or (.content.raw | type == "string"))) then
       [(.comments // [])[] | select(.deleted != true)
        | select((.content.raw // "" | split("\n")[0]) == $marker)
        | {id, author_uuid: .user.uuid, general: (.parent == null and .inline == null)}]
     else error("invalid Bitbucket comment listing") end'
```

Bitbucket Data Center comments use `.author` and `.text`, but its documented user listing does not
identify which account owns the current token. A configured `bkt` username alone is not forge
proof of the publishing account. Until a verified authenticated-user response is available for the
target instance, stop before creating or updating a Data Center verdict. Do not infer ownership from
a matching marker or a display name.

Classify all exact, not-deleted matches across the applicable surfaces before writing:

| Result                                                                           | Action                                                                            |
| -------------------------------------------------------------------------------- | --------------------------------------------------------------------------------- |
| One eligible owned match                                                         | Re-read it, then update that object through its own endpoint.                     |
| No owned match, only known foreign matches or none                               | Publish a new verdict for the current account. Foreign comments remain untouched. |
| More than one owned match                                                        | Stop and report their IDs; never choose one or add another.                       |
| Owned match in an ineligible Bitbucket reply or inline comment                   | Stop and report its ID; never update it as the general verdict.                   |
| Unknown author, missing ID, incomplete listing or uncertain publication response | Stop and reconcile through a fresh remote read before retrying.                   |

After an update, read back its returned ID and verify the author and marker. Creation commands may
return only a URL or success message, so repeat the full lookup after creation and require exactly
one newly owned match on the current marker before treating publication as confirmed. On a timeout
or uncertain response, do the same remote reconciliation before any retry; if it remains uncertain,
stop without another write. Same `<pr>:<sha>` is account-local idempotency; a different SHA receives a new verdict and leaves the old
record intact. Do not use `gh pr comment --edit-last`: the last comment may carry another marker.
The lookup and re-read do not lock the forge: simultaneous publishers can both create a comment.
Coordinate concurrent publication for the same account and head, and report any duplicate found by
post-publication reconciliation rather than choosing one to overwrite.

## Review state and merge enforcement

GitHub records `gh pr review --request-changes` as a review state. Its [review documentation](https://docs.github.com/en/pull-requests/how-tos/review-pull-requests/approving-a-pull-request-with-required-reviews)
says this state is informational unless an applicable ruleset or classic branch protection requires
pull requests; reviewer permissions and bypass rights also matter. Inspect the actual destination
branch, not the local checkout or the repository default. Substitute the target repository and
URL-encoded destination branch from the phase-1 anchor; `{branch}` in `gh api` would resolve to the
current checkout branch:

```sh
gh api "repos/$target_repository/branches/$encoded_destination_branch" --jq '{name, protected}'
gh api --paginate --slurp "repos/$target_repository/rules/branches/$encoded_destination_branch"
gh api "repos/$target_repository/branches/$encoded_destination_branch/protection"
```

The branch rules endpoint reports [active rulesets applicable to that branch](https://docs.github.com/en/rest/repos/rules#get-rules-for-a-branch),
including rules from higher scopes. Its slurped response contains one array per page; inspect every
page before concluding that no pull-request rule applies. The classic protection endpoint returns
`Branch not protected` when no classic rule exists; that response alone says nothing about rulesets.
Read the applicable pull-request requirement, the reviewer's `write` or stronger permission, and
any administrator or ruleset bypass for the account that would merge. Inspect the ruleset details
when bypass actors matter; [GitHub may omit them without ruleset write access](https://docs.github.com/en/rest/repos/rules#get-a-repository-ruleset).
Treat a failed or incomplete lookup as unknown, not as proof of no protection.

If no active rule demonstrates a block, or a bypass or permission remains uncertain, label the
verdict advisory even when GitHub records `CHANGES_REQUESTED`. Claim a mechanical block only after a
controlled PR with the same repository, destination branch, applicable rules, reviewer permission
and merge account actually refuses merging because of that review. Record the repository, branch,
active rules, account and observed refusal in the verdict. A rule listing or disabled merge button
caused by another condition is not that observation. Do not create or merge a controlled PR solely
to support an ordinary verdict; use the advisory wording when the evidence is unavailable.

Bitbucket [Cloud](https://developer.atlassian.com/cloud/bitbucket/announcement-new-request-changes-webhooks/)
and [Data Center](https://confluence.atlassian.com/bitbucketserver/review-and-discuss-a-pull-request-808488540.html)
have a Request changes reviewer status,
but this skill has no verified `bkt` command to set it. `bkt pr approve` exists; declining a PR is a
different act with a different meaning. Here the comment **is** the verdict, and the closing
sentence ("do not approve or merge this head") remains an advisory request.

GitHub falls back to a comment-only verdict whenever the authenticated account authored the
PR: it rejects the call with `Can not request changes on your own pull request`, and there is no
flag around it. `gh pr view --json` exposes no `viewerDidAuthor` field (checked on `gh` 2.97), so
compare the author against the token yourself, in phase 1 —

```sh
test "$(gh pr view 1042 --json author --jq .author.login)" = "$(gh api user --jq .login)" &&
  echo "own PR: request-changes review unavailable"
```

When it is your own PR, publish the comment alone and state its advisory scope. In every GitHub
case, separate the recorded review state from demonstrated merge enforcement.
