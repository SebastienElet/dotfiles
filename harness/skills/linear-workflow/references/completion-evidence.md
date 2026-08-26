# Completion Evidence in Linear Issues

Read this before checking, unchecking, or restructuring any checkbox or evidence section in a Linear
issue, and before completing an issue from merged work. Transport selection stays in
[transport adapters](transports.md).

## What a checked box asserts

A checked box is an assertion of proof, not a progress marker. It claims that a named oracle
produced the result written on that line.

Checking is authorized by:

- a test, command, or check whose run you can name, together with the environment it ran in and its
  green result;
- a CI run you can point to that actually covers the line;
- an observation you made yourself and can reproduce: a measurement taken, a query executed, a
  screen seen.

Checking is not authorized by:

- the issue being closed, its pull request merged, or its cycle ending;
- the neighboring boxes being checked and this one spoiling an otherwise clean list;
- the code appearing to do it, or a reviewer having approved it;
- a check that exists but does not exercise the line — a green suite that never runs the surface you
  touched proves nothing about it;
- prose asserting it, in the description, a comment, or a pull request.

A line that demands a measurement nobody made stays unchecked, however small the remaining gap
looks. Report it as unproven instead.

## Workflow state and box state are independent

They are two axes, and converging them destroys the only record of what is still unproven.

1. Never check a box because the issue is being completed, and never keep an issue open only because
   a box is unchecked.
2. `Done` with unchecked boxes is a legitimate outcome, but it is an explicit human decision to
   accept unproven residue. An agent routing lifecycle state never takes it alone.
3. When a transition leaves boxes unchecked, record the residue explicitly in the same exchange:
   name each unchecked line and why it is unproven — measurement never taken, environment out of
   scope, deliberately accepted. Put it in the response, and in an issue comment when the transport
   supports one, so the gap survives the issue being closed.
4. Never uncheck a box that evidence still supports in order to justify reopening or delaying an
   issue, and never uncheck one merely because its proof is not in front of you: absence of evidence
   in this session is not evidence that the run never happened. Re-audit a checked box only when
   current evidence disproves it, or when the user asks for a fresh audit and its proof cannot be
   recovered.
5. Box state may gate a transition, and never the reverse: refusing `Done` over an unchecked line is
   sound, checking a line because a transition is wanted is fabrication.
6. Never hide residue by deleting an item, softening its wording, moving it out of the task list, or
   replacing the section with a completion summary. The unchecked line is the record.

## Merged work whose verification boxes are unchecked

A merge proves the code landed. It proves nothing about a line the merge did not run.

1. Identify the issue's verification boxes: the checkboxes carrying acceptance criteria or
   validation scenarios. Sub-task and implementation to-do boxes are not verification boxes. When a
   box's nature is genuinely ambiguous, list it and ask rather than deciding the lifecycle on a
   guess.
2. Every verification box checked completes the issue: the merge is then the last missing fact.
3. At least one unchecked box sends the issue to its team's review state instead of `Done`, with the
   unchecked lines named. The work is not undone; it is unverified.
4. No verification box at all leaves nothing to gate the transition, so the merge remains the
   completion oracle. An evidence section written as plain bullets gates nothing either: report it
   as untracked instead of treating it as a gate.
5. Resolve the review state inside the issue's own team — the `started` workflow state reserved for
   review, observed as `In Review` across this workspace's teams. Never carry one team's name over
   to another, and stop rather than inventing a state when a team exposes none.

## Verifying issues that wait in the review state

This is the pass that turns unverified merged work into a defensible `Done`, and it is the only one
allowed to produce the missing evidence.

1. Select the assigned issues in the review state whose pull request is merged. That set is the
   queue. An issue whose pull request is still open belongs to code review, not here.
2. For each issue, take its verification boxes one by one and produce the missing oracle: run the
   check, point at the CI run that actually covers the line, take the measurement.
3. Check only what the run you just named proves, in one anchored edit per issue.
4. Move an issue to `Done` once every verification box holds.
5. Leave an issue in the review state when a box stays unproven, and report what is missing: the
   oracle nobody has run, and what running it would take. Do not check it, and do not close around
   it without an explicit decision.
6. Report the queue as a whole — issues completed, issues still waiting, and the unproven line
   behind each one. A batch that silently drops an issue reads as a verified batch.

## Evidence sections that are not checkboxes

A bulleted evidence section holds nothing: it does not distinguish a proven line from an asserted
one, so a reader assumes the whole section is proven. Treat it as untracked prose.

1. Give the per-line verdict in your response rather than relying on the issue text to carry it.
2. When the section is meant to gate completion and the request authorizes editing the issue,
   convert its lines to checkbox items, unchecked, in one anchored edit. Preserve the wording: this
   is a structural change, not a rewrite.
3. Then check only the lines the evidence rule above authorizes. Converting a section never
   authorizes checking a line the conversion itself did not prove.
4. When editing is not authorized, say that the section tracks nothing and report the verdicts
   instead of asking anyone to trust it.

## Editing boxes without breaking the issue

Writing a box requires either an explicit request to reconcile the description or the verification
pass above. Reconciling lifecycle state on its own authorizes inspecting and reporting, never
writing.

1. Read the current description first. The stored Markdown is not what the Linear UI renders: issue
   mentions are serialized as `<issue id="…" href="…">ENG-482</issue>`, and retyping a description
   by hand mangles them.
2. Prefer the transport's anchored partial edit over rewriting the description. On the Linear
   connector this is `save_issue` with `id` and `patch`, used in place of the `description`
   argument:

   ```json
   {
     "id": "ENG-482",
     "patch": [
       {
         "op": "replace",
         "old_string": "- [ ] Scenario 2: import rejects a malformed row",
         "new_string": "- [X] Scenario 2: import rejects a malformed row"
       }
     ]
   }
   ```

3. Anchor on the whole line, marker included, with enough of its text to match the current content
   exactly once. Operations apply in order and atomically, so one failing anchor aborts the entire
   save. Never set `replace_all` for a checkbox edit: two scenarios can share a prefix, and one
   anchor must identify one line.
4. Batch every box of one decision into a single call, up to the transport's operation limit, so the
   description is never left half-updated.
5. Linear normalizes a checked marker to uppercase `- [X]`. Never require lowercase `- [x]`, never
   anchor on it when re-reading a line you just checked, and never edit an issue merely to change
   that case.
6. Re-read the issue after the save and compare the marker of every line you touched. A matched
   anchor proves the operation was accepted, not that the stored description says what you intended.
7. When no anchored edit exists, build the full description write from the description you just
   read, byte for byte, changing only the markers. Never reconstruct it from the rendered view or
   from memory.
