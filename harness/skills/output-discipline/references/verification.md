# Verification record

## Environment and deterministic checks

Executed on 2026-09-11, macOS 26.6.2 arm64; Bun 1.4.0, Codex CLI 0.153.4,
Claude Code 2.1.236. The repository also supports Linux checks, but Linux was not exercised here.
No commit, push, publication or deployment to the real HOME was performed.

| Check                                                                                                                                                                              | Observed result                                                                                                                                                           |
| ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `cargo test --manifest-path tooling/arnes/Cargo.toml --locked --test output_discipline --test hooks_setup --test hooks_doctor --test hooks_reconciliation --test hooks_validation` | 72 tests pass, including 12 loader/integration tests.                                                                                                                     |
| `cargo fmt --manifest-path tooling/arnes/Cargo.toml --check`                                                                                                                       | Pass.                                                                                                                                                                     |
| `cargo clippy --manifest-path tooling/arnes/Cargo.toml --workspace --all-targets --all-features --locked -- -D warnings`                                                           | Pass.                                                                                                                                                                     |
| `bun test tooling/install-agent-skills.test.ts tooling/assemble-agent-instructions.test.ts`                                                                                        | 9 tests pass, 29 assertions.                                                                                                                                              |
| `bun run typecheck`, `bun run lint`, `bun run format:typescript:check`                                                                                                             | Pass; format covers 162 tracked TypeScript files.                                                                                                                         |
| `bun tooling/check-prettier.ts --check '**/*.{yml,yaml,md,json}' '!home/.config/nvim/lazy-lock.json' '!home/.config/nvim/lazyvim.json'`                                            | Pass.                                                                                                                                                                     |
| `tooling/arnes/target/debug/arnes eval validate-evals`                                                                                                                             | 3 behavioral cases and 16 activation contracts valid; structural validation, not execution of every query.                                                                |
| `tooling/arnes/target/debug/arnes eval validate-evidence`                                                                                                                          | 0 historical reports; live evidence is optional.                                                                                                                          |
| Skill-manager procedural doctor                                                                                                                                                    | Required sections, explicit routing pair, metadata/category, scope, resource links and README membership checked; approved `disable-model-invocation` extension retained. |
| Index regeneration                                                                                                                                                                 | Two generations followed by Prettier are byte-identical (`shasum -a 256 -c`); only one new index row.                                                                     |
| Project adapters                                                                                                                                                                   | `.claude/skills`, `.cursor/skills`, `.codex/skills` still resolve to `../.agents/skills`.                                                                                 |

Standard validation: unavailable (`skills-ref` not installed).
Moon is not installed: `moon run harness:check` and native Moon graph/deployment validation were
not executable. A full Cargo test attempt stopped at the existing
`deployment_contract::moon_deployments_satisfy_instruction_rule_and_skill_doctors` test with
OS `NotFound` when launching Moon. The aggregate suite is therefore **not green**; remote CI has
not run. No fake Moon, skipped test or replacement gate was introduced.

The new Rust tests were observed failing before implementation. They execute the shipped CLI
with temporary homes: absent/present marker, custom config directory, missing/unreadable source,
LF/CRLF/trailing-space/unclosed frontmatter, canonical deployment from another working directory,
repeat injection and no mutation. A further regression reproduced injection of another project's
sentinel when the deployed manifest was absent; the correction requires the deployed manifest
symlink and now leaves those cases non-blocking without stdout.

Setup tests cover each host independently, preservation of unrelated handlers, idempotence,
removal after undeclaration, unsupported Cursor and doctor detection of matcher/execution drift.
Doctor diagnoses installation; it does not attest model compliance or host trust.

The real `install-agent-skills.ts` entry point deployed both host projections into a disposable
HOME, and both links resolved to this checkout's canonical skill. The real instruction assembler
also produced a temporary Codex projection with the approved SOUL removals. No global installation
dependencies were run.

## Live loading and lifecycle

Lifecycle probes used the Codex default `gpt-6-astra`, separately from the controlled
`gpt-5.6-sol` presentation sample below. Manual compaction was exercised; automatic threshold
compaction was not.

