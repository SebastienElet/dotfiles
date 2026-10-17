# Review thread closure

Use this procedure for a correction tied to a review thread, including a PR-level comment thread
on forges that expose one. Only a standalone timeline comment or review body without a thread has
no resolution state; keep the cumulative repair record for those cases. Do not reply or resolve
until the current head has an independent `approved` verdict and either every required remote
check has passed or the journal records that none is required.

## Inventory and binding

1. Read every review thread, including replies and resolution state. Follow pagination for both
   the thread list and comments within a thread; an incomplete read cannot authorize resolution.
   Bind each corrected finding to the stable thread ID and top-level comment ID, not its path or
   current diff line. Record the original URL, reviewer, head SHA and thread state in the journal.
2. On GitHub, use `gh api graphql` to read `pullRequest.reviewThreads` with `pageInfo`, `id`,
   `isResolved`, `viewerCanResolve` and `comments` (`databaseId`, `body`, `url`, `author`,
   `replyTo`). The GraphQL thread ID is for resolution; the top-level comment's numeric
   `databaseId` is for the REST reply endpoint. `gh pr view --json comments` returns timeline
   comments and does not inventory review threads.
3. On Bitbucket, use `bkt pr comments <pr> --details --json`, then
   `bkt pr comments view <pr> <root-comment-id> --json` for each relevant thread. Both
   `bkt pr comment --parent` and `bkt pr comments resolve` take the top-level comment ID.
   Cloud supports a resolution filter; Data Center does not, so inspect each thread directly.

## Reply, then resolve

1. Re-read the PR head, the complete thread and the journal. Continue only for a correction
   present on the independently approved head with its required checks satisfied. If the reviewer
   added a new objection after the correction, return it to the repair slate. If someone already
   resolved the thread, record that state and do not reopen it.
2. Write one short factual reply in the PR's language, naming the final correction and evidence.
   End it with `<!-- pr-fix-thread:<thread-id> -->`. Search the entire thread for that marker
   before writing. If one reply with the marker belongs to the publishing account, reuse it; when
   its content is stale, update only that owned reply. A foreign or duplicate marker is an
   ownership conflict: stop and report it. Never write the reply at the PR root.
3. On GitHub, write the reply body to a file, build a JSON payload with
   `jq -n --rawfile body "$reply_file" '{body:$body}' > "$payload_file"`, then call
   `gh api -X POST repos/<owner>/<repo>/pulls/<pr>/comments/<root-comment-id>/replies --input "$payload_file"`.
   The endpoint requires the top-level review comment ID, not a reply ID. Read the thread back;
   verify the reply's marker, author, URL and `replyTo` before proceeding. For an owned stale
   reply, update that comment through the review-comment update endpoint instead of adding one.
4. On Bitbucket, call `bkt pr comment <pr> --text "$(cat "$reply_file")" --parent <root-comment-id>`.
   Read the thread back with `bkt pr comments view` and verify the reply belongs to the intended
   parent and account. Do not use `bkt pr comment` without `--parent` for a thread response.
5. Re-read the head and thread. If both still match and resolution is permitted, resolve with
   GitHub's `resolveReviewThread` mutation using `input:{threadId:$threadId}` or
   `bkt pr comments resolve <pr> <root-comment-id>`. Read the thread back and require its
   resolved state. If the API rejects the action or the state cannot be read, keep the thread
   pending and report the capability or permission gap; never claim that a reply resolved it.
6. After each remote write, atomically record the head SHA, thread ID, reply ID and URL, author,
   resolution state and read-back evidence in the journal. A timeout or ambiguous response calls
   for a fresh read by marker or thread ID before retrying. If the head moves, do not resolve the
   old thread on the strength of the old verdict. If a reviewer reopens a thread later, treat its
   new content as pending rather than resolving it again automatically.

Before approval and before publishing the cumulative root repair record, re-read every corrected
thread. If one was reopened or gained a new reviewer objection while awaiting the user's answer,
pause and return to review even when the head SHA did not move. Publish the root record only after
every corrected thread is confirmed resolved or already resolved. It summarizes the repair; it
never replaces thread replies. Leave uncorrected or disputed threads open and report them
separately.

GitHub's [review-comment reply endpoint](https://docs.github.com/en/rest/pulls/comments#create-a-reply-for-a-review-comment)
and [review-thread resolution mutation](https://docs.github.com/en/graphql/reference/pulls#resolvereviewthread)
provide the two distinct writes. Bitbucket Cloud documents
[comment-thread resolution](https://developer.atlassian.com/cloud/bitbucket/rest/api-group-pullrequests/#api-repositories-workspace-repo-slug-pullrequests-pull-request-id-comments-comment-id-resolve-post),
while `bkt pr comments resolve --help` covers Cloud and Data Center. Verify the target forge's
actual response before treating either as supported in that repository.
