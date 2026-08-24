# Linear Transport Adapters

Select a transport at runtime. The business workflow remains in its composing skill and
`linear-workflow`.

## Selection

1. Prefer an authenticated Linear connector exposed by the current agent when its current schema
   covers identity, filtered issue listing with pagination, issue details, hierarchy, blocking
   relations, links or attachments, workflow state updates, and URL attachment.
2. Otherwise use the installed `linear` CLI only after its local help confirms the commands and
   `linear auth whoami` succeeds for the intended workspace.
3. Otherwise use Linear GraphQL only when authentication and the current schema are already
   available and every response is checked for transport failure, top-level errors, null required
   fields, and mutation success.
4. Stop before the first uncovered operation. Never combine partial transports unless identity and
   workspace equality are proven.

## Connector adapter

- Inspect the connector's tools in the current session; configuration or marketplace availability
  alone does not prove activation or authentication.
- Retrieve issue data and relations with structured fields. Resolve attachment URLs without
  inferring their meaning from prose.
- After a state or link mutation, retrieve the issue independently and compare the stored value.

## CLI adapter

The locally verified `linear` 2.5.0 surface provides:

- `linear auth whoami --workspace <workspace>` for identity and authentication;
- `linear issue query --workspace <workspace> --assignee <username> --all-states --limit 0 --json`
  for an exhaustive structured list of one user's issues;
- `linear issue view <issue-id> --workspace <workspace> --json` for structured issue, hierarchy,
  inverse relations, and attachments;
- `linear issue relation list <issue-id> --workspace <workspace>` to inspect relations;
- `linear issue update <issue-id> --workspace <workspace> --state <state>` for lifecycle
  transitions;
- `linear issue link <issue-id> <pull-request-url> --workspace <workspace>` for URL attachment.

Run current local help again before use. This binary is a third-party Linear CLI, not an official
Linear CLI. Do not use `linear issue pr`: its installed documentation delegates to GitHub's `gh`,
so it cannot create the required Bitbucket pull request.

## Fallback limits

- An unauthenticated CLI is unavailable even when it has stored workspace names.
- A read-only connector cannot update an issue or attach a pull request.
- Do not install a plugin, add a connector, log in, or create test data merely to complete probing.
