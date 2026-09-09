# PR repair record template

Fill this skeleton from the complete journal in the pull request's language. Use one short bullet
per corrected problem, describing the final behavior. Keep detailed mechanisms, per-pass proofs and
superseded attempts in the local journal. Pass the public summary through a body file.

## Completion summary

```text
<!-- pr-fix:<pr> -->
Corrections completed on `<final-head-sha>`.

- <problem corrected and resulting behavior>.

Validation: <environment, tier, principal checks and counts>; required CI: <linked result on this SHA, or none required>; independent review: <no remaining actionable defect>.
Limits: <material evidence gaps or deliberately excluded findings and reasons; omit this line when none>.
```

## Stop notice

Publish immediately when the run stops for concurrent activity or merge, even when no correction
was pushed. Approval and CI requirements apply to the completion summary only. Keep any prior
completion summary intact; use this separate marker for the notice and its retries.

```text
<!-- pr-fix-stop:<pr>:<initial-head-sha> -->
Repair stopped: <developer push, new or edited comment, merge, or unavailable activity check>.
Evidence: <event or comment URL, actor and time; expected and observed SHA/state, or exact check failure>.
Interrupted: <stage>; no further corrections or verdict will be published by this run.
Already pushed: <commit SHAs and corrections, or none>.
Preserved locally: <commits not yet pushed and uncommitted changes, or none>.
Validation: <completed checks and results>. Limits: <interrupted checks and evidence gaps>.
Restart requires a new user request.
```

Report local paths privately to the user. Never present an interrupted review as complete.
If publication fails, retain the body and report the failure; do not claim the notice was published.

## Completion summary self-check

- Publish only after independent `approved` and successful required remote CI on the current SHA.
- Stable PR marker first; update the existing repair comment even when the SHA changes.
- One opening sentence, correction bullets, one validation paragraph; aim for 15 non-empty lines.
- Cover all final corrected problems across passes; group related outcomes instead of truncating.
- State material limits and reasoned omissions directly after validation; retain full proof in the
  journal and never copy old results as evidence for the final SHA.
- Body passed through a file; no local paths, formal verdict, approval action or merge decision.
