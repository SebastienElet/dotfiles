---
name: do-nothing-script
description: >
  Turn a manual, repeated procedure into a do-nothing script (Dan Slimmon's gradual automation
  technique): one function per step, each printing its instructions and waiting for the operator,
  automated one step at a time. Use when documenting or automating a runbook, checklist, release
  process, onboarding, incident procedure, or any multi-step manual chore. Make sure to use this
  skill whenever a request mentions a runbook, checklist, "steps I always do by hand", gradual or
  partial automation, or asks to script a procedure whose steps cannot all be automated yet, even if
  the words "do-nothing script" are never used.
metadata:
  category: ops
---

# Do-Nothing Script

## Overview

A do-nothing script encodes a manual procedure as ordered step functions. Each step prints exactly
what the human must do, then blocks until they press Enter. It saves no time on day one — its value
is that the procedure stops living in someone's head, steps stop being skipped, and each step is
already an isolated function that can be replaced by real code later. Reference:
<https://blog.danslimmon.com/2019/07/15/do-nothing-scripting-the-key-to-gradual-automation/>.

## Usage

```text
$do-nothing-script turn the release checklist into a script
$do-nothing-script automate step 3 of scripts/provision_user
```

## Steps

1. Collect the procedure as an ordered list of steps, in the operator's own words. Ask for the real
   sequence rather than inventing one — a missing step is the failure this technique exists to fix.
2. Pick the language: Bash for repository scripts (follow the `scripts` skill), Python when steps
   need API calls, parsing, or state passed between steps.
3. Write one function (or class with `run()`) per step, named after the action:
   `create_ssh_keypair`, `add_user_to_ldap`. One step = one thing the operator does.
4. Each unautomated step prints its literal instructions — including exact commands, URLs, and
   values to copy — then calls a single shared `wait_for_enter` helper.
5. Interpolate the runtime context into the instructions (username, version, ticket id) so the
   operator copies text instead of adapting it.
6. Drive the steps from a main function that runs them in order and prints a completion line.
7. Automate one step at a time: replace its body with real code, keep its name and position, keep
   the rest manual. Never block shipping the script on automating everything.

```bash
#!/usr/bin/env bash
set -euo pipefail

wait_for_enter() { read -rp "Press Enter to continue... "; }

step_create_branch() {
  echo "Run: git switch -c release/${VERSION}"
  wait_for_enter
}

step_tag() { git tag -a "v${VERSION}" -m "release ${VERSION}"; }  # automated

main() {
  VERSION="${1:?usage: release VERSION}"
  step_create_branch
  step_tag
  echo "Release ${VERSION} done."
}

main "$@"
```

## Gotchas

- **Automating the whole procedure in one go** — the point is to ship the manual version today; a
  rewrite that stalls leaves the runbook back in someone's head.
- **Vague step text** — "deploy the service" makes the script useless to anyone but its author —
  print the exact command with values already substituted.
- **Steps that quietly do work while claiming to be manual** — an operator who trusts the prompt and
  also runs the command will double-apply it — a step is either printed or executed, never both.
- **Merging several actions into one step** — a fused step cannot be automated independently, which
  removes the only long-term benefit of the technique.
- **No `set -euo pipefail` in the Bash version** — a failing automated step must stop the run, not
  let the operator continue on a broken state.
- **Skipping the shared `wait_for_enter` helper** — inlining `read` in each step makes it impossible
  to add later behavior (logging, `--yes` non-interactive mode) in one place.

## Constraints

- Every step must be its own named function — no inline blocks in `main`.
- Manual steps must print copy-pasteable, context-interpolated instructions and then block.
- Never mix manual instructions and automated work in the same step.
- Automate incrementally; keep step names and ordering stable so the runbook stays recognizable.
- Bash scripts in this repository must also follow the `scripts` skill (portable, extensionless,
  `#!/usr/bin/env bash`).
- Do not store secrets in the script — instruct the operator to fetch them, or read them from the
  environment.
