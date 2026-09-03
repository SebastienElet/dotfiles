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

## Validation rules before admission

The runtime accepts one UTF-8 YAML mapping, at most **1,048,576 bytes (1 MiB)** including YAML
syntax and whitespace. JSON is also accepted as YAML. Mapping keys must be unique; extra keys are
rejected at every schema level. Do not submit runtime-assigned fields (`id`, `status`, timestamps,
fingerprints, or transitions). `schema_version` is the integer `1`; other versions are rejected.

Text limits below are inclusive and count **Unicode scalar values**, as Rust `str.chars()` does:
not UTF-8 bytes, grapheme clusters, or tokens. An accented scalar counts once; a combining sequence
counts each scalar separately. Text is checked before identity normalization; whitespace counts
toward length. Persisted text fields reject non-string scalars (including numbers, booleans, null),
mappings, sequences, and custom YAML tags; a custom tag on top-level `kind` is historically accepted. Use quoted strings when a YAML scalar could otherwise be interpreted as another type.

| Field                     | Required shape and bounds                                                                                      |
| ------------------------- | -------------------------------------------------------------------------------------------------------------- |
| `kind`                    | `goal`, `decision`, `evidence`, `invariant`, `unknown`, or `assumption`                                        |
| `statement`               | String, 1–500 Unicode scalar values                                                                            |
| `scope`                   | `project` or `user`; omission defaults to `project`                                                            |
| `retrieval_terms`         | Sequence of 1–20 strings, each 1–100 Unicode scalar values                                                     |
| `proof`                   | Mapping with `summary` and `sources`                                                                           |
| `proof.summary`           | String, 1–1,000 Unicode scalar values                                                                          |
| `proof.sources`           | Sequence of 1–20 mappings, each with `kind` and `locator`                                                      |
| `proof.sources[].kind`    | `git-file`, `local-file`, `official-url`, or `user-decision`                                                   |
| `proof.sources[].locator` | String satisfying the source rules below; no separate text-length cap beyond the document limit                |
| `oracle`                  | Mapping with `human_fallback`, `outcomes`, and `automated` when required                                       |
| `oracle.automated`        | When present and non-null: mapping with `kind: source-fingerprint` and `expected: all-proof-sources-unchanged` |
| `oracle.human_fallback`   | Mapping with `question` and `valid_when`, each a string of 1–500 Unicode scalar values                         |
| `oracle.outcomes`         | Mapping with `valid` and `invalidated`, each a string of 1–500 Unicode scalar values                           |

Only `scope` and `oracle.automated` may be omitted. A missing or null automated oracle is accepted
only when **every** proof source is a `user-decision`; the human fallback and both outcomes remain
mandatory. Empty source and retrieval-term sequences are rejected. The 500-scalar statement cap
keeps the retrieved statement bounded; put supporting detail in the proof and primary source rather
than silently truncating a human-approved statement.

### Source prerequisites

- `git-file`: a nonempty relative path from the command's current directory, using normal path
  components (no parent traversal or leading `./`). The resolved file must remain within the current
  Git worktree and be tracked by Git. Admission stores a worktree-relative locator. User-scoped
  entries cannot use Git sources. Project identity requires an accessible, unambiguous, absolute
  canonical `git-common-dir`; worktrees share an identity, separate clones and moved repositories
  do not.
- `local-file`: an absolute path. Local and Git sources must exist and end in a regular file, not a
  symlink or directory; the parent directory is canonicalized. Their bytes must be readable and
  total at most 1,048,576. Missing files are rejected; an oversized file or an access/tool failure
  is unavailable (exit `4`).
- `official-url`: HTTPS with a domain name, no literal IP, credentials, or fragment. The final URL
  must satisfy the same restrictions. Fetching permits at most five HTTPS redirects, five seconds
  to connect, fifteen seconds overall, and 1,048,576 response-body bytes. A successful response must
  be HTTP 2xx; HTTP 404/410 is rejected, other fetch failures are unavailable. The draft must also
  contain a `user-decision` source recording the human decision that the domain is authoritative;
  a successful fetch does not establish officiality. `curl` and Git must be available when used.
