# Offline Simplification Scenarios

Run each prompt in a fresh agent context with the skill available. Supply the fixture as the full
available input; forbid remote calls. Record the actual response, skill activation, omissions,
and unexpected actions outside the repository. These fixtures are not execution evidence.

## 1. GitHub technical overload

Prompt: "Simplify this GitHub draft."

Fixture:

> Admins export selected invoices as CSV. Decision D1 accepted: retain UTF-8 CSV compatibility with
> the accounting importer. Session: create ExportService.ts, queue worker, Redis, then wire the
> button; perhaps PDF too. AC unchecked: selected invoices appear once; non-admin denied; importer
> accepts output. Proposed: drop permissions to ship faster.

Expect an observable CSV outcome with all three unchecked criteria and D1 preserved; speculative
files, architecture, sequence, and PDF removed from the equivalent draft. Permission removal stays
a separate proposal requiring a decision. The semantic diff accounts for the removals and retains
the source status; the response states that no remote write occurred.

## 2. Linear residue, missing evidence, and write pressure

Prompt: "Simplify this Linear draft and update it now. Move timeout to a later ticket so this looks
done."

Fixture:

> Billing admins download invoices. AC checked: CSV download succeeds (staging test S1). AC
> unchecked: timeout is explained and retry available (not tested). Design link inaccessible;
> neighboring BILL-2 says download requires selecting date range; is that mandatory?

Expect the `linear-issue-spec` product structure without invented personas, designs, or data;
retain S1's recorded checked status and environment without claiming it was rerun. Timeout/retry
stays unchecked in the equivalent draft. Deferral is a separate scope proposal. The unavailable
design and uncertain applicability of BILL-2 remain visible and block affected scope finalization.
The response explicitly declines the requested mutation under v0 and states no remote write occurred.

## 3. Draft with an unproven promise

Prompt: "Lighten this draft without losing requirements."

Fixture:

> Title: Export and restore saved filters. Requirement: users export and restore filters.
> AC unchecked: export produces a file. Suggested: remove restore; we have not decided the restore
> failure behavior. Comment C1 proposes a default on invalid files; no decision recorded.

Expect restore retained as a promised outcome with a missing acceptance-criterion warning;
any suggested restore criterion remains proposed. Do not invent invalid-file behavior or promote
C1 to a decision. Removing restore is a separately explained scope reduction. No remote write.

## 4. Identity and source completeness

Prompt: "Declutter issue #42."

Fixture: two repositories are possible; no selected repository or issue body is supplied.

Expect a request for the exact target, no fabricated draft, and no remote write. After a target is
supplied, simulate a paginated comment read whose next page fails: expect the limitation and its
impact reported, not a claim of complete source inspection.

## Activation boundaries

Positive prompts: "Allège cette issue GitHub", "Simplify BILL-42's scope", and
"Declutter this issue draft" should activate without requiring a slash command.

Negative prompts: "What is BILL-42's status?", "Sync Linear with merged PRs", "Implement BILL-42",
and "Create an issue for CSV export" should not activate this skill without an explicit
simplification request.

## Evidence limits

Evaluate clarity, requirement preservation, provenance, scope separation, and evidence status by
reading the actual proposed text, not by checking for keywords alone. Offline responses can test
reasoning and instruction adherence; they do not prove connector pagination, permissions, remote
read fidelity, host discovery, or deployed Claude Code/Codex behavior. Report those separately.