Arnes installed only the requested hook in a temporary manifest, then actual host processes ran
from a separate temporary project. Codex auth was copied into the temporary HOME and removed
at the end; no personal configuration was imported. Codex's one-invocation
`--dangerously-bypass-hook-trust` was used only for these inspected fixture hooks, without
bypassing the read-only sandbox. Normal deployment still requires review in `/hooks`.

| Path                                | Evidence and limit                                                                                                                                                  |
| ----------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Codex manual discovery/activation   | Five actual sessions contain the complete injected `<skill>` block, with the expected name/body; control sessions do not.                                           |
| Codex automatic startup             | Session rollout contains `OUTPUT DISCIPLINE ACTIVE` and the canonical body; the model identifies the mode.                                                          |
| Codex resume                        | Repeated real `exec resume` calls inject the body again; confirmed in the rollout.                                                                                  |
| Claude discovery                    | Debug log loads 22 user skills from the fixture symlinks; `/output-discipline` is submitted, but model execution fails before a response.                           |
| Claude automatic startup            | Debug log reports `Hook SessionStart:startup (SessionStart) success` followed by the complete active message and body. This occurred before authentication failure. |
| Claude response, resume, compaction | Unverified: isolated CLI returns `Not logged in · Please run /login`; no login or active configuration change attempted.                                            |
| Opt-in absence/errors               | Exercised through the shipped loader's subprocess tests; not a claim that every lifecycle event was executed in both hosts.                                         |

The initial adaptation was tested with `stop output discipline`, followed by a real Codex resume
while the marker remained. The model first reported deactivation, then explicitly reported
reactivation after resume. This is one observed failure of conversational deactivation persistence,
not a universal model guarantee. After that result, the user chose to remove deactivation entirely;
the final skill and loader contain no off command and no new session state.

## Small behavioral sample

Five paired cases, one observation per variant, same prompts and Codex `gpt-5.6-sol` model;
12 successful CLI calls including a second real turn for the multi-turn case. Each call was
bounded to 90 seconds, with at most two concurrent processes. HOME, CODEX_HOME and workspace were
isolated per variant; the active variant added explicit `$output-discipline` invocation.
User config and exec rules were ignored, sandbox was read-only, effort remained the CLI default.
No new evaluation tooling was installed or committed.

The tested skill SHA-256 was
`50a96c02b3b3db0aba6fbddedc34bb05342c884d2d8ff099baf638b2aeec1190`.
This was before the user's removal of conversational deactivation; the ten presentation rules,
six exceptions and pre-send check are unchanged since that sample. The observations below must
not be represented as a fresh full evaluation of the final skill revision.

| Stable case                               | Decisive information retained                                                                | Observed noise or limitation                                                                                 |
| ----------------------------------------- | -------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------ |
| Git: list tracked modified files          | Both keep unstaged/staged/HEAD variants and exclusion of untracked files.                    | The baseline places the combined answer first; no consistent advantage from the skill.                       |
| Multi-turn: build succeeded, tests remain | Both retain build status and pending tests; skill makes step state explicit.                 | Skill is longer and initially labels tests in progress without evidence; corrected after user clarification. |
| HTTP 401: valid token, missing header     | Both explain missing `Authorization`, `Bearer` format and provide a fictitious curl example. | Skill raises the correction earlier but still repeats the header.                                            |
| Merge/rebase: shared branch               | Recommendation, both options, commit identity, coordination and force-push risk survive.     | Skill groups trade-offs well; its final next action still bundles several actions.                           |
| Detailed explanation of isolation         | Both retain depth, anomalies, concurrency, constraints, retries and external side effects.   | Skill is longer and still emits a six-item group and closing synthesis.                                      |

The full prompts, responses, versions, source hash and observations are delivered separately as
`output-discipline-behavior-report.md`. They are model outputs for presentation analysis, not
verified technical advice. A single pair per case establishes neither statistical superiority nor
perfect compliance, and the existing baseline was already concise. No word-count threshold was
used as an oracle; no factual uncertainty or warning was deliberately removed to shorten a response.

An independent static review found the canonical-source fallback defect, which was corrected;
no other actionable finding remained. Added code comments: none.
