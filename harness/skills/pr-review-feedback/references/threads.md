# Review thread operations

Identify a discussion by its forge thread or top-level comment ID, never by a diff line that can
move after a push. Read every page of threads and every page of replies. A general comment may be
threaded on Bitbucket; classify by the forge's thread data, not by its position on the PR page.

## GitHub

1. Read `pullRequest.reviewThreads` through `gh api graphql`, including `pageInfo`, `id`,
   `isResolved`, `viewerCanResolve`, and the thread's `comments` with `databaseId`, `body`, `url`,
   `author` and `replyTo`. Paginate both connections. `gh pr view --json comments` contains PR
   timeline comments and does not replace this inventory.
2. For a corrected thread, draft a short reply naming the final commit and observed check. Add
   `<!-- pr-review-feedback:<pr>:<thread-id>:<sha12> -->` to the body. Search the whole thread for
   that marker from the publishing account. If it exists once, reuse it; a foreign or duplicate
   marker is a conflict to report. A later corrected head gets a new marker and reply, preserving
   earlier discussion.
3. Write the body to a file. Build a JSON payload with
   `jq -n --rawfile body "$reply_file" '{body:$body}' > "$payload_file"`, then call
   `gh api -X POST repos/<destination-owner>/<destination-repo>/pulls/<pr>/comments/<root-comment-id>/replies --input "$payload_file"`.
   The numeric ID must belong to the top-level review comment. Read the thread back and verify
   author, marker, URL and `replyTo` before resolving.
4. Re-read the PR head and entire thread. If the head still matches and no new objection appeared,
   call `resolveReviewThread` with the GraphQL thread ID, then query `isResolved` again. If
   `viewerCanResolve` is false or the mutation fails, leave the thread open and report it.

GitHub documents the [reply endpoint](https://docs.github.com/en/rest/pulls/comments#create-a-reply-for-a-review-comment)
and [resolution mutation](https://docs.github.com/en/graphql/reference/pulls#resolvereviewthread).

## Bitbucket

1. Select the recorded Bitbucket context and destination repository on every `bkt` call: pass
   `--context <context> --workspace <destination-workspace> --repo <destination-repo>` for Cloud,
   or `--context <context> --project <destination-project> --repo <destination-repo>` for Data
   Center. Use `bkt pr comments <pr> --details --json` and
   `bkt pr comments view <pr> <root-comment-id> --json` with those flags to inspect the root, replies, anchor and
   resolution state. `bkt pr comment <pr> --text "$(cat "$reply_file")" --parent <root-comment-id>`
   replies in that discussion; without `--parent` it creates a general PR comment. Use the same
   stable marker per PR, root comment ID and head SHA, then read the discussion back. Pass the same
   context and destination flags when replying.
2. Re-read the source head and discussion. Resolve with
   `bkt pr comments resolve <pr> <root-comment-id>` with the same context and destination flags
   only after verifying the reply. Read the
   thread back and require a resolved state. Bitbucket Cloud supports a resolution filter;
   Data Center does not expose it in the list, so inspect the thread directly. If the instance
   does not expose a reliable resolved state, report the gap rather than claiming closure.

The local `bkt pr comment --help` and `bkt pr comments resolve --help` cover Cloud and Data Center;
the target instance's response remains the execution evidence. Bitbucket Cloud also documents
[comment-thread resolution](https://developer.atlassian.com/cloud/bitbucket/rest/api-group-pullrequests/#api-repositories-workspace-repo-slug-pullrequests-pull-request-id-comments-comment-id-resolve-post).

## Partial failure and new feedback

- A timeout after a reply or resolution is uncertain. Query the same thread by marker and state
  before retrying. An unreadable thread stays pending; a root comment is no substitute.
- If the head moves between reply and resolution, stop and re-anchor. The reply may remain as
  history, but it does not justify closing the thread on a different head.
- If the reviewer reopens the thread or adds a new objection, treat that content as new feedback.
  Do not automatically resolve it again merely because an earlier reply exists.
- A standalone timeline comment or review body without a thread has no resolution operation.
  Respond at the PR level with a link to the original comment or review and marker
  `<!-- pr-review-feedback:<pr>:<timeline|review>:<comment-or-review-id>:<sha12> -->`. Read the destination PR's
  timeline comments for that marker from the publishing account before writing or retrying an
  uncertain write; reuse a matching response and report an unreadable or conflicting result.
  State that the original comment or review cannot be marked resolved.
