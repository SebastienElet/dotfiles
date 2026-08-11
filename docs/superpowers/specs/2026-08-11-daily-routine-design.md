# Daily Routine — Design

**Date**: 2026-08-11 · **Status**: approved

## Purpose

`daily-routine` builds a scoped daily development status from authenticated local CLIs, prints it,
and creates the missing Today to-dos in Things 3. The configuration is the sole scope boundary: no
repository outside it is queried.

The delivered command is non-interactive. The earlier do-nothing-script wording describes the
operator workflow represented by the Things to-dos; the clarified command itself does not prompt,
wait for Enter, or launch an agent. `--no-things` prints the same report and exits without reading
or writing Things. `--limit <count>` bounds each section to its oldest `count` items, so a
long-standing backlog is delivered one batch per run instead of as one unworkable list.

## Considered approaches

1. Put the complete program in `src/main.rs`. This minimizes the file count but mixes CLI parsing,
   provider schemas, correlation, reporting, and macOS integration in a file that would be hard to
   learn from or maintain.
2. Use small concrete modules for configuration, commands, providers, domain rules, reporting, and
   Things. This keeps the Rust idiomatic and readable without hiding the workflow. **Selected.**
3. Build provider traits and injectable command-runner abstractions. This maximizes unit-test
   isolation but adds type machinery that is not justified for a personal CLI with three fixed
   providers.

## Structure

- `main.rs` parses `--self-check`, `--no-things`, and `--limit <count>`, loads the configuration,
  and orchestrates the ordered steps.
- `config.rs` owns the Serde TOML model, defaults `requires_linear` to `true`, validates repository
  paths, and derives the ordered unique repository set and configured team keys.
- `command.rs` runs child processes without a shell and reports exit status, stderr, and malformed
  JSON with command context.
- `bitbucket.rs`, `github.rs`, and `linear.rs` contain only concrete CLI invocations and their
  deserialization structs.
- `model.rs` contains normalized pull requests, Linear issues, identities, tracks, categories, and
  report items.
- `rules.rs` correlates pull requests and issues, attaches items to tracks, evaluates discrepancies,
  consolidates duplicate findings, and sorts the report.
- `things.rs` reads Today and adds missing titles through the URL scheme.
- `self_check.rs` contains in-process fixtures and assertions; it invokes no external command.

Only `serde` with `derive`, `serde_json`, and `toml` are dependencies. Process execution, argument
parsing, scoped concurrency, civil-date conversion, percent encoding, and Linear identifier
matching use the standard library.

## Pipeline and scope

1. Parse the configuration from `$HOME/.config/daily-routine/config.toml`. A missing `HOME`, missing
   file, unreadable file, invalid TOML, or invalid repository path prints the diagnostic followed by
   the complete compile-time `config.example.toml` to stderr and exits 1.
2. Resolve Bitbucket, GitHub, GitHub-team, and Linear viewer identities once. Identity failures are
   recorded against the categories that need them rather than aborting unrelated sources.
3. Deduplicate repositories by `(provider, owner/name)` before any provider call. Provider list
   calls are sequential; only per-pull-request detail calls are concurrent.
4. Collect pull requests and assigned Linear issues, normalize them, correlate known team
   identifiers, then evaluate the report rules.
5. When `--limit <count>` is set, keep the oldest `count` items of each category and warn how many
   each truncated category withheld, so a shortened report never reads as an emptied backlog. The
   limit bounds the report itself rather than only the Things writes, so both tell the same story.
6. Print categories in `REVIEW`, `RETOUR`, `LINEAR`, `SUIVANT` order. Each line identifies its track,
   stable Things title, reason when applicable, and URL.
7. Unless `--no-things` is set, read Today before the first write, then add only titles absent from
   that snapshot or earlier successful additions in the same run.

Repository attachment first considers all tracks that declare the repository. A correlated Linear
issue whose team matches one of those tracks selects that track; otherwise the first declaration in
configuration order wins. A repository remains attachable without a ticket. Ticket-only items use
the first configured track declaring their team. Tracks with no teams produce no `LINEAR` or
`SUIVANT` items.

## Provider collection

### Bitbucket

For every configured Bitbucket repository, use its short name with `bkt pr list` to collect open
authored pull requests, open review requests, and the recent merged authored pull requests needed
for ticket-completion checks. Review candidates in draft are discarded; `bkt pr view` confirms the
current viewer is a reviewer whose `participants[].approved` is false.

For each open authored pull request, `bkt pr comments --state unresolved` and
`bkt pr task list` collect feedback. Deleted comments and comments authored with the viewer's
`account_id` are ignored; any open Bitbucket task qualifies. Detail work is distributed across at
most eight scoped worker threads, with one pull request processed by one worker.

### GitHub

For every configured GitHub repository, collect open authored pull requests and the recent merged
authored pull requests needed for ticket-completion checks. Review candidates are the deduplicated
union of `review-requested:@me` and one `team-review-requested:<org>/<slug>` search for each viewer
team belonging to the repository owner's organization. Draft review candidates are discarded.

Each open authored pull request gets one GraphQL detail call for `reviewDecision` and the first
comment of up to 50 review threads. An unresolved thread qualifies only when its first comment is
not from the viewer; `CHANGES_REQUESTED` independently qualifies the pull request. These calls use
the same bounded scoped-worker implementation as Bitbucket.

### Linear

One raw GraphQL query resolves the viewer and up to 200 assigned issues, including state, team,
project, labels, priority, update time, and branch name. Only issues whose team key appears in the
configured tracks enter correlation or reporting.

