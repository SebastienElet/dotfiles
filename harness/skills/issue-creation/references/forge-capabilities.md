# Forge Capabilities

Use this reference after the target forge is identified. Capabilities and authentication can change;
probe the current environment and consult the provider's official documentation before depending on
a command or field.

## Provider-neutral selection

1. Prefer an available purpose-built connector with explicit issue search, create, and retrieve
   operations.
2. Otherwise use an authenticated forge CLI whose current help and official documentation confirm
   those operations.
3. Use a provider API only when its authentication and response contract are already available and
   verified for the task.
4. If no safe write path exists, complete draft and validation work, name the missing capability,
   and do not simulate publication.
5. Never install a client, add credentials, or create test issues on a real service without separate
   authorization.

For any provider, search both open and closed work, submit multiline bodies through structured
connector fields or a file, and retrieve the created issue independently. Parse structured output
when available. Treat non-zero exits, transport failures, GraphQL `errors`, missing identifiers,
unknown shapes, null required fields, empty bodies, and mismatched targets as failures rather than
coercing them into success.

## GitHub

Probe `gh version`, `gh auth status`, and command help. With a repository target:

- search with `gh search issues <terms> --repo OWNER/REPO --state open --json ...` and repeat for
  closed issues, or use an equivalent connector search that covers both states;
- create with `gh issue create --repo OWNER/REPO --title <title> --body-file <file>` and only the
  validated optional metadata;
- retrieve with `gh issue view <number-or-url> --repo OWNER/REPO --json ...`.

Request and compare the fields relevant to the candidate, including `title`, `body`, `state`,
`labels`, `assignees`, `milestone`, relationships, `url`, and `number` when supported. Additional
project-field writes may require separately verified authorization scopes.

Official documentation:

- [Create](https://cli.github.com/manual/gh_issue_create)
- [Search](https://cli.github.com/manual/gh_search_issues)
- [Retrieve and JSON fields](https://cli.github.com/manual/gh_issue_view)

## GitLab

Probe `glab version`, `glab auth status`, and command help. With a project target:

- search with `glab issue list -R GROUP/PROJECT --all --search <terms> --in title,description
--output json`, or an equivalent connector search;
- create with `glab issue create -R GROUP/PROJECT --title <title> --description-file <file> --yes`
  and only verified optional flags;
- retrieve with `glab issue view <iid-or-url> --repo GROUP/PROJECT --output json`.

The documented JSON surface may not promise a stable field list. Compare fields actually returned;
use a verified connector or GitLab Issues API only when required, and report omitted labels,
relationships, or other requested fields as not verified.

Official documentation:

- [CLI and authentication](https://docs.gitlab.com/cli/)
- [Create](https://docs.gitlab.com/cli/issue/create/)
- [List](https://docs.gitlab.com/cli/issue/list/)
- [Retrieve](https://docs.gitlab.com/cli/issue/view/)

## Linear

Prefer Linear's official connector when it is available and authenticated. Verify that its current
tools cover issue search, creation, and retrieval; the read-only endpoint cannot publish. When using
the GraphQL API, query the exact fields needed for duplicate search and verification, call
`issueCreate` for publication, and inspect both the mutation's success value and top-level GraphQL
`errors`, including on HTTP 200 responses.

Retrieve the created issue with a separate query and compare its identifier, title, description,
URL, state, labels, relations, team, and project to the extent the current schema exposes them. A
third-party `linear` binary is not the official Linear CLI: inspect its provenance, authentication,
help, and limitations before deciding whether it is an acceptable available path.

Official documentation:

- [Linear MCP](https://linear.app/docs/mcp)
- [Linear GraphQL API](https://linear.app/developers/graphql)

## Other compatible forges

Apply the same lifecycle only after verifying a search surface covering open and closed issues, one
create operation, one independent retrieval operation, authentication, target selection, and the
actual fields returned. Provider-specific limitations change the verification report, not the
coherence or authority rules. If any mandatory operation is missing, stop before publication and
report it.
