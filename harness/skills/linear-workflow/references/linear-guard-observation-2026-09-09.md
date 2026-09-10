# Linear guard inspection — 2026-09-09

Follow-up: [the 2026-09-10 MCP experiment](linear-guard-observation-2026-09-10.md) reached the official
server directly with existing authentication and executed both cases; this earlier record covers
only the initial CLI/GraphQL inspection, not exhaustion of available transports.

## Result and scope

Follow-up to [dotfiles #245](https://github.com/SebastienElet/dotfiles/issues/245), whose body and
empty comment list were read before editing. Baseline commit:
`0543d6f0395db97a61952eee6cb48173e0356d4f`.

**Blocked experiment, not a successful guard observation.** No Linear connector tools were exposed
in this Codex desktop session. The installed third-party CLI `linear 2.6.0` could authenticate
outside the sandbox and query Linear GraphQL. Its inspected update surfaces do not express the
requested combined state change and anchored identity replacement. No disposable issue was
created, no existing business issue was read or modified, and no Linear mutation was submitted.
There is therefore no disposable issue to archive or delete.

| Required observation                                                      | Result                                        |
| ------------------------------------------------------------------------- | --------------------------------------------- |
| Non-unique anchor rejects save; state unchanged after re-read             | Not executed; no anchored save available      |
| Unique anchor changes state; description strictly identical after re-read | Not executed; same limitation                 |
| `updatedAt` before/after either save                                      | Not observed                                  |
| Activity before/after either save                                         | Not observed                                  |
| State changes despite patch failure                                       | Not observed; neither confirmed nor disproved |

These results cover CLI 2.6.0 help and the inspected GraphQL schema in one authenticated workspace,
not an unavailable connector or all Linear clients. No state-write guarantee is established.
Issue #245 remains open for the two mutation observations.

## Environment and anonymization

- Date: 2026-09-09; schema inspection checkpoint `2026-09-09T14:09:03Z`.
- Environment: Codex desktop local worktree, macOS (`Darwin arm64`), zsh; CLI installed via Homebrew.
- Transport: `linear api --workspace <workspace-A>` to Linear GraphQL; remote MCP unavailable in
  the session tool inventory (`ALL_TOOLS` search for Linear tools returned none).
- Workspace and user names, email, URLs, and local account paths are replaced by placeholders.
  No credentials were printed or copied into this record. Server schema field names are unchanged.
- Initial sandbox authentication failed; the same read outside the sandbox succeeded. The initial
  failure must not be reported as missing credentials on the host.

## Relevant requests and responses

CLI probes, with output excerpts and exit status:

```text
linear --version
linear 2.6.0                                      [exit 0]

linear auth list                                [sandbox, exit 0]
* <workspace-A> missing credentials

linear auth whoami --workspace <workspace-A>     [sandbox, exit 1]
Failed to get user info: Workspace "<workspace-A>" not found in credentials.

linear auth whoami --workspace <workspace-A>     [outside sandbox, exit 0]
Workspace: <workspace-A>
  Slug: <workspace-A>
  URL: https://linear.app/<workspace-A>
User: <user-A>
  Display name: <user-A>
  Email: <redacted>

linear issue update --help                      [exit 0]
--description <description>    Description of the issue
--description-file <path>      Read description from a file
--state <state>                Workflow state for the issue (by name or type)

linear team states --help                       [exit 0]
List workflow states for a team
--json                        Output as JSON

linear schema --workspace <workspace-A> --output <temporary-schema.graphql>
Schema written to <temporary-schema.graphql>     [outside sandbox, exit 0]
```

The complete update help exposed no `patch`, `old_string`, `replace_all`, or conditional-write
option. Team-state help was inspected, not a team's states enumerated. The emitted SDL's SHA-256
was `6dfb06a6589aee185ac74282241e6f7efcf200bca9da94cecc3e6dc4a6b8b81f`;
its relevant declaration was `issueUpdate(id: String!, input: IssueUpdateInput!): IssuePayload!`.
The hash identifies that local output, not its freshness; the following independent server queries
checked the live input fields and mutation arguments.

Executed with `linear api --workspace <workspace-A>` outside the sandbox, exit 0:

```graphql
query GuardTransportSchema {
  __type(name: "IssueUpdateInput") {
    name
    inputFields {
      name
    }
  }
}
```

Complete response:

```json
{
  "data": {
    "__type": {
      "name": "IssueUpdateInput",
      "inputFields": [
        { "name": "title" },
        { "name": "description" },
        { "name": "descriptionData" },
        { "name": "assigneeId" },
        { "name": "delegateId" },
        { "name": "parentId" },
        { "name": "priority" },
        { "name": "estimate" },
        { "name": "subscriberIds" },
        { "name": "labelIds" },
        { "name": "addedLabelIds" },
        { "name": "removedLabelIds" },
        { "name": "releaseIds" },
        { "name": "addedReleaseIds" },
        { "name": "removedReleaseIds" },
        { "name": "teamId" },
        { "name": "cycleId" },
        { "name": "projectId" },
        { "name": "projectMilestoneId" },
        { "name": "lastAppliedTemplateId" },
        { "name": "stateId" },
        { "name": "sortOrder" },
        { "name": "prioritySortOrder" },
        { "name": "subIssueSortOrder" },
        { "name": "dueDate" },
        { "name": "inheritsSharedAccess" },
        { "name": "trusted" },
        { "name": "trashed" },
        { "name": "slaBreachesAt" },
        { "name": "slaStartedAt" },
        { "name": "snoozedUntilAt" },
        { "name": "snoozedById" },
        { "name": "slaType" },
        { "name": "autoClosedByParentClosing" }
      ]
    }
  }
}
```

A first broader introspection failed; this was a read-only query, not a failed patch:

```graphql
query GuardMutationSchema {
  __schema {
    mutationType {
      fields {
        name
        args {
          name
          type {
            kind
            name
            ofType {
              kind
              name
            }
          }
        }
      }
    }
  }
}
```

```json
{
  "errors": [
    {
      "message": "Query too complex",
      "extensions": {
        "type": "invalid input",
        "code": "INPUT_ERROR",
        "statusCode": 400,
        "userError": true,
        "userPresentableMessage": "The query is too complex. Complexity: 16384. Maximum allowed complexity: 10000.",
        "http": { "status": 400 }
      }
    }
  ]
}
```

CLI exit 1, diagnostic on stderr; the empty stdout was not evidence. A smaller read succeeded
(exit 0; non-null required data and no `errors`):

```graphql
query GuardMutationSchema {
  __schema {
    mutationType {
      fields {
        name
        args {
          name
        }
      }
    }
  }
}
```

Response excerpt selected from `data.__schema.mutationType.fields`:

```json
{ "name": "issueUpdate", "args": [{ "name": "input" }, { "name": "id" }] }
```

No returned mutation name contained `patch` (case-insensitive). This is a bounded schema
inspection, not proof that no other conditional mechanism can exist. No issue save request,
save response, or post-save issue re-read exists for either case; both remain unexecuted.

## Documentation versus observation

The [official GraphQL guide](https://linear.app/developers/graphql), read on 2026-09-09, documents
the endpoint, introspection, `issueUpdate`, and the need to inspect errors even on HTTP 200.
It also documents omission of property changes from activity during the first three minutes
following issue creation. That activity behavior was **documented only**, not measured here.
A future probe must record issue age and inspect activity after that creation window.

The [official MCP page](https://linear.app/docs/mcp), read on the same date, documents authenticated
remote access and update tools. It does not supply the specific `save_issue` identity-replacement
schema or whole-save rollback guarantee needed here. The baseline skill attributed that guarantee
to a connector contract without preserving the contract; that attribution remains unverified.
Neither a valid payload nor a GraphQL validation error proves state/patch atomicity.

## Independent claim audit and skill validation

Author: `/root`. Independent auditor: `/root/guard_audit` (`design-claim-auditor`).
The ledger below concerns the baseline assertions, before their replacement; `contradicted` is the
audit status for an unsupported guarantee, not an observation of partial state mutation.

| claim                                                         | authority                                                             | mechanism                                         | scope                                | named behavioral oracle                                 | status       | failure mechanism                                                                      |
| ------------------------------------------------------------- | --------------------------------------------------------------------- | ------------------------------------------------- | ------------------------------------ | ------------------------------------------------------- | ------------ | -------------------------------------------------------------------------------------- |
| Failed anchor prevents accompanying state transition          | Baseline completion-evidence lines 167–173; #245                      | One save containing state and anchored patch      | Same issue state and description     | None green; live IssueUpdateInput lacks patch/condition | contradicted | Inspected path cannot encode guard; separate state update can use stale classification |
| Unchanged replacement acts as identity guard                  | Baseline completion-evidence lines 191–209; #245                      | Unique contiguous identity replacement with state | Same issue non-empty description     | None green; no acceptance or rejection observation      | contradicted | Missing or rejected patch does not prove state rollback                                |
| Schema acceptance plus documentation authorizes guarded write | Baseline transports lines 11–14 and completion-evidence lines 171–173 | Capability inspection and reported contract       | Hypothetical authenticated connector | Baseline reading scenario only                          | contradicted | Documentation may differ from runtime                                                  |

Baseline reading scenario: connector accepts `state` and `patch`, documentation claims atomicity,
but no live rejection observation exists. The independent agent answered that the baseline text
permits the write because it relies on the connector documentation. This is a skill-reading
result, not a Linear experiment. The replacement requires documented write-boundary semantics
and the two live observations before enabling this state write.