A second query resolves blocking relations, because merging it into the first exceeds Linear's query
complexity ceiling of 10000. It reads `inverseRelations` — the relations stored on the blocked side,
where a `blocks` entry names an issue blocking this one — for the first 100 unresolved assigned
issues, 20 relations each. The API exposes no filter on that connection, so `blocks` is selected
client-side, and a `completed` or `canceled` blocker is discarded because a resolved issue blocks
nothing. Both page sizes are bounded by that same ceiling, so a saturated page is reported as a
warning rather than silently under-reported. This query is skipped when no issue is in scope, and a
failure degrades with a warning instead of discarding issues the report still renders correctly:
losing precision costs less than losing `LINEAR` and `SUIVANT` entirely.

## Correlation and report rules

Correlation scans the pull-request title, branch, then description for `<configured-team>-<digits>`
with ASCII word boundaries. The first match in that field order wins. Matching is case-insensitive,
while normalized identifiers use the configured uppercase team key. No regular-expression crate is
needed.

`REVIEW` contains each non-draft open review request that still awaits the viewer. Bitbucket uses
participant approval; GitHub search results already encode the outstanding request. The oldest
available event is the pull-request creation time because neither provider exposes review-request
time in the selected commands.

`RETOUR` contains each open authored pull request with at least one qualifying unresolved external
thread, an open Bitbucket task, or GitHub `CHANGES_REQUESTED`. Multiple signals become one item; its
event time is the oldest qualifying comment or task time, falling back to the pull-request update
time when only `CHANGES_REQUESTED` supplies the signal.

`LINEAR` implements all six discrepancies:

1. a merged correlated pull request whose issue is not `completed`;
2. an open correlated pull request whose issue is `backlog` or `triage`;
3. a `started` issue with neither a non-empty Linear `branchName` nor any correlated scoped pull
   request;
4. a `started` issue whose `updatedAt` age is greater than `stale_days`;
5. an assigned issue missing a project, any label, or a non-zero priority;
6. an open pull request with no configured Linear identifier when its selected repository
   declaration has `requires_linear = true`.

`renovate/*` branches are exempt only from rule 6. Draft authored pull requests remain subject to
all `LINEAR` rules. Destination branches do not affect any rule. Multiple discrepancies for the same
ticket or pull request are consolidated into one Things item with multiple reasons in terminal
output.

An issue held by an unresolved blocker leaves `LINEAR` entirely, whichever rule it triggers,
including rule 1 on a merged pull request: it cannot be acted on today, so its metadata is repaired
when it becomes actionable. The exclusion applies to issues only; an uncorrelated pull request under
rule 6 has no issue to be blocked by.

`SUIVANT` considers `triage`, `backlog`, and `unstarted` issues that no unresolved blocker holds.
Per track, it selects at most `next_count` by Linear priority (`1` through `4`, then `0`) and oldest
`updatedAt` as a tie-breaker, so a blocked candidate frees its slot for the next actionable one. The
selected items are then merged into the category's chronological output order.

Within every category, report items use their rule-specific event time and sort oldest first, with
track configuration order and stable reference as deterministic tie-breakers.

## Things titles and writes

Titles are `[CATEGORY] <reference> <title>`, where a pull request uses `#<number>` and an issue uses
its Linear identifier. The complete title is truncated to 120 Unicode scalar values so the prefix
and reference remain stable and valid UTF-8. Notes contain only the pull-request or issue URL.

The Today AppleScript sets its text delimiter to linefeed, so titles containing commas remain
intact. Exact title equality is the deduplication key. Percent encoding works byte-by-byte on UTF-8:
only `A-Z`, `a-z`, `0-9`, `-`, `_`, `.`, and `~` remain literal. `open` receives the complete Things
URL as one process argument, never through a shell.

## Failure behavior

Configuration errors are fatal because the scope would otherwise be unknown. All later failures
are non-fatal and explicit on stderr. A missing CLI, inaccessible repository, failed subprocess,
non-JSON output, or incomplete provider response removes only the data dependent on that call and
marks the affected category as partial in terminal output. Other repositories, providers, and
categories continue.

Things read failure prevents all writes because safe deduplication is impossible. An individual
Things write failure is reported and later items continue. Empty result sets are valid and distinct
from collection failures.

## Verification

`--self-check` runs assertions over hard-coded fixtures without reading configuration or invoking a
CLI. It covers title/branch/absent correlation, the six `LINEAR` rules, mono-track and shared-repo
attachment, ticketless attachment, `requires_linear = false`, the blocked-issue exclusion, the
`--limit` truncation and its warning, report ordering, configuration parsing/defaults,
`days_from_civil`, and percent encoding of an accented title.

Development follows red-green-refactor around the pure rules before provider orchestration. Final
verification is `cargo fmt --check`, `cargo clippy -- -D warnings`, `daily-routine --self-check`, a
release build, `make -n daily-routine`, and one real `daily-routine --no-things` run. The real output
is shown to the user before any Things write is attempted.

## Repository integration

The versioned binary crate includes `Cargo.lock`; `.gitignore` ignores only
`daily-routine/target/`. A `rust` Make target installs Homebrew's `rust` formula through a
`${BREW_BIN}/cargo` sentinel, which follows ADR-002 and replaces the removed pre-ADR rustup recipe.

The `daily-routine` target depends transitively on Rust, builds with `--release`, and symlinks
`${LOCAL_BIN}/daily-routine` to the checkout's release binary. It is attached to the existing
semantic aggregate used by `all`, rather than added as an ad-hoc direct dependency. Because CI runs
`make all`, its recipe must build without configuration files, service authentication, or runtime
provider calls.
