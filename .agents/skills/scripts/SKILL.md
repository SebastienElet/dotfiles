---
name: scripts
description: >
  Choose and maintain repository scripting boundaries. Use when adding or editing shell, executable
  helpers, hooks, CI recipes, or script tests. Make sure to use it whenever parsing, validation,
  fallback, policy, state, or error mapping might be placed in Bash, even outside tooling/.
metadata:
  category: dev
---

# Scripts

## Overview

Keep Bash at the process boundary: bootstrap a machine, set environment, and invoke fixed commands
in a short linear sequence. Use Just for named development recipes, Bun and TypeScript for small
testable utilities, and Rust for substantial or system-oriented CLIs. Existing complex shell is
legacy to reduce, not a pattern for new work.

## Usage

Invoke this skill with a script-related request:

```text
$scripts add a bootstrap wrapper for an existing command
$scripts decide whether this shell helper belongs in TypeScript or Rust
```

## Steps

1. Inspect neighboring scripts, every caller, the applicable ADRs, and the behavior that tests
   actually exercise before choosing a language.
2. Classify the change as boundary glue only when it sets environment, validates a few arguments,
   invokes a fixed sequence of commands, forwards their output or status, or selects one explicit
   platform branch. Use Bash only for that case.
3. Stop before writing Shell when the behavior parses data, combines values, owns fallback or policy,
   tracks state, retries work, or maps errors. Use Bun and TypeScript when that behavior remains a
   small cohesive utility with bounded effects and a simple interface.
4. Use Rust when the tool needs a substantial CLI contract, durable state, complex concurrency,
   performance, distribution as a self-contained binary, privileged system integration, or stronger
   compile-time modeling than the TypeScript utility warrants.
5. For a defect in legacy Shell, add the smallest regression fix only when it does not expand the
   script's responsibility. A feature that crosses the boundary migrates the affected behavior
   first rather than adding another branch or helper.
6. Test the production artifact through its real entry point. Never define a second implementation
   inside a test, and never claim a smoke test proves behavior that exists only in instructions.
7. For TypeScript, keep production code and `bun:test` tests in normal modules. Pin Bun and
   TypeScript 7, commit the lockfile, run `bun test`, and gate `tsc --noEmit` in CI; Bun's runtime
   execution is not evidence that type-checking passed.
8. Parse every external or structured input once at the TypeScript boundary with a Zod schema,
   derive the trusted type from that schema, and fail closed with the validation error.
9. Keep non-trivial fakes, parsers, fixtures, and assertions outside Shell. Do not embed TypeScript,
   Python, Ruby, or another implementation language inside a Shell heredoc.
10. For a permitted Bash wrapper, use `#!/usr/bin/env bash`, a descriptive extensionless kebab-case
    name under `tooling/`, meaningful non-zero exits, and commands portable to macOS and Linux.
11. Preserve the executable bit, run `bash -n` and ShellCheck on the exact file, and exercise the
    wrapper on every platform whose behavior it can change.
12. For destructive behavior, default to inspection or dry-run and refuse unsafe paths.

## Gotchas

- **Following the nearest Bash precedent** — legacy placement is not architectural approval, so a
  new feature silently inherits parsing and policy debt; classify the behavior before copying it.
- **Testing functions declared by the test** — the harness proves its own duplicate while the real
  artifact can still fail; invoke the shipped entry point and observe its outputs and effects.
- **Embedding another language in a heredoc** — two runtimes and quoting layers hide the true
  implementation boundary; create a normal TypeScript or Rust module with ordinary tests instead.
- **Treating Bun execution as type safety** — Bun strips types without proving them; run the pinned
  TypeScript 7 compiler with `--noEmit` as a separate gate.
- **Hand-writing input type guards** — validation rules and trusted types drift apart; define the
  boundary schema in Zod and infer the downstream type from it.
- **Choosing Rust for every parser** — small utilities pay compilation and implementation overhead
  without gaining a material guarantee; prefer typed, validated TypeScript until the CLI boundary or
  system constraints justify Rust.
- **Depending on modern Bash features** — macOS may provide an older Bash — avoid features such as
  associative arrays unless the script explicitly verifies a compatible runtime.
- **Using GNU-only utilities** — flags supported on Linux may fail on macOS — prefer portable syntax
  or branch explicitly by platform.
- **Testing destructive commands against real data** — a correct script can still remove the wrong
  path — validate with temporary fixtures and a dry-run before real execution.

## Constraints

- Bash may own only bootstrap and short, linear process orchestration.
- Never add Bash that parses JSON, YAML, TOML, provider responses, or other structured external
  values; joins evidence; implements fallback, policy, or state; or translates error domains.
- Never create or extend a custom Shell test framework, duplicate production behavior in a test, or
  embed another implementation language in Shell.
- Use Bun and TypeScript as the default for small repository-owned utilities that exceed the Shell
  boundary; require Rust only when its delivery or correctness properties are material.
- Every TypeScript utility must be covered by `bun test` and a pinned TypeScript 7 `tsc --noEmit`
  gate that runs in CI.
- Every external or structured TypeScript input must be validated with Zod at its boundary; do not
  replace the schema with casts, handwritten type guards, or permissive defaults.
- Use `#!/usr/bin/env bash` for every permitted new executable Shell wrapper.
- Keep executable script names extensionless, descriptive, and kebab-case.
- Never ignore command failures that affect correctness.
- Never assume GNU-specific commands or flags are available on macOS.
- Do not rename existing scripts or change their interface without checking all callers.
