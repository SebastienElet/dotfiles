# Linear Checkbox Evidence

Load this procedure only when an issue description contains a task list or a section intended to
track completion evidence, or when the user explicitly asks to reconcile checkboxes.

Without an explicit request to reconcile checkboxes or the whole evidence section, use only the
evidence and status-independence rules to inspect and report residue. Do not enter the conversion or
safe-edit procedures.

## Evidence oracle

1. Treat each checked item as a claim that the exact statement on that line is established.
2. Check it only from item-specific, observed evidence, such as:
   - a named test that exercises the stated scenario and a green result from the relevant commit
     and environment;
   - the requested measurement and its recorded result;
   - a directly observed manual result or durable artifact that establishes the whole claim.
3. A merge, `Done` state, implemented code, test existence without an executed result, generic green
   CI that does not exercise the claim, plan, expectation, or request for a tidy issue is not enough.
4. Keep an item unchecked when proof is missing, partial, stale, from an irrelevant environment, or
   unable to establish every part of the claim. Name the missing proof rather than inferring it.
5. Preserve already checked items unless current evidence disproves them or the user explicitly asks
   for a fresh audit and their proof cannot be recovered. Do not turn absence of evidence in the
   current session into evidence of absence.

## Status independence

- Workflow state and checkbox state have separate oracles. Never check items because the issue is
  closing, and never block a proven lifecycle transition solely to make the checklist reach 100%.
- An issue may be `Done` with unchecked items. Leave those markers unchanged and list each residue,
  with its missing proof, in the reconciliation result.
- Do not hide residue by deleting an item, weakening its wording, moving it out of a task list, or
  replacing the section with a completion summary.
- Treat `- [x]` and `- [X]` as equivalent checked markers. Linear may normalize the stored form to
  uppercase; marker case is not an invariant.

## Non-checklist evidence sections

A plain bullet list has no per-item completion state even when its heading says `Completion
evidence` or similar. Never report such a section as reconciled merely because its claims appear in
the issue.

When the user authorized evidence reconciliation for the whole section, convert each plain evidence
bullet to a task-list item without changing its text, then apply the evidence oracle: checked only
when proven, unchecked otherwise. When the request covers only existing checkboxes, the bullets are
ambiguous, or preserving their meaning is uncertain, leave the section unchanged and report that it
has no checkable state.

## Safe description edits

1. Retrieve the latest issue description immediately before editing.
2. Inspect the selected Linear transport's current schema. Use `save_issue.patch` when it exposes
   targeted operations shaped like:

   ```json
   {
     "id": "ISSUE-ID",
     "patch": [
       {
         "op": "replace",
         "old_string": "- [ ] Exact claim",
         "new_string": "- [X] Exact claim"
       }
     ]
   }
   ```

3. Give every `replace` a non-empty `old_string` that occurs exactly once in the latest description.
   Include enough surrounding text to make the anchor unique; never set `replace_all` for a
   checkbox edit.
4. Patch only the intended list markers or bullet prefixes. Preserve all untouched serialized
   content byte-for-byte, especially issue mentions shaped as
   `<issue id="…" href="…">TEAM-123</issue>`.
5. If the transport lacks targeted patching, stop and report the limitation instead of manually
   reconstructing and overwriting the entire description.
6. Retrieve the issue again after the write. Verify every intended item, all remaining residue, and
   the preservation of surrounding text and serialized mentions. Treat Linear's `[X]` normalization
   as equivalent to the submitted checked marker.

## Constraints

- Never check an item without evidence that establishes that exact item.
- Never derive checkbox state from issue or pull-request state.
- Never overwrite the complete description for a marker-only edit.
- Never claim a plain bullet list has been reconciled as completion state.
