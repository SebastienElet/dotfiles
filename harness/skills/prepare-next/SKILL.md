---
name: prepare-next
description: >
  Select and prepare the next available project ticket before development. Use when the user asks
  to choose the next ticket and settle its functional or technical decisions before coding. Make
  sure to use this skill whenever next-work selection includes development preparation, even if
  the user does not name the skill. Excludes implementation, backlog reports and status lookup.
compatibility: Requires authenticated tracker read access and access to the project's repository.
metadata:
  category: dev
---

# Prepare Next

## Overview

Choose one available ticket from current project evidence and resolve the material decisions needed
before development. Reuse `grilling` for the interview and return a concise preparation handoff.
Preparation can reduce foreseeable interruptions; newly discovered facts may still require a decision
during implementation.

## Usage

`$prepare-next <project or repository> [ticket ID or list]`

Example: "Prepare the next available ticket in the invoice project before coding."
An explicit ticket or list bounds the candidates; otherwise select within the verified project.
Infer the project only from unambiguous repository and tracker evidence. Preparation authorizes reads
and an interview, with the default deliverable in the conversation. It does not start development,
launch agents, change tracker fields or apply a readiness label.

## Steps

1. **Resolve the project and supported access.** Read repository instructions and Git remotes;
   verify the tracker project and repository mapping through an authenticated connector or established
   CLI. Inspect its current capabilities and pagination before querying. Ask only when the target
   remains materially ambiguous. If access or required fields are unavailable, name the missing
   evidence and stop the affected selection rather than reconstructing ticket state from prose.

2. **Find available candidates.** Read current identity, project membership, work states, ownership,
   blocking relations, hierarchy, priority, deadlines and existing work links as supported by the
   tracker. Exhaust the relevant pages before claiming a project-wide selection. Retain open,
   unblocked, available tickets consistent with the project's assignment and start policy. Exclude
   completed, canceled or already active work and parents with any open sub-issue, including blocked
   children. For
   Linear/Bitbucket, compose `linear-workflow` for shared policy and its read-only transports; only
   issues assigned to the current user are eligible. Use structured fields for eligibility. A missing
   field is not evidence that a blocker or owner is absent. Dependencies that are themselves eligible
   are candidates. Keep excluded and blocked-ticket inventories out of the response; if none qualify,
   say that no available ticket was found.

3. **Choose with evidence.** Follow the project's explicit work ordering or prioritization policy
   first. Otherwise compare confirmed urgent incidents and deadlines, declared priority, and the
   eligible ticket's ability to unlock other work. Respect the tracker's priority semantics; an unset
   value does not imply urgency. Use unlocking effect as a tie-breaker rather than inventing business
   impact or a weighted score. When missing priority evidence or conflicting criteria leave a material
   choice unresolved, present only the relevant available candidates and ask for the smallest
   arbitration. Return one selected ticket and a short sourced reason; do not print the backlog.

4. **Inspect the selected work.** Read the full current ticket, linked decisions and relevant code
   flow, consumers, tests, designs and ADRs in force. Verify the ticket's claims against those sources.
   Keep established facts, working assumptions and unresolved choices distinct. Resolve discoverable
   facts yourself, including dependency behavior from official documentation when consequential.
   Identify material functional and technical decisions, known manual prerequisites, and the existing
   checks relevant to acceptance. Leave local reversible implementation choices to the future agent.

5. **Resolve material decisions with `grilling`.** When user judgment is needed, delegate the
   interview to `grilling` with the selected scope, established constraints, verified sources and
   unresolved choices. Preparation is the authorized caller; the delegated scope is interviewing and
   source research only. Reuse its dependent-decision procedure rather than maintaining a second
   questionnaire. Skip the interview when no material decision remains. Continue independent research
   while waiting, but never adopt an unanswered recommendation as a decision. A contradiction with an
   ADR in force or unavailable evidence blocks the affected preparation; report that specific blocker
   without adding a list of blocked backlog tickets.

6. **Consolidate the result.** Preserve the intended outcome and propose only the ticket amendments
   needed to record settled decisions, constraints and observable acceptance criteria. For a Linear
   product ticket requiring functional shaping, compose `linear-issue-spec` for the substantive draft;
   `issue-creation` owns any requested issue publication or editing authority. Do not invoke `to-spec`
   or `grill-me` implicitly; their direct invocation contracts remain separate. Keep implementation
   architecture, files, libraries and detailed test plans out of a proposed product specification.
   Scale the handoff to the ticket instead of expanding every preparation into a full specification.

7. **Recheck and hand off.** Read the selected ticket's eligibility and work links again before
   calling it ready to launch. If another worker has taken it or a blocking relation has appeared,
   report that the selection changed and reassess the remaining available candidates; retained
   answers apply only when their scope still matches. Return the ticket reference, selection reason,
   observable outcome, settled decisions and sources, known prerequisites, relevant existing checks,
   and either "ready to launch" or the specific unresolved preparation blocker. Identify the source
   snapshot by ticket revision or observation time and repository HEAD. This is a preparation
   assessment, not a reservation or proof of implementation. The future implementation agent must
   rediscover the current code and recheck eligibility.

## Gotchas

- **Ranking before filtering availability** — an urgent blocked or occupied ticket becomes the next
  assignment. Filter from structured current state first and present only available candidates.
- **Reading only the first tracker page** — a lower-priority ticket can appear to be the best choice.
  Complete the bounded project query or report that a project-wide choice cannot be established.
- **Asking about facts or local technical preferences** — preparation creates the same interruptions
  it should remove. Research facts and reserve the interview for consequential choices.
- **Treating preparation as a launch instruction** — worktrees, agents or lifecycle writes start
  prematurely. Return the handoff and leave execution to an authorized start workflow.
- **Reusing a stale readiness assessment** — ownership or dependencies may change during the
  interview. Recheck the selected ticket and require a fresh check at implementation startup.

## Constraints

- Select only within the verified project or explicitly bounded ticket input and established ownership
  policy; never invent urgency, priority, business impact or eligibility.
- Never present blocked-ticket inventories or replace a missing structured eligibility field with
  a conclusion from description text.
- Never finalize unresolved material choices as assumptions or override an ADR through the interview.
- Never implement, provision worktrees, launch agents, mutate tracker state or publish preparation
  from this invocation alone; saving or external writing requires an explicit requested destination
  and the applicable workflow's authority.
- Reuse `grilling` for material decision interviews and specialist skills for their own responsibilities.
- Never promise interruption-free development or treat preparation readiness as an execution guarantee.
