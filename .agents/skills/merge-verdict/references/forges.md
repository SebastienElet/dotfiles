# Forges

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

| Step | GitHub (`gh`) | Bitbucket (`bkt`) |
| --- | --- | --- |
| Metadata | `gh pr view <n> --json headRefOid,baseRefName,mergeStateStatus,title,body` | `bkt pr view <n> --json` |
| Head SHA | `.headRefOid` | Cloud `.pull_request.source.commit.hash`, DC `.fromRef.latestCommit` — confirm the path against the actual payload |
| Fetch the head | `gh pr checkout <n>` | `bkt pr checkout <n>` |
| CI state | `gh pr checks <n>` | `bkt pr checks <n>` |
| Diff | `gh pr diff <n>` | `bkt pr diff <n>` |
| Existing comments | `gh pr view <n> --json comments` | `bkt pr comments <n> --json` |
| Publish | `gh pr comment <n> --body-file verdict.md` | `bkt pr comment <n> --text "$(cat verdict.md)"` |
| Update | `gh api -X PATCH /repos/{owner}/{repo}/issues/comments/<id> -F body=@verdict.md` | `bkt api -X PUT /2.0/repositories/{ws}/{repo}/pullrequests/<n>/comments/<id> --input -` |
| Enforceable verdict | `gh pr review <n> --request-changes --body-file verdict.md` | no reliable equivalent |

`bkt` covers both Bitbucket Cloud and Data Center; the two differ on JSON shape and on some flags
(`bkt pr comments --state` is Cloud-only). Verified against `gh` 2.97 and `bkt` as installed by this
repository — re-check `--help` when a command is rejected rather than guessing a flag.

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

The marker is the lookup key. Search the comment bodies for `<!-- merge-verdict:` before publishing:

```sh
gh pr view 1042 --json comments \
  --jq '.comments[] | select(.body | startswith("<!-- merge-verdict:")) | {id, body: .body[0:60]}'

bkt pr comments 1042 --json --jq '.[] | select(.content.raw // .text | startswith("<!-- merge-verdict:"))'
```

Same `<pr>:<sha>` → update that comment. Different SHA → publish a new verdict and leave the old one
in place; it is the record of what was judged on the superseded head.

## The two forges are not equivalent on enforcement

On GitHub, *changes required* is a repository state: `gh pr review --request-changes` blocks the
merge button until it is dismissed, so the comment text is documentation of a block that already
exists.

Bitbucket has no equivalent an agent can set reliably — `bkt pr approve` exists, its negation does
not; declining a PR is a different act with a different meaning. There the comment **is** the
verdict, which is why the closing sentence ("do not approve or merge this head") carries the whole
enforcement and must never be softened or dropped.
