---
name: assertledger
description: >
  Qualify regression tests with AssertLedger. Use when asked to check whether a test detects a
  declared bug or to replay AssertLedger evidence. Make sure to use this skill whenever that
  qualification is requested, even if AssertLedger is unnamed. Excludes routine TDD, coverage,
  general code review, and unsupported test runners.
compatibility: Requires AssertLedger CLI, Git, and Node.js 22.15 or newer; container execution also requires a Linux Docker-compatible daemon and a locally available digest-pinned image.
metadata:
  category: dev
---

# AssertLedger

## Overview

Evaluate the same regression candidate against corrected code, a declared fault, and a neutral
control. Preserve the observations and replay the resulting manifest before reporting the bounded
verdict. The operator supplies the fault and the control's meaning; the tool does not establish
overall program correctness, producer authenticity, or permanent freedom from flaky tests.

This locally maintained workflow follows the published
[AssertLedger 1.4.0 documentation](https://github.com/hoklims/assertledger/tree/v1.4.0/docs).

## Usage

Use `$assertledger <repository and regression>` or `/assertledger <manifest to replay>`.
For example: `Check whether the committed parity regression detects the supplied buggy revision.`
Use `tdd` for ordinary implementation and `proof-integrity-review` when the verification mechanism
itself changes. AssertLedger qualification is an additional requested workflow, not a blanket test gate.

The dotfiles minimal profile installs the CLI with `moon exec harness:assertledger` on macOS.
Outside this repository, obtain the compatible CLI through the host's declared tool installation;
the skill itself needs no dotfiles checkout at runtime. If the CLI is missing, report that prerequisite
instead of installing a package or modifying project configuration implicitly.

## Steps

1. Run `assertledger --version` and `assertledger doctor /path/to/repository --json`. This diagnostic
   is static and read-only. Inspect conflicts and framework support before planning execution;
   detection of a test command does not demonstrate that an adapter can run it.
2. For Git qualification, establish the buggy, corrected, and neutral committed revisions, the
   neutral reason, candidate path, unchanged base-test paths, and a new output directory relative
   to the repository. Retrieve supplied facts where possible; never invent missing control semantics.
   Consult the [Git guide](https://github.com/hoklims/assertledger/blob/v1.4.0/docs/git-regression.md)
   for eligibility: dependency-free `.js`, `.mjs`, or `.cjs` using `node:test`, unchanged base tests,
   and identical file-path sets apart from the candidate. Local uncommitted changes are ignored.
3. Establish the execution backend from the operator's authorization. A container run requires
   an operator-selected image pinned by digest, already present on a Linux daemon and containing
   Node. Trusted local execution is `UNSANDBOXED` and requires authorization for that scope.
   Never combine both modes or infer execution permission from installation or a project config.
4. Run the chosen command with the actual established inputs. For an authorized container run:

   ```sh
   assertledger check /path/to/repository --before BEFORE --after AFTER --neutral NEUTRAL --neutral-reason "Established control rationale" --test tests/regression.test.js --base-test tests/base.test.js --out .assertledger/evidence-001 --container-image node@sha256:DIGEST --json
   ```

   For authorized trusted local execution, replace `--container-image node@sha256:DIGEST` with
   `--allow-unsafe-execution`. Do not pass either flag before that backend is established.

5. Preserve the command's exit status, summary, executed request, and manifest. A complete output
   directory contains `manifest.json`, published last. Run
   `assertledger replay /path/to/repository/.assertledger/evidence-001/manifest.json --json`.
   Positive evidence requires `decision.status: VERIFIED` and replay `valid: true`; retain all
   failed attempts, controls, reason codes, execution limits, and isolation facts. Rejection,
   inconclusive observations, operational errors, or invalid replay cannot support acceptance.
6. Report the exact fault, revisions, selected candidate, environment, artifact locations, and
   limitations. Use `assertledger explain CODE --json` for refusals. Replay checks integrity and
   decision consistency without rerunning tests or authenticating their producer. Preserve source
   revisions, requests, and raw logs separately when reproduction or independent audit matters.
7. For an already supplied versioned request or a supported Bun campaign, read the matching
   [reference](https://github.com/hoklims/assertledger/blob/v1.4.0/docs/reference.md) before using
   `verify` or replay. The Bun adapter pins Bun 1.4.2 and credits failures from `assertledger/bun`
   `assertSame`; native Bun `expect` failures do not count as target detection. This lower-level
   path does not extend the Git `check` command's eligibility.

## Gotchas

- **Applying Git qualification to TypeScript/Bun or Rust** — static detection can look promising
  while `check` cannot execute the candidate; retain the existing native tests and report the limit.
- **Ignoring a repository symlink refusal** — removing links or inventing exclusions changes the
  declared evidence; preserve the checkout and report `UNSUPPORTED_REPOSITORY_SYMLINK`. Only the
  operator may declare a local-only exclusion after examining the documented rules.
- **Reusing an evidence directory** — output reservation fails rather than overwriting a previous
  campaign; select a new relative path and retain the prior evidence.
- **Counting a crash as detection** — timeouts, compilation, discovery, and process failures do
  not establish the claimed regression; inspect the reason codes and correct the execution problem.
- **Running client setup automatically** — `connect --write` installs a project skill and MCP config,
  creating another discovered copy of this user skill; preview only on a requested client setup and
  resolve that duplication before any write.

## Constraints

- Never manufacture fault semantics, neutral rationale, observed results, or execution authorization.
- Never initialize projects, register MCP servers, pull images, or change exclusions implicitly.
- Never execute an unsupported runner through a guessed adapter or weaken policy to obtain acceptance.
- Never treat replay validity as proof of actual execution, authenticity, or general correctness.
- Never discard failed observations, hide incomplete output, or treat a demo as repository evidence.
- Preserve native test and CI requirements; this skill neither replaces them nor authorizes a merge.
