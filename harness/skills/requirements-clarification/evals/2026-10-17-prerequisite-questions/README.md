# Questions for unresolved prerequisites

## Diagnosis and scope

The reported failure is an analysis that identifies material prerequisites but returns a plan
without asking the user for the decisions needed to resolve them. The user reported three
occurrences, including the T00 analysis of [Semctx issue 365](https://github.com/hoklims/semctx/issues/365).
The issue was read on 2026-10-17; it proposes a bounded research budget and specifies corpus,
independent verification and execution requirements. It does not establish that every prerequisite
requires a new user decision. The original analysis transcript was not available in the issue,
which had no comments. Its exact instruction context and historical cause are not reconstructed.

There was already an explicit question requirement in `harness/USER.md`, reinforced by commit
`51f79cfe` and output discipline. Deployed user skills for Codex, Claude and Cursor resolved to the
canonical checkout, and the deployed clarification skill matched this worktree's baseline bytes.
A missing global rule or a stale clarification-skill projection is therefore not the demonstrated
defect in the reproduced environment.

The procedure had a weaker completion path: step 6 asked a question on a blocking condition, while
step 7 allowed delivery of a specific unresolved blocker. The analysis-only clause prohibited
implementation without explaining how to collect decisions within that scope. This is a plausible
procedural cause, not an observation of the model's internal reasoning. All six positive baseline
runs actually read the clarification skill; five nevertheless omitted the conversation question.

The previous [autonomy trial](../2026-10-15-autonomy/README.md) contains another observable omission.
Its `candidate-review-only-1` response recommends confirming the export permission matrix but asks
no question. That trial reported the absence of source modification for the read-only case; it did
not establish that clarification happened. The [continuation trial](../../../output-discipline/references/continuation-evaluation.md)
tested a later distribution authorization, not the unresolved prerequisites of an analysis.

Reflection decision: **propose**, type `sequence-recipe`, classification `harness-gap`. The user
explicitly authorized this correction and its trials. The candidate changes the existing
`requirements-clarification` procedure and description; it adds no global rule, router, hook,
validator, runner or CI gate. The trigger is a material prerequisite whose decision is reserved to
the user. The desired sequence is source lookup, delegated choice, then an actual conversation
question with a proposal and consequences. Analysis remains analysis; independent work continues.
The counterexample is a task with settled decisions and delegated technical details. A missing
question, an unnecessary decision request on that control, or unauthorized implementation falsifies
the candidate. Reassess or withdraw it if later trials contradict these results or the host makes
this guidance redundant. Behavioral support here is limited to Codex; the user's requested common
source and its other projections receive no cross-agent behavioral guarantee.

## Trial method

The 24 native executions ran on macOS Darwin arm64 with Codex CLI 0.163.0, requesting
`gpt-6.1-sol`, reasoning effort `medium`, and a 240-second timeout per execution. No execution
timed out or returned malformed output. Each used a fresh temporary home, Codex home and Git
workspace. Agent-command network access and web search were disabled. Saved authentication was
copied temporarily and removed after each run; no credentials or private conversation transcripts
are retained.

Both conditions include the same global instructions, persona, user preferences, visual preferences
and explicitly appended output-discipline body. Only `requirements-clarification` is discoverable
as a skill. Baseline sources come from `ee5e97e1`; the candidate differs only in that skill's
description and body. This measures the combined change, not a separate causal contribution for
each sentence or the description. User configuration, command rules, deployed hooks, plugins and
MCP are excluded. This is not a full installed-harness trial.

[scenarios.json](scenarios.json) preserves exact prompts, files and replicate counts.
[observations.json](observations.json) preserves source and prompt hashes, completion and usage,
user-facing messages, executed commands and statuses, changed file bytes, and manual verdicts.
Temporary paths are normalized to `$RUN`; repeated tool output is omitted. Verdicts were assigned
by reading the questions, proposed consequences, source usage and actual file changes, not by
searching for question marks or trusting the agent's success report. The runner was disposable and
is not part of the repository.

The observable question must reach the conversation. A prerequisite table, a future approval step,
or a question only inside `readiness.md` does not pass. A recommendation is not user approval.
For the analysis case, the file comparison and command trace must show no implementation or
experiment. For preparation cases, `readiness.md` must exist and the supplied sources remain intact.

## Results

| Scenario                                                         | Baseline                                                     | Candidate                                                                      |
| ---------------------------------------------------------------- | ------------------------------------------------------------ | ------------------------------------------------------------------------------ |
| Material budget and corpus decisions open                        | Questions absent, 0/3 passing; document prepared 3/3         | Questions with concrete proposals and consequences, 3/3; document prepared 3/3 |
| Facts accessible, decisions settled, technical details delegated | Document prepared without questions, 3/3                     | Same behavior, 3/3                                                             |
| Analysis only with material decisions open                       | Analysis delivered; questions in 1/3; no file changes in 3/3 | Analysis and questions in 3/3; no file changes in 3/3                          |

The positive runs did not ask the user to rediscover the documented runners or assigned verifier.
They did not launch the dependent research. All preparation runs created only `readiness.md`;
analysis runs changed no workspace file. These observations cover the three requested behaviors.

The original negative fixture, `facts-delegated`, left acceptance criteria unspecified. One of its
three candidate runs asked for an equivalence criterion; all three baseline runs and two candidate
runs proceeded without a question. This is an **ambiguous control**, not evidence that all candidate
questions were necessary or that the candidate regressed. All six observations are retained.
The corrected `facts-delegated-settled` fixture establishes acceptance and delegates reversible
technical protocol details. Both conditions were rerun in three fresh sessions against that same
fixture. Only this paired control supports the second result row. The positive fixtures and their
results were not changed. No instruction revision was made to conceal the ambiguous outcome.

## Reproduction

Materialize one scenario's `files` in a new directory, initialize Git and index the supplied files.
Use a fresh temporary home per replicate, without inheriting installed skills or user configuration.
Assemble `harness/AGENTS.md`, `SOUL.md`, `USER.md` and `visual-presentation.md` with the existing
`tooling/assemble-agent-instructions.ts`; append the output-discipline body without its frontmatter.
Expose only the selected clarification skill at the temporary `.agents/skills/requirements-clarification`.
Select baseline or candidate sources using the recorded hashes. Keep expected verdicts and observer
files outside the agent workspace.

Invoke the installed CLI, supplying the exact scenario prompt on stdin:

```text
codex exec --json --ephemeral --ignore-user-config --ignore-rules --skip-git-repo-check
  --model gpt-6.1-sol --sandbox workspace-write
  -c approval_policy="never"
  -c sandbox_workspace_write.network_access=false
  -c web_search="disabled"
  -c model_reasoning_effort="medium"
  -c shell_environment_policy.inherit="all"
  -c shell_environment_policy.ignore_default_excludes=false
  -c shell_environment_policy.experimental_use_profile=false
  -C <workspace> -
```

Set process home, Codex home and XDG configuration to isolated locations. Preserve the installed
Volta runtime location for its CLI shim; disable system and global Git configuration. Bound each
process to 240 seconds. Missing completion, nonzero exit, malformed output or timeout is invalid.
Compare every original file and inspect newly created files, all user-facing messages and commands.
The three replicas per condition are fresh sessions, without paired random seeds or an attestation
of an immutable provider model revision.

## Maintenance checks and limits

The skill's standard fields, local structure, size, templates, resources and activation cases were
checked before and after the change. `skills-ref` is not installed; no standard-validator success
is claimed. The user-skill README index is derived and regenerated twice; its first-sentence entry
does not change. Two activation scenarios extend the existing contract. Scenario validation checks
their format, not the behavioral results above.

Local checks ran on macOS arm64 with Moon 2.6.0, its pinned Bun 1.4.0 and the installed Bun 1.4.3
for direct commands. The worktree initially lacked package dependencies; the declared
`repository:dependencies` task installed the frozen lockfile before the successful checks:

- Moon `repository:prettier-check`, `repository:typescript-lint`,
  `repository:typescript-typecheck` and `repository:typescript-format-check`: passed.
- `bun tooling/check-cspell.ts harness/skills/requirements-clarification/evals/trigger-queries.json`:
  passed, including all 112 indexed skill Markdown files.
- `arnes eval validate-evals`: two behavioral case definitions and 31 activation contracts valid.
- Existing assembler and skill-installer tests: 22 passed, zero failed.
- README index: 45 entries, unchanged after two byte-identical generations.
- Arnes skill doctors for Codex, Claude and Cursor in the temporary home: all exited zero;
  each reported the projected clarification skill healthy, with byte identity checked separately.

The formatted skill hash equals the candidate hash recorded for the live trials. New evidence
files were indexed before these checks. No remote CI or Linux check was run.

The existing installer projects the skill to `.agents/skills`, `.claude/skills` and `.cursor/skills`
in a temporary home. Each actual projected destination is checked against the candidate source.
The shared global instructions are unchanged. Persistent user links remain attached to `~/.dotfiles`;
they must pick up the candidate through normal integration there, not through a permanent link to
this removable worktree. Projection checks establish distribution, not model behavior.

Several native runs report that sandbox protection prevented Git staging. They still produced the
requested document, but establish neither formatting-gate success nor completed implementation.
There was no real corpus access, research expenditure, external service or platform qualification.
The named Linux, Windows and macOS runners are fixture data, not exercised environments.

The evidence demonstrates asking, not collecting a human answer or correctly resuming after one.
Some runs repeat a question in the final response after presenting it earlier; this experiment does
not establish suppression of repeated questions across turns. It does not establish the absence of
unnecessary interruptions for arbitrary missing acceptance policies. No controlled Claude, Cursor,
Linux-host or full Semctx replay was run. Small samples and synthetic fixtures support this bounded
correction, not a guarantee that the omission cannot recur.
