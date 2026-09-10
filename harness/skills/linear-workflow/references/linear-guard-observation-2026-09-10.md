# Linear identity guard experiment — 2026-09-10

## Result

Both authorized combined saves were executed on one disposable issue through the official Linear
MCP. **Both failed because identity replacement is rejected**, including the unique-anchor case.
The post-save reads showed unchanged state, description, `updatedAt`, and empty issue history.
This transport cannot use the proposed identity replacement as a workflow state guard.

The [anonymized request/response record](linear-guard-observation-2026-09-10.json) contains the live
contract excerpt, exact save arguments and errors, independent issue reads, comparisons, and
cleanup result. Responses marked `responseProjection` retain only relevant returned fields; no
omitted field is asserted verified. Placeholders replace workspace, team, user, issue, state and
history identifiers consistently; description strings and timestamps are unchanged.

| Case                            | Anchor occurrences | Requested transition | Returned result                             | Re-read outcome                                    |
| ------------------------------- | ------------------ | -------------------- | ------------------------------------------- | -------------------------------------------------- |
| Non-unique identity replacement | 2                  | Backlog → Todo       | `isError: true`; identical strings rejected | Backlog; identical description/date; history empty |
| Unique identity replacement     | 1                  | Backlog → Todo       | Same error                                  | Backlog; identical description/date; history empty |

Exact error returned for each save:

```text
Error: Patch failed, nothing was saved. Operation 1 (replace): old_string and new_string are identical
```

The first case does **not** establish rejection caused by non-uniqueness: the reported reason is
identity replacement. The second disproves the expected successful identity-guarded transition
on this inspected transport. No state write despite a failed patch was observed. Do not check
those unestablished success criteria in [#245](https://github.com/SebastienElet/dotfiles/issues/245).

## Transport and protocol

- Codex desktop, macOS `Darwin arm64`, zsh; 2026-09-10 UTC.
- Official endpoint `https://mcp.linear.app/mcp`, Streamable HTTP, JSON-RPC; negotiated protocol
  `2024-11-05`, server `Linear MCP` version `1.0.0`.
- Existing `linear 2.6.0` credential supplied in memory to the documented bearer header; no new
  login, plugin installation, or credential persisted in evidence. Network/keyring calls ran
  outside the sandbox.
- `initialize`, `notifications/initialized`, and `tools/list` succeeded. `save_issue` exposes
  `state` plus `patch`; its live patch documentation says a failing operation aborts the whole
  save and anchors must match exactly once. Schema admission is not execution evidence.
- `get_workspace`, `get_user(me)`, and `list_teams` identified one workspace and one team.
  GraphQL `viewer.id` and `organization.id` matched MCP before combining read transports.
- `list_issue_statuses` resolved Backlog and Todo to that team's IDs. The fixture was assigned to
  the authenticated user. No existing business issue was read or mutated.

The [official MCP documentation](https://linear.app/docs/mcp) permits direct bearer authentication
using an existing token. This path was missed by the
[previous CLI/GraphQL inspection](linear-guard-observation-2026-09-09.md); absence of integrated
MCP tools did not establish that the server itself was unavailable.

## Fixture, reads, and activity

The disposable issue was created at `2026-09-10T06:51:21.517Z` with two occurrences of
`ANCHOR-245-DUPLICATE` and one `ANCHOR-245-UNIQUE`. MCP and GraphQL independently retrieved the
created issue. Each test sent one `save_issue` with a target state ID and one `replace` whose
`old_string` and `new_string` were identical; neither supplied `description` nor `replace_all`.
The two calls were distinct test cases, not retries of a failed state decision.

The pre-test read began at `06:54:41Z`, over three minutes after creation. The
[GraphQL documentation](https://linear.app/developers/graphql) documents suppression of early
activity; testing outside that window avoids relying on it as the explanation for empty history.
After each save, MCP `get_issue` and GraphQL `issue` were independently called. GraphQL reads also
returned `history(first: 100)` with `hasNextPage: false`; no pagination was omitted.

All pre-cleanup reads returned:

- state `Backlog`, same state ID;
- the exact same 226 UTF-8 description bytes, SHA-256
  `f1dc38f61e3ebb9e61362d346560e51a3acdfd8c434647131f2457566ff58395`;
- `updatedAt: 2026-09-10T06:51:21.517Z`;
- `history.nodes: []`, `hasNextPage: false`.

A delayed GraphQL re-read at `06:55:35Z` returned the same values. Activity was observed through
Linear's issue-history API, not a rendered UI feed, notifications, or an indefinite monitoring
window. Because both saves failed, this establishes nothing about the timestamp/activity effect
of an accepted identity replacement. No concurrent writer was introduced; isolation and the
location of the anchor comparison relative to commit remain unverified.

## Cleanup

After those observations, `issueArchive(id: <issue-A>, trash: false)` returned `success: true`.
The independent read returned `archivedAt: 2026-09-10T06:55:37.389Z`, unchanged Backlog state and
description, and one new history entry. That entry followed cleanup and is not attributed to either
failed save. The disposable issue is archived, retained for inspection, not deleted.

## Consequence for the skill

The live schema supplies a documented whole-save contract; both actual identity replacements are
observed rejected with unchanged stored values. The skill removes this identity guard as an enabled
recipe for the observed transport and requires no workflow state write until an alternative
conditional mechanism is verified. No description-changing workaround, parent-child mechanism,
or CI runner is introduced.

Author: `/root`; independent claim auditor: `/root/guard_audit`.

| claim                                                   | authority                      | mechanism                           | scope                         | named behavioral oracle               | status       | failure mechanism                                            |
| ------------------------------------------------------- | ------------------------------ | ----------------------------------- | ----------------------------- | ------------------------------------- | ------------ | ------------------------------------------------------------ |
| Identical replacement strings are rejected              | Two live MCP responses         | Identical-string error              | Two calls, this endpoint/date | Both save responses in linked record  | held         | Identity guard cannot execute                                |
| Rejected calls changed nothing observed                 | Before/after independent reads | Compare state, bytes, date, history | Those failures on one issue   | Both MCP and GraphQL post-reads       | held         | Does not prove later-stage rollback                          |
| Duplicate case establishes rejection for non-uniqueness | #245 expected outcome          | Unique-match validation             | Duplicate branch              | Error names identical strings instead | contradicted | Non-uniqueness was not the reported cause                    |
| Unique identity replacement succeeds                    | #245 expected outcome          | Combined state and identity patch   | Unique branch                 | Rejected response and unchanged reads | contradicted | Identity replacement is rejected                             |
| Every failed patch rolls back state                     | Live whole-save contract; #245 | Transactional rollback              | All failure stages            | None beyond these validation failures | debt         | Mutations may not have begun in observed cases               |
| Guard closes concurrent stale-read race                 | No isolation authority         | Conditional isolated write          | Concurrent writers            | None                                  | contradicted | Neither concurrency nor serialization boundary was exercised |
