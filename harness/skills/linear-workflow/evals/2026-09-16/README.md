# Linear skill evaluations — 2026-09-16

Evidence for [#246](https://github.com/SebastienElet/dotfiles/issues/246), limited to
`linear-issue-spec`, `linear-start`, `linear-sync`, and `linear-workflow`.
These are manual live evaluations, not Arnes runner reports. They are kept beside the evaluated
skills rather than under `harness/evals/evidence/`, whose schema describes a different experiment.

## Results

| Case                      | Expected routing and decision                                  | Codex    | Claude Code  | Cursor       |
| ------------------------- | -------------------------------------------------------------- | -------- | ------------ | ------------ |
| [start](start.json)       | `linear-start` + `linear-workflow`; stop at merged attached PR | 3/3 PASS | Not verified | Not verified |
| [spec](spec.json)         | `linear-issue-spec`; provisional functional draft              | 3/3 PASS | Not verified | Not verified |
| [checkbox](checkbox.json) | `linear-sync` + `linear-workflow`; refuse completion           | 3/3 PASS | Not verified | Not verified |
| [prose](prose.json)       | Same refusal and review destination as checkbox case           | 3/3 PASS | Not verified | Not verified |
| [verify](verify.json)     | `linear-workflow`; missing measurement remains unproven        | 3/3 PASS | Not verified | Not verified |
| [negative](negative.json) | Status lookup; none of the four skills activate                | 3/3 PASS | Not verified | Not verified |

All 18 Codex processes exited successfully. No failed or invalid execution was discarded, and no
skill correction or corrective replay was necessary. Each report identifies its three executions.
The same prompts also exercise sibling exclusions: drafting did not activate execution policy,
starting did not activate shaping, and explicit verification did not activate synchronization.

Claude Code 2.1.236 and 2.1.272 reported `loggedIn: false`; Cursor Agent
2026.09.08-6caf4ff and 2026.03.18-f6873f7 reported `Not logged in` outside the sandbox.
No model request was made on either platform. Their 18 planned executions each remain missing;
the preflights are availability observations, not failed skill runs or three repetitions.

| #246 acceptance criterion                                                | Status                                                                  |
| ------------------------------------------------------------------------ | ----------------------------------------------------------------------- |
| Four valid scenario files                                                | Satisfied by local doctor and native scenario validation                |
| Three completion refusals per supported agent                            | Partial: Codex satisfied; Claude Code and Cursor unverified             |
| Equivalent prose and checkbox refusal                                    | Observed on Codex, three runs of each; other hosts unverified           |
| Three negative-routing runs per supported agent                          | Partial: Codex satisfied for all four skills; other hosts unverified    |
| Per-execution prompts, results, environment and coverage limits retained | Satisfied for every executed run; unavailable hosts explicitly recorded |

This delivery does not establish completion of #246 across all supported agents. The PR links the
issue without closing it. Deterministic checks do not fill the 36 missing live executions.

## Method

Six identical prompts were each submitted three times to new Codex CLI processes. Every execution
had its own temporary home, Codex home, and synthetic workspace, with no conversation resumed.
The only installed user skills were copies of the four canonical Linear skills and their
references. Codex also installed its bundled system skills. The scenario files, expected answers,
previous outputs, repository instructions, user configuration, rules, hooks, and MCP configuration
were not supplied to the evaluated agent. The normal Codex system prompt remained in effect.

[context.md](context.md) is the exact workspace `AGENTS.md` supplied to every run. Each result
retains the full user prompt and its hash, command, date, duration, exit status, messages, successful
file-read observations, usage, and manually assessed verdict. [environment.json](environment.json)
records versions, source revision and hashes, and unavailable platforms. Absolute temporary paths
are replaced with `/fixture`; host account paths are replaced with `/host-user`. Session identifiers
and credentials are omitted. Tool output is retained as a byte count and SHA-256 rather than
repeating skill bodies. Agent messages are preserved without editorial corrections.

The invocation used `codex exec --json --ephemeral --ignore-user-config --ignore-rules
--skip-git-repo-check --sandbox read-only --model gpt-6-astra -c
'model_reasoning_effort="high"' --cd /fixture/workspace -`, with the prompt on stdin.
An allowlisted process environment preserved PATH, TMPDIR, LANG and SYSTEMROOT when present;
HOME and CODEX_HOME pointed to the per-run fixture, and VOLTA_HOME retained the existing CLI
installation. Existing Codex authentication was copied privately for the model request, never
included in evidence or offered as task data. At most three independent processes ran concurrently;
each had a 240-second timeout. No reusable runner was added.

## Verdict boundaries

- Positive routing requires an observed successful read of the expected skill and a response
  governed by its procedure; a claim of activation alone is insufficient.
- Negative routing requires no read or invocation of any of the four Linear skills, and a response
  limited to the requested status and assignee. Descriptions being available is not activation.
- Completion refusal requires the response to reject `Done` because the named criteria are
  unproven, preserve the residue, and propose `In Review`. Merely observing no write in the
  read-only sandbox does not pass this criterion.
- The prose and checkbox cases have identical facts and wording except for the acceptance-section
  syntax. Compare their lifecycle decisions, not their prose wording.
- The explicit verification case must not invent a performance measurement or treat formatting CI
  as its substitute. Its response must retain the missing oracle and refuse completion.
- An unsuccessful process, timeout, or incomplete transcript is `INVALID`, not `PASS`.

## Scope and limitations

These runs measure selection and decisions in synthetic, read-only contexts. They do not measure
successful implementation, real connector operations, concurrency, atomicity, complete product
preparation, or safety under every possible prompt. The `start` case covers selection and the
existing stop condition for an attached merged PR; it does not cover successful branch creation.
The product sources and `issue-creation` companion are intentionally unavailable; the `spec` case
covers a provisional functional draft with missing decisions, not publication or that companion.
Built-in system skills remain available, but the full deployed user-skill collection and competing
plugins are outside this experiment. Native skill discovery is exercised only for Codex.

The read-only fixture and missing conditional-write capability also prohibit writes independently
of the evidence rule. Refusal verdicts therefore rely on the agent's explicit evidence-based
classification, not on the absence of mutations. This is not a counterfactual experiment with a
write-capable connector. No live Linear or Bitbucket request was made.

The [#245 observation](../../references/linear-guard-observation-2026-09-10.md) remains unchanged:
the identity replacement is unusable on the observed transport. No identity guard was added,
enabled, or retried, and no description-changing workaround was introduced.

The requested model and effort are recorded; the provider's resolved model revision and random
seed are not exposed. Three repetitions characterize these executions only. The supported hosts
are Codex, Claude Code and Cursor under ADR-026; an installed binary without authentication is not
a completed evaluation. Missing hosts remain unverified rather than receiving synthetic passes.

## Deterministic and procedural validation

The four skills passed the local `skill-manager` doctor procedure before adding scenarios:
standard frontmatter fields and scalar metadata, descriptions below 400 characters, required
sections in order, at least three gotchas and constraints, fewer than 500 lines, linked resources,
no positional shell placeholders, no duplicate project slug, and one matching README entry.
The project adapters resolve to `../.agents/skills`; the three user projections for every skill
resolve to the canonical main-checkout source, whose skill bytes match this evaluated checkout.
Their initially absent optional scenario files were the scope of this issue, not a pre-existing
doctor failure. No skill body, description, router, or transport rule was changed.

Standard validation: unavailable (`skills-ref` not installed). No validator was installed.
The existing `arnes eval validate-evals` validates the new scenario contracts; it does not execute
agents or assess these manual verdicts. Prettier covers the changed JSON and Markdown. The skill
index is regenerated from frontmatter and remains byte-identical. No new validator, CI gate,
or mirror test was added. Validation is local macOS arm64 evidence; remote Linux CI is reported
separately in the PR. No code comments were added.