- `user-decision`: the locator is the explicit human decision text, fingerprinted without fetching.
  The runtime checks its type and sensitive/executable content; the human and skill establish its
  substance and authority. An empty locator is not a useful proof even though it has no separate
  minimum-length validator.

All proof sources are rechecked during admission. A changed fingerprint produces a conflict;
revalidate the proof before resubmitting. An unavailable dependency cannot be repaired by shortening
or weakening the draft: preserve the accepted draft and restore the dependency.

### Content exclusions

Every persisted text sink is checked: statement, retrieval terms, proof summary, source locators,
human fallback, outcomes, and confirmation reason. Diagnostics never include their values.
The existing deterministic sensitive checks are ASCII-case-insensitive and recognize:

- private-key PEM headers;
- URL authority userinfo containing `@`;
- `authorization` followed by a colon;
- assignments to `password`, `secret`, `token`, or `api` + separator + `key`, using `=` or `:`;
  API-key separators may be whitespace, `_`, `-`, or `.`;
- credential prefixes `sk-`, `ghp_`, `github_pat_`, `xoxb-`, `xoxa-`, `xoxp-`, `xoxr-`, `xoxs-`;
- prompt markers `system prompt:`, `<|system|>`, `[system]`, or `begin system prompt`;
- at least two lines prefixed by `user:`, `assistant:`, or `system:` after optional whitespace or
  Markdown list/quote/heading markers.

Assignments and credential prefixes use lexical boundaries; they do not reject every incidental
word containing these letters. These are named-pattern checks, not universal detection of secrets
or unmarked private prompts/transcripts. The skill's broader refusal rule still applies even when
the runtime would accept the text; do not encode or disguise sensitive content to evade detection.

Executable text is also refused: command substitution opening `$(`, shell shebangs, shell code
fences, lines starting with `$ ` or `% `, `sh`/`bash`/`zsh`/`fish` with `-c` and a command, or a pipe
into one of those shells. Code-fence languages include `shell`; executable paths use their basename.
Keep a declarative summary instead of executable material.

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

## Diagnosing a failed command

Every CLI error retains `error.code` and `error.field` and adds a redacted English `error.message`
that names the failed criterion and the next action. Length/count failures also return inclusive
`minimum`, `maximum`, and `unit` (`unicode_scalars`, `items`, or `bytes`). A list member may carry
`item_index` (zero-based); YAML errors may carry `line` and `column` (one-based parser locations,
sometimes the beginning of the containing mapping). Field paths contain only known schema names;
unknown keys, values, source paths, URLs, prompts, transcripts, and parser excerpts are never echoed.

For example, an overlong statement returns exit `2` and this diagnostic:

```json
{
  "error": {
    "code": "invalid_field",
    "field": "statement",
    "message": "Adjust the string length to the inclusive bounds; length counts Unicode scalar values, not bytes, graphemes or tokens.",
    "minimum": 1,
    "maximum": 500,
    "unit": "unicode_scalars"
  }
}
```

Use the supplied criterion and bounds to repair the draft locally. Preserve its accepted meaning;
if repair changes the claim, proof, scope, or oracle substantively, present the revised draft for
acceptance before admission. The command reports the first failure in validation order, so check
the complete contract before resubmitting. There is no dry-run/validate command: never use repeated
`admit` calls to discover a constraint, because a successful call persists an entry.

