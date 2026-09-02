# Breaker adjudication report

## Revision

- Date: 2026-09-03
- Branch: `codex/159-named-invariant-registry`
- Adjudication base: `5d6d0bbab4f29530bc69b28b0e8bfac99d0f95e9`
- Final commit: reported with delivery because a commit cannot contain its own identifier

## Result

The generic production surface writer has been removed. The registry code no longer exposes a
filesystem, lock, staging, compensation, or workflow API that can replace an arbitrary surface.
Surface application is routed by the workflow contract to its mandatory owner:

| Surface                                 | Exact path                                   | Required owner                   |
| --------------------------------------- | -------------------------------------------- | -------------------------------- |
| `always-loaded-instruction`             | `harness/AGENTS.md`                          | `agent-instructions`             |
| `conditional-skill`                     | `harness/skills/harness-reflection/SKILL.md` | `skill-manager`                  |
| hook or lint                            | owner-specific path                          | `scripts` and `enforcement-code` |
| permission, type, or architectural test | owner-specific path                          | `enforcement-code`               |

The first two rows are the only destinations accepted by the bounded manifest validator. The
enforceable rows deliberately use their separate owner workflow and are rejected by that generic
validator. The owner applies the approved exact diff and runs its doctor or contracts before the
registry replacement is recorded.

The remaining mutation code is read-only validation. It parses a strict approval attestation,
binds its paths, preimages, replacements, target identifier, and before/after records to the exact
request, derives the mutation kind from the lifecycle transition, and rejects no-op files. It does
not authenticate whether an attestation came from a human or whether `approvedBy` identifies that
person. Contextual human approval remains a procedural skill precondition.

For promotion, the supported surface preimage must not contain `candidateTextExact` and its
replacement must contain it. Retirement requires the inverse. These checks prove only exact text
presence or absence; the required owner doctor supplies the surface-specific check. They do not
claim to decide the general meaning of the text.

Retired records are terminal. An active-to-retired transition preserves every historical field
except `approval`, `lifecycle`, and `retirement`. Its newly accepted attestation must equal the new
persisted approval record; it is not compared with the earlier approval.

## Runtime oracle boundary

`validateInvariantRegistryText` remains a structural and repository-policy library boundary. It
does not execute tests. After that validation succeeds, the executable CLI runs the exact declared
`bun test <testPath>` invocation for each `verified` record whose last measurement names an oracle.
The CLI fails when the invocation cannot start or exits nonzero.

The runtime regression first passes the same fixture through the library validator, then invokes
the shipped CLI in a subprocess. Its tracked `failing.test.ts` throws through a real Bun test
process, and the CLI exits nonzero. Direct execution also produced one actual failing test and exit
status 1.

## Consumer and destination closure

One canonical mapping now drives both mutation validation and registry policy:

| Surface                   | Claude                      | Codex                      | Cursor              |
| ------------------------- | --------------------------- | -------------------------- | ------------------- |
| always-loaded instruction | `claude-global-instruction` | `codex-global-instruction` | unsupported         |
| conditional skill         | `claude-user-skill`         | `codex-user-skill`         | `cursor-user-skill` |

Consumer mechanisms are closed per agent in the registry schema. Policy and CLI tests reject a
nonexistent adapter, a mechanism owned by another agent, and Cursor user-skill support on an
always-loaded instruction. Manifest tests reject `README.md`, `package.json`, production validator
code, and the registry reference before any write can occur.

## Integration evidence

The historical PR 206 and PR 207 fixtures retain and assert their recorded GitHub pull-request and
comment URLs. The integration parses their real stored source values, builds two distinct candidate
proposals, validates each exact registry manifest, and sends their combined registry through global
source de-duplication and policy. This covers historical source-to-proposal admission only; it does
not prove a complete promotion or surface application.

A separate fixture is explicitly marked `synthetic-local-not-historical`. In a temporary Git root
it performs this observable sequence:

1. parse two valid synthetic forge sources and build the approved promotion manifest;
2. apply the exact instruction replacement through a fixture-only owner stand-in;
3. read the fixture files and validate the applied surface plus unchanged registry preimage;
4. record the approved registry replacement;
5. run the copied production CLI, which executes the tracked workflow-state oracle;
6. remove the exact candidate text for retirement and validate the applied files;
7. prove that the still-active registry makes the real CLI/oracle fail;
8. record the exact retired registry and prove the CLI/oracle passes;
9. read the fixture root and assert the exact retired registry bytes and surface bytes.

The test helper is not a production writer and does not stand in for human authentication. The
intermediate failing CLI kills the mutant where retirement reports success without recording its
registry replacement.

## CSpell ownership

`tooling/cspell-texts.ts` is the source of the checked text path list and the CI entry point. It uses
only Bun and JavaScript standard facilities, so a clean copied checkout does not need project
packages to launch it. Functional tests prove the exact CSpell command is called, its nonzero status
is propagated, and a missing executable fails closed. No test parses workflow YAML and no YAML copy
of the path list exists.

CI still installs the exact `cspell@10.2.0` and `@cspell/dict-fr-fr@2.3.2` pins, links the French and
user dictionaries, and always invokes the owned entry point. The real CI recipe was replayed with
an isolated temporary home: 76 files checked, 0 issues. The three singular technical forms added by
this change are recorded once in the existing user dictionary.

## RED and mutant evidence

