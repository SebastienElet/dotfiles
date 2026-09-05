---
name: code-enforcement
description: >
  Write code whose purpose is to refuse: hook, guard, validator, permission check, lint rule, CI
  gate. Use when adding or changing any control that decides pass or fail. Make sure to use it
  whenever a check can be bypassed or fail to run, even if it is called a wrapper or a policy.
metadata:
  category: dev
---

# Code Enforcement

## Overview

A guard is judged by what it refuses when someone tries to get around it, not by whether it passes
its happy path. The two failure modes that survive review are a control that inspects a
representation of the dangerous value instead of the value itself, and a control that returns
success because it could not run. Both look green in every test written from the author's own
assumptions.

## Usage

Follow the steps below when writing or changing a hook, a guard, a validator, a permission check, a
lint rule or a CI gate — before opening the file, since step 1 often changes where the code belongs.

Typical cases: "block `rm -rf` outside the repository" (step 1 moves the decision off the command
string), "fail CI when a migration has no rollback" (steps 3 and 4 stop the gate from passing when
the migration list comes back empty), "refuse a push without a signed commit" (step 5 enumerates the
alias, the `--no-verify` and the second remote).

## Steps

1. **Name the sink.** Write down the exact value that reaches what you protect: the expanded argv,
   the request body, the SQL parameter, the resolved path. Validate that value.
2. **Check whether you can actually see it.** If the code only has a representation of the value — a
   shell command string, a diff, a log line, a commit message — the decision is a heuristic, not a
   check. Say so in a comment and in the delivery note, and label that layer advisory. Do not let a
   heuristic be the only barrier for a consequence you cannot undo.
3. **Fail closed.** Missing runtime, unreadable input, parser exception, unresolved variable, empty
   result: refuse, and name what was missing. A guard that exits 0 because it could not run is a
   disabled guard, and it stays disabled silently for months.
4. **Keep the probe's failure visible.** No `2>/dev/null` on the command that decides whether the
   guard can run, and no branch that turns an empty result into success. If the probe is allowed to
   fail quietly, step 3 is unenforceable.
5. **Enumerate the bypasses before writing the fix.** List how the control can be satisfied or
   skipped: compound command, duplicate flags, a second invocation, an alias or a function, a
   missing interpreter, another host, a subshell, a renamed file. Keep the list in the commit body
   or the test file.
6. **Turn each entry of that list into a test case** wired into the project's automated checks. A
   manual self-test does not count: it disappears with the session that ran it.
7. **When a bypass is cheap and its consequence real, run an adversarial pass.** Dispatch a
   fresh-context subagent whose mandate is to break the guard, not to review it; its output is the
   list of attempts and their results. Skip this for a control whose worst case is a warning.

## Gotchas

- **Parsing a shell command to decide.** Quoting, `$()`, `&&`, aliases and `env` prefixes all
  change what finally executes. Such a layer is a speed bump; the real check belongs where the
  command's effect lands.
- **`command -v` guards.** `if command -v tool > /dev/null; then check; fi` silently skips the
  check on any machine missing the tool — the exact machine that needs it most. Invert it: refuse
  when the tool is absent.
- **A discovery step that finds nothing.** `git grep -E` does not support `\b`, so such a pattern
  matches zero files without erroring, and the gate passes having read nothing. Guard the step
  with a canary: a file you know must appear in its output.
- **Testing only the refusal you implemented.** The interesting test is the input you did not
  think of; that is what step 5 produces.
- **A guard tested only by hand.** It passes on the author's machine, then rots. If the project has
  no place to wire the test, that missing harness is part of the work.
- **The context that wrote the guard cannot break it.** It shares the blind spots that produced the
  hole, and will confirm its own design. Only a fresh context is adversarial.

## Constraints

- Never decide on a proxy for the protected value when the value itself is reachable.
- Never exit 0 on a path where the guard could not do its job; refuse and name the cause.
- Never suppress the errors of the probe that establishes whether the guard can run.
- Every enumerated bypass must exist as an automated test before delivery.
- State plainly which layer is advisory and which is enforcing; a heuristic presented as a
  guarantee is worse than no guard.