| Rejection code                                   | Correction                                                                               |
| ------------------------------------------------ | ---------------------------------------------------------------------------------------- |
| `invalid_field`                                  | Supply the named required field with the expected type/value or obey the returned bounds |
| `missing_proof`                                  | Add at least one primary proof source                                                    |
| `missing_oracle`                                 | Supply the source-fingerprint oracle unless all sources are user decisions               |
| `unsupported_schema`                             | Use integer schema version 1                                                             |
| `too_many_items`                                 | Reduce the named sequence to its returned maximum                                        |
| `sensitive_content`                              | Remove the forbidden content; retain a safe summary without encoding the original        |
| `shell_command`                                  | Replace executable material with a declarative statement                                 |
| `invalid_source_kind`                            | Select one of the four supported source kinds                                            |
| `unknown_field`                                  | Remove keys outside the named mapping's allowed set; omit runtime-assigned fields        |
| `duplicate_field`                                | Keep each mapping key once near the reported location                                    |
| `malformed_yaml`                                 | Repair the document syntax near the reported location                                    |
| `input_too_large`                                | Reduce the full serialized input to 1 MiB                                                |
| `invalid_utf8`, `empty_stdin`, `empty_query`     | Supply valid UTF-8 and the required nonempty text/document                               |
| `source_invalid`                                 | Follow the specific source criterion in the message and the source index when provided   |
| `scope_unavailable`, `scope_mismatch`            | Run from the authorized Git project; do not switch to user scope without consent         |
| `admission_not_authorized`                       | Obtain explicit persistence authorization                                                |
| `invalid_memory_id`, `entry_not_found`           | Use the ID returned by successful admission in the same configured store                 |
| `invalid_transition_reason`                      | Supply a nonblank, bounded, declarative reason                                           |
| `invalid_human_conclusion`, `entry_not_terminal` | Use a human terminal status compatible with the actual entry kind                        |
| `invalid_arguments`                              | Follow the command syntax and closed option values given in the message                  |
| `missing_hook_*`, `invalid_hook_*`               | Repair the named hook input criterion described below                                    |

`entry_conflict`, `source_changed`, `entry_not_active`, `selection_stale`, and `store_lock_timeout`
remain conflicts (exit `3`); their messages distinguish reconciliation, source revalidation,
terminal-state refusal, fresh retrieval, and waiting for another writer. Storage, permission,
oracle, source, input/output, trace, and retrieval-deadline failures remain unavailable (exit `4`).
Preserve the draft on failure, do not claim it was stored, and never repair the store by directly
editing files. After `output_unavailable` or a trace failure, the write may already have completed;
restore the channel and reconcile rather than assuming nothing was persisted.

## Retrieve, confirm, and hook inputs

`retrieve` requires nonempty UTF-8 stdin with at least one non-whitespace character and the common
1 MiB limit. It does not persist the query or run persisted-text rejection on it. Memory omitted
by relevance, limits, freshness, or oracle verdict is not an input rejection; consume only the
successfully injected subset. Stored-document validation failures remain omitted, with their check
codes, rather than exposing invalid stored content. An empty result is not proof of admission failure.

`confirm` requires an ID of `mem_` followed by exactly 24 lowercase hexadecimal digits, an active
entry, and a human status compatible with its kind. Its reason is UTF-8, 1–500 Unicode scalar values,
contains a non-whitespace character, and obeys all persisted-text exclusions. It must arrive on
stdin with `--reason-stdin`. Terminal entries cannot be reactivated. Generated transition records
start from `active`, end at the resulting status, and have verdict `valid` for human conclusions or
`invalid` for automated invalidation.

`hook --agent codex|claude` accepts one JSON object of at most 1 MiB, with these required strings:

- `hook_event_name`: exactly `UserPromptSubmit`;
- `prompt`: the complete prompt, nonempty after trimming whitespace;
- `cwd`: a nonempty absolute path, without NUL or parent traversal.

Do not duplicate those keys. Extra hook fields are allowed. The hook does not persist its prompt
and returns only a sanitized injection; its overall retrieval deadline is 25 seconds. A rejection
returns a redacted diagnostic and no context. On any hook failure, apply no memory.