Tests were introduced against the load-bearing failures before the corresponding implementation.
The resulting oracles now reject:

- an approval record or manifest that does not equal the request;
- caller-selected mutation kind, retired-to-active, mutable retirement sources or exceptions, and
  a retirement record that does not contain the new exact attestation;
- each isolated manifest mismatch: path, replacement, preimage, delta before, delta after, and an
  equal before/after delta;
- a no-op surface, absent promotion text, already-present promotion text, retained retirement text,
  unsupported destination, and consumer mismatch;
- reintroduction of a generic surface writer or generic mutation-execution contract;
- a verified runtime invocation different from its declared oracle, plus a real failing invocation;
- retirement surface removal without the corresponding registry update;
- application ordered before selection, proposal, exact manifest, or approval;
- a CSpell boundary that imports project packages or masks command failure.

These are the executed observable oracles. No behavioral result is inferred from an unexecuted
agent scenario.

## Verification evidence

Environment: local macOS 26.6.2 arm64, Bun 1.4.0, Node 24.20.0, TypeScript 7.0.2, CSpell 10.2.0,
and Actionlint 1.7.12.

- Full Bun suite outside the filesystem sandbox: 598 passed, 2 explicit opt-in skips, 0 failed,
  1401 assertions across 74 files in 38.45 seconds.
- Explicit skips: real Docker lifecycle and multi-worktree ColGrep integration. No result is claimed
  for those opt-in paths.
- Targeted breaker suite: 75 passed, 0 failed.
- Direct failing runtime oracle: 0 passed, 1 failed, exit status 1, as required by the negative
  oracle.
- Canonical empty-registry CLI: passed.
- `bun run lint`: passed.
- `bun run typecheck`: passed.
- `bun run format:typescript:check`: passed.
- CI-equivalent CSpell entry point with both dictionaries: 76 files, 0 issues.
- Actionlint on `.github/workflows/lint.yml`: passed.
- `make -n codex`, `make -n claude-code`, and `make -n cursor`: passed.
- Dedicated Oxlint `max-lines-per-function` check on every changed production TypeScript file:
  passed.
- `git diff --check`: passed.

Every changed production and test TypeScript file is below 250 lines. The largest changed test is
`harness-reflection-contract.test.ts` at 242 lines; the largest changed production file is
`invariant-registry-policy.ts` at 224 lines. `invariant-registry-contract.test.ts` remains below the
trigger. The longer changed files are plans or the progressively disclosed skill reference, not
production or test code.

## Skill doctor and evaluation state

Manual `skill-manager` doctor for `harness-reflection` found the local contract healthy:

- frontmatter, slug, `dev` category, description, required section order, four gotchas, and four
  constraints pass;
- the skill is below 500 lines, has no templated shell placeholders, has one canonical slug and one
  README entry, and routes its scoped reference explicitly;
- trigger queries parse with positive and negative cases;
- the unchanged frontmatter derives the same README bytes, SHA-256
  `a3b24a3546bc8ea980cadd2f65f625417c74c309d6bea4f49e557e3a203c633e`;
- `skills-ref` is not installed, so standard validation is reported as unavailable rather than
  passed.

The deployed Arnes doctor reports `harness-reflection` healthy for Claude, Codex, and Cursor. Its
whole-command status is nonzero because of unrelated pre-existing plugin drift for Codex and absent
deployed `memory-governance` destinations for Claude and Cursor; no clean aggregate doctor result is
claimed.

The changed skill SHA-256 is
`4772b71eeb655fb2eb13672d290e6a3d8f1e9e8d7520e034d21704aa860dc37c`. The exact
`SKILL.md + NUL + reference` digest is
`a4fa1e7647e0dc0db902ab9278fd60abac1358673ecae6f28a67a0b0195f9808` and matches the evaluation
artifact. Its status is `pending`, `runs` is empty, and no branch is reported as covered.

## Preserved scope

- `harness/invariants/registry.json` remains exactly version 1 with an empty invariant list.
- `harness/skills/pr-feedback`, `tooling/arnes`, `home/.arnes.yaml`, `Makefile`, and the deployment
  topology are unchanged.
- `.github/workflows/lint.yml` is unchanged; its pins and mandatory CSpell call remain intact.
- No real invariant was promoted or retired.

## Limits

- The code accepts and records an approval attestation and proves exact consistency. It cannot prove
  human origin or identity.
- `validateAppliedHarnessMutation` validates the file snapshot supplied by its caller. The synthetic
  integration supplies bytes read from its real fixture root; production owner workflows remain
  responsible for reading their actual applied surface.
- There is no multi-file transaction, atomicity, cooperative lock, compensation, concurrency, or
  hard-crash recovery guarantee. An interruption between the owner surface change and registry
  recording can leave a visible intermediate Git diff that requires reconciliation.
- The CLI runs only the validated invocation declared for a verified record. It provides no timeout,
  network isolation, future-validity guarantee, or result for undeclared oracles.
- The historical fixtures prove only their named source, de-duplication, policy, proposal, and
  manifest path. The full local sequence uses explicitly synthetic sources and a fixture-only owner
  stand-in.
- Verification was executed on local macOS only. No Linux or hosted run is claimed.
- Independent `design-claim-auditor` execution was unavailable because this adjudication explicitly
  prohibited subagents. The design-claim contract test passed in the full suite; this is not an
  independent review.

## Comments added

None.
