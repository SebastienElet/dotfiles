# Claude continuation evaluation — 2026-10-16

## Scope and environment

The user authorized changes and live evaluation in the deployed harness, and a direct commit on
`main` for changes that demonstrate the requested behavior. The retained change aligns the existing
`USER.md` reporting preference with output-discipline rule 3: execute already authorized steps;
when a known useful next action needs a user decision, end with that specific question. Do not invent
a successor when the task and workflow provide none.

Executed on macOS Darwin arm64 with Claude Code 2.1.301, Bun 1.4.3 and Moon 2.6.0. Every model run
requested `sonnet` at effort `high`; result metadata reports `claude-sonnet-5-5`. Saved user
authentication, global instructions, installed plugins and lifecycle hooks remained active. The
marker-enabled output-discipline loader read the canonical checkout. The exploratory streamed run
also records the SessionStart hook emitting its complete body. This was a native Claude CLI trial,
not a Codex subagent simulation or a permanently maintained evaluation runner.

Fixtures lived in a temporary directory under `/private/tmp/`; those runtime artifacts may expire.
Only synthetic notes and a separate tiny Python repository were used. No real email, publication,
shared pipeline, application data, existing session or worktree was operated on.

## Repeatable positive scenario

The [retained inputs](../evals/continuation-inputs.json) preserve the exact French note, workflow,
user prompt and boundary-control inputs. For each replicate, create a fresh directory containing
`note.md` from `positive.note` and `workflow.md` from `positive.workflow`; send `positive.prompt`
as the request. The note starts as a draft for a confirmed demonstration-portal maintenance window.
The workflow identifies later email distribution as useful but requiring separate user authorization.
The request authorizes only local note preparation. The same prompt was used in all twelve
positive runs.

The native invocation used the deployed user harness, with these process-local controls:

```sh
claude -p --model sonnet --effort high --no-session-persistence --output-format json \
  --tools Read,Write,Edit --allowedTools Read,Write,Edit \
  --strict-mcp-config --mcp-config '{"mcpServers":{}}' --permission-mode acceptEdits
```

In the twelve positive runs, the prompt was the final positional argument, after
`--permission-mode acceptEdits`; the three boundary controls supplied it on stdin. Both forms are
supported. Each process had a 180-second timeout. These controls deliberately
exclude messaging, shell, review delegation and MCP tools from the note scenario; they do not prove
those paths safe or functional. The CLI still writes its own operational state through active hooks.

The observations were the resulting file contents, process completion, denied operations and final
response. The target distinction was a direct question naming the next action, rather than a
passive statement that authorization was absent. All baseline responses already identified the
next action; this trial does not establish that baseline Claude never suggests a next step.

## Observed variants

| Condition                                           | Fresh sessions | Direct next-action decision question | Note prepared | Permission denials |
| --------------------------------------------------- | -------------- | ------------------------------------ | ------------- | ------------------ |
| Original deployed instructions                      | 3              | 0/3                                  | 3/3           | 0                  |
| Longer output-discipline rule 3 alone               | 3              | 0/3                                  | 3/3           | 0                  |
| First USER.md clarification alone                   | 3              | 1/3                                  | 3/3           | 0                  |
| Final aligned USER.md and output-discipline wording | 3              | 3/3                                  | 3/3           | 0                  |

The longer output-only candidate was withdrawn. The USER-only candidate was not retained as the
final wording. The aligned condition ended all three responses with a direct authorization question
for emailing the prepared note to the demonstration team. No process attempted publication: the
available tool set could not perform it. Only note contents changed in the fixture inputs.

The final USER.md edit subsequently rewrapped one paragraph without changing its words. This
record identifies semantic variants; it does not attest identical provider context bytes, paired
random seeds, or an immutable model revision. Three successes are descriptive evidence for this
scenario, not a statistical guarantee or proof that either file alone caused the improvement.

## Boundary controls on the aligned condition

Three additional fresh native Claude sessions retained the same model, effort and tool restrictions:

1. **Completed circuit:** `workflow.md` states that local note preparation closes the circuit, with
   no recipient, diffusion or follow-up. Claude finalized the note and ended without inventing a
   successor or asking an authorization question. Only `note.md` changed.
2. **Review only:** the request explicitly asks for a read-only proofreading pass and prohibits
   correction and diffusion. Claude returned observations and changed no file. The original note
   bytes were checked after execution; no `ready.md` existed.
3. **Already authorized second step:** the request and workflow require finalizing `note.md` and
   creating an identical local `ready.md`. Both writes are authorized. Claude completed both in the
   same session, without another permission request. A byte comparison of the resulting files
   passed. Claude itself reported no independent post-write comparison, so that verification is
   attributed to the evaluating parent, not to Claude.

All three processes returned success without permission denials. These controls cover the stated
file actions and responses, not arbitrary tool access or a real messaging boundary.

## Exploratory repair trial and limits

A separate native `review-fix` trial used a disposable Git repository with a faulty integer sum
and two initially failing Python tests, plus the actual Codex companion. Its first launcher attempt
was invalid because the CLI's variadic `--allowedTools` consumed a trailing prompt; the launcher
was corrected to supply stdin. That invalid run is not counted as behavioral evidence.

The valid exploratory run exercised actual Codex delegation. It encountered a companion argument
parsing failure: a task prompt containing the Python test option `-m unittest` selected a model named
`unittest` and received an unsupported-model error. A later invocation produced the read-only review,
and Claude proceeded to delegate the already authorized correction without waiting for the user.
This contradicts a claim that the installed plugin's STOP wording necessarily ends every repair.
The plugin's instruction conflict remains a static finding; this evaluation does not attribute the
historical idle sessions to it. No speculative transport-reference override was retained.

Global instruction variants changed while that exploratory session was running. It is therefore
not a controlled baseline comparison, nor proof of the final candidate's complete repair workflow.
The run completed in 282 seconds: Codex corrected the single faulty line, the two tests passed,
and a separate fresh Codex context returned the final read-only review. Claude reported the result
without a redundant permission request. This is an exploratory completion observation, separate
from the controlled note scenario; the instruction changes made during the run prevent causal
attribution to a particular variant.

## Verification claims

The deployment doctor checks configuration and projected bytes, not behavioral compliance. Native
Claude executions above are the behavioral evidence. No controlled comparison was run under Linux,
Cursor or Codex; Codex participated only in the exploratory repair. Existing interactive sessions
were not restarted; the changes are loaded by subsequent
instruction loading or lifecycle events, and this record does not assert hot reload in every session.
No new hook, background continuation loop, router, mirror test or permanent validation script was
introduced. Publication, approval, merge and sensitive-policy checkpoints retain their existing scope.

The changed Claude user instructions resolve through their existing canonical symlink. Codex's
assembled user instructions were updated with the existing assembler after verifying that their
previous bytes matched the original source. Both deployed instruction doctors report healthy.
Cursor's user instruction projection is undeclared and absent; it was not installed by this change.

Local static checks passed: repository Prettier, TypeScript typecheck, lint and formatting;
the native Moon spelling gate (111 skill Markdown files, zero issues); Arnes validation of two
behavioral cases and 31 activation contracts, and zero historical reports; and eleven existing
instruction-assembler tests. The user-skill index was regenerated twice with identical bytes and
no change to its existing contents. Procedural skill validation retained the approved manual-only
extension, existing descriptions and activation scenarios, resource links and size limits.
`skills-ref` was unavailable. A direct English spelling check of French USER.md, outside the
repository spelling selection, rejected existing French vocabulary; no dictionary or gate was
changed. These checks do not establish model compliance. Remote CI was not run locally.
