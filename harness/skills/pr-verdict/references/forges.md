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

| Step                | GitHub (`gh`)                                                                                                                                                   | Bitbucket (`bkt`)                                                                                                  |
| ------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| Metadata            | `gh pr view <n> --json headRefOid,headRefName,headRepository,headRepositoryOwner,isCrossRepository,maintainerCanModify,baseRefName,mergeStateStatus,title,body` | `bkt pr view <n> --json`                                                                                           |
| Head SHA            | `.headRefOid`                                                                                                                                                   | Cloud `.pull_request.source.commit.hash`, DC `.fromRef.latestCommit` — confirm the path against the actual payload |
| Fetch the head      | `gh pr checkout <n>`                                                                                                                                            | `bkt pr checkout <n>`                                                                                              |
| CI state            | `gh pr checks <n>`                                                                                                                                              | `bkt pr checks <n>`                                                                                                |
| Diff                | `gh pr diff <n>`                                                                                                                                                | `bkt pr diff <n>`                                                                                                  |
| Existing comments   | `gh pr view <n> --json comments`                                                                                                                                | `bkt pr comments <n> --json`                                                                                       |
| Publish             | `gh pr comment <n> --body-file verdict.md`                                                                                                                      | `bkt pr comment <n> --text "$(cat verdict.md)"`                                                                    |
| Update              | `gh api -X PATCH /repos/{owner}/{repo}/issues/comments/<id> -F body=@verdict.md`                                                                                | `bkt api -X PUT /2.0/repositories/{ws}/{repo}/pullrequests/<n>/comments/<id> --input -`                            |
| Enforceable verdict | `gh pr review <n> --request-changes --body-file verdict.md` — refused on your own PR                                                                            | no reliable equivalent                                                                                             |

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
  reach the shell as syntax:

```sh
jq -n --rawfile b verdict.md '{content: {raw: $b}}' |
  bkt api -X PUT /2.0/repositories/{ws}/{repo}/pullrequests/1042/comments/1001 --input -
```

Data Center takes `{"text": …, "version": <n>}` instead, and rejects the update without the
comment's current `version` — read it from `bkt pr comments <n> --json` first.

## Finding a previous verdict

The marker is the lookup key. Search the comment bodies for `<!-- pr-verdict:` before publishing:

```sh
gh pr view 1042 --json comments \
  --jq '.comments[] | select(.body | startswith("<!-- pr-verdict:")) | {id, body: .body[0:60]}'

bkt pr comments 1042 --json --jq '.[] | select(.content.raw // .text | startswith("<!-- pr-verdict:"))'
```

Same `<pr>:<sha>` → update that comment. Different SHA → publish a new verdict and leave the old one
in place; it is the record of what was judged on the superseded head.

## The two forges are not equivalent on enforcement

On GitHub, _changes required_ is a repository state: `gh pr review --request-changes` blocks the
merge button until it is dismissed, so the comment text is documentation of a block that already
exists.

Bitbucket has no equivalent an agent can set reliably — `bkt pr approve` exists, its negation does
not; declining a PR is a different act with a different meaning. There the comment **is** the
verdict, which is why the closing sentence ("do not approve or merge this head") carries the whole
enforcement and must never be softened or dropped.

GitHub falls back to that same Bitbucket situation whenever the authenticated account authored the
PR: it rejects the call with `Can not request changes on your own pull request`, and there is no
flag around it. `gh pr view --json` exposes no `viewerDidAuthor` field (checked on `gh` 2.97), so
compare the author against the token yourself, in phase 1 —

```sh
test "$(gh pr view 1042 --json author --jq .author.login)" = "$(gh api user --jq .login)" &&
  echo "own PR: no native blocking state"
```

When it is your own PR, publish the comment alone and state inside the verdict that nothing
mechanical holds the merge button. A blocking verdict whose reader believes the forge is enforcing
it, while the forge is not, is worse than no verdict: it buys a false sense of a gate.
