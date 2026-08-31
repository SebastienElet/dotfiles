# Memory Entry Contract

Read this contract before proposing, admitting, retrieving, or confirming durable memory.

## Admission

Admit an entry only when all of these hold:

- its statement is durable within the smallest named project or user scope;
- ordinary code, configuration, or documentation discovery does not reveal it cheaply;
- a current primary source establishes it;
- retaining it avoids material repeated investigation;
- an observable oracle can detect when it no longer holds;
- it contains no raw transcript, complete private prompt, secret, credential, or workaround for a
  defect in owned code.

Project is the default scope. User scope requires an explicit user persistence request. A primary
source may be a tracked Git file, supported local file, official URL, or explicit user decision.
Keep only the durable statement and source-backed proof; omit incident narrative and volatile state.

## Complete draft

Return this complete YAML before admission. Use one kind from `goal`, `decision`, `evidence`,
`invariant`, `unknown`, or `assumption`; use `project` or `user` scope. Omit `automated` only when no
supported source-fingerprint oracle applies.

```yaml
schema_version: 1
kind: invariant
statement: <one durable statement>
scope: project
retrieval_terms:
  - <term that future task prompts will contain>
proof:
  summary: <how the source establishes the statement>
  sources:
    - kind: git-file
      locator: <stable repository-relative path>
oracle:
  automated:
    kind: source-fingerprint
    expected: all-proof-sources-unchanged
  human_fallback:
    question: <question answered when automated proof is unavailable>
    valid_when: <observable condition that keeps the statement valid>
  outcomes:
    valid: <meaning of a valid verdict>
    invalidated: <meaning of an invalid verdict>
```

Source kinds are `git-file`, `local-file`, `official-url`, and `user-decision`. The runtime assigns
the ID, resolved scope key, source fingerprints, status, and timestamps.

## Commands

Pass the full value on stdin and wait for completion:

```bash
printf '%s' "$query" | agent-memory retrieve --query-stdin --format json
printf '%s' "$draft" | agent-memory admit --format json
printf '%s' "$reason" | agent-memory confirm --id "$id" --status "$status" --reason-stdin
agent-memory audit --include-terminal --format json
```

Valid human statuses are `achieved` or `abandoned` for goals, `superseded` for decisions,
`resolved` for unknowns, and `confirmed` for assumptions. Evidence and invariants have no human
terminal status. Exit `0` means success or duplicate, `2` means usage error or rejection, `3` means
conflict, and `4` means unavailable. Stdout is JSON; stderr diagnostics are redacted JSON.
