# Local PR measurement

At the end of phase 5, invoke `arnes measure pr-verdict` once from the structured review summary.
This is a local measurement, independent of publication consent and the forge's comment marker.
It also runs before returning the pre-repair verdict to `pr-fix`. Do not parse the final Markdown,
transcript, a hook payload or a published comment to recover these values.

Build the summary from the review's working inventories:

| Argument                  | Source                                                                                                                                                          |
| ------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `--forge`                 | Lowercase hostname of the anchored PR URL, such as `github.com` or `bitbucket.org`, without scheme, credentials or path                                         |
| `--repository`            | Canonical repository path from the anchored PR's forge metadata, without hostname or `.git`; the repository containing the PR, including for fork contributions |
| `--pr-id`                 | Positive numeric identifier resolved in phase 1                                                                                                                 |
| `--head-sha`              | Full lowercase SHA actually reviewed; never re-resolve it from the current branch at emission time                                                              |
| `--agent`                 | `codex`, `claude-code` or `cursor` when known; omit when unavailable                                                                                            |
| `--verdict`               | `changes-required`, `approved-with-reservations` or `approved` selected in phase 5                                                                              |
| `--blocking-findings`     | Number of distinct retained blocking findings, including findings about missing evidence                                                                        |
| `--non-blocking-findings` | Number of retained reservations or non-blocking findings, counting each once                                                                                    |
| `--observable-behaviors`  | Number of rows in the changed-behavior ledger                                                                                                                   |
| `--evidence-gaps`         | Number of distinct missing-evidence items inventoried in phases 2–4, across both ledger and barrier; an item repeated in several places counts once             |

For evidence gaps, missing positive evidence and a missing negative witness are separate items;
an uncovered platform or integration boundary is another item when explicitly inventoried. Keep
these items in the working inventory before composing the verdict; a blocker and a gap may describe
the same missing proof but are separate counts. Unknown counts are not zero: complete the inventory
or report measurement unavailable, without inventing values or changing the verdict.

This synthetic example represents two blocking findings, one reservation, three behavior rows and
two distinct gaps; replace every value from the actual review before invocation:

```sh
arnes measure pr-verdict \
  --forge github.com --repository example/project --pr-id 1042 \
  --head-sha aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa --agent codex \
  --verdict changes-required --blocking-findings 2 --non-blocking-findings 1 \
  --observable-behaviors 3 --evidence-gaps 2
```

Arnes adds the schema version, event type, UTC Unix timestamp in milliseconds, operating system and
architecture. It does not infer the verdict from the counts, check the remote PR, inspect the diff
or establish causal links. The phase-1 anchor remains the evidence for the declared PR and SHA.
No network service, session identifier, personal hostname, prompt or verdict text is stored.
Measurement v2 has no retained verdict artifact, so this iteration adds no artifact reference.

The command uses the existing measurement store:
`$XDG_STATE_HOME/dotfiles/agent-harness/pull-requests/<identity-hash>/events.jsonl`, with
`~/.local/state` as the default state base. The hash is SHA-256 of the compact JSON PR identity
object in `forge`, `repository`, `pr_id` order; events retain that identity in readable form.
The timeline is shared across agents and sessions, regardless of their Git worktree. It has schema version
1, independently of hook run versions, and does not require a run to exist. Existing runs and their
60-day v2 retention are unchanged. A whole PR timeline expires 90 days after its latest event
timestamp; a new event preserves the whole history until that inactivity interval elapses again.
Hooks and successful PR emissions invoke the same opportunistic maintenance, at most once per day.
Without another invocation, expiry is not an immediate scheduled deletion. Corrupt, empty, unsafe
or future-dated timelines are preserved with an observable maintenance error.

Lifecycle locks live outside deleted timelines in `pr-locks/<hash-prefix>.lock`, using 256 shared
slots. Writers take the lock before creating a PR directory; maintenance rechecks expiration under
that same lock immediately before deletion. `retention.json` version 2 reports `candidate_prs` and
`removed_prs` alongside the existing run counts and maintenance status; version 1 remains readable.

Each line is one factual event: `schema_version`, `event_type`, `timestamp_ms`, `pr`, `head_sha`,
nullable `agent`, `operating_system`, `architecture`, and `data` containing the verdict and four
counts. The writer validates existing records and holds its lifecycle lock across duplicate detection
and append. It preserves existing bytes and refuses unsupported schema versions or corrupt history.
A partial write can leave a visibly invalid tail: no automatic repair or deletion is attempted.

Handle the command result before returning the verdict:

- Exit 0, `recorded`: report `Measurement: recorded` separately from the verdict.
- Exit 0, `duplicate`: the same `{pr, head_sha, event_type}` and verdict data already exist;
  report `Measurement: duplicate (existing event found)`. Agent and timestamp differences do
  not create another observation or refresh its age. Routine maintenance may expire that event.
- Nonzero exit: report `Measurement: failed` with the diagnostic, separately from the unchanged
  verdict. A different verdict or count for an existing key is a conflict; keep the first event
  and surface the conflict. There is no replacement flag in this iteration. A maintenance error
  after emission explicitly says whether the PR event was recorded or found as a duplicate;
  it does not undo the write or change the verdict.
- Missing Arnes or unsupported subcommand: report `Measurement: unavailable` with the diagnostic
  and return the unchanged verdict; do not install tools or fall back to a parallel store.

Keep a failed emission's structured summary in the current task for an explicit retry after the
storage problem is resolved. Do not suppress stderr, translate telemetry errors into review findings,
or let a failed command prevent the review's return. A retry after a successful write is idempotent
while its event remains stored; purged history no longer provides duplicate detection.

## Next iteration checkpoint

The next iteration can add a new event type and its producer to this same PR envelope and store,
with its own behavioral tests and an explicit decision on same-SHA revisions when needed. Deferred:
raw prompts, intention summaries, `pr-feedback` ingestion, `pr-fix` history, generic corrections,
pre-PR association through a Git worktree, causal analysis, derived metrics, dashboards and reports.
Future causal links must distinguish at least `inferred` and `confirmed`; none exist here.
`pr-feedback` remains read-only, `pr-verdict` remains independent with no auto-fix, and
`harness-reflection` remains unchanged for later cross-measurement analysis.
