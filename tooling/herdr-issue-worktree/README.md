# Issue Worktree

Ctrl-click a GitHub, GitLab or Linear issue URL in Herdr to choose Claude or Codex.
Escape or an empty provider answer cancels. Confirm a matching repository suggestion, or enter an
absolute repository path; cancellation before confirmation starts no agent and creates no checkout.
Linear always requires an explicit repository, followed by the composed workflow's read-only preflight.

The plugin retains the complete clicked URL and workspace context as JSON data. Git inspection uses
argv, canonical paths, verified remotes and bounded subprocesses, with inherited Git overrides removed.
The selected provider runs a preparation agent in a new native source tab or workspace, preserving
focus. Preparation reads the authenticated tracker and composes the installed
`herdr-issue-worktree` skill. That workflow owns the final native linked worktree, current identifier
and exact title, checkout policy and implementation handoff. Repository instructions and composed
workflows retain branch, eligibility, pull-request and lifecycle decisions.

New agents receive their initial prompt through native `agent start --` arguments. Single-line JSON
encoding preserves quotes, shell syntax, newlines and control characters as data. A timeout can occur
after execution has begun; startup errors trigger inspection, never another launch or prompt.
No trust dialog is answered. The picker reports preparation state, not verified implementation success.

Repeated clicks inspect native issue metadata and check Git common directory, linked checkout,
workspace label, branch, provider, pane occupant, cwd and stored session reference. Working agents
are retained without input. **Existing idle agents require continuation in their retained pane.**
Automatic continuation remains blocked on Herdr 0.9.3: `agent.prompt` accepts a working agent and
has no expected-session precondition; a managed name can survive a same-provider session change.
See the installed-version [prompt implementation](https://github.com/ogulcancelik/herdr/blob/v0.9.3/src/app/api/agents.rs#L103)
and [name ownership reconciliation](https://github.com/ogulcancelik/herdr/blob/v0.9.3/src/terminal/state.rs#L2353).

A per-session, per-issue exclusive reservation under `HERDR_PLUGIN_STATE_DIR/dispatch/` prevents
concurrent picker processes from provisioning twice. Normal outcomes release it. Interrupted or
uncertain dispatches retain it and report its path. Inspect native panes, agents and Git worktrees
before manually removing an abandoned reservation; elapsed time does not prove a request was unapplied.
Collisions, incompatible repositories, changed occupants and unavailable identity stop the launch.

## Installation and validation

`moon run repository:herdr-plugin` links this plugin alongside worktree cleanup on macOS, using the
existing optional Herdr installation. Bun, authenticated tracker access and the composed skills must
be available to the selected provider. No tracker engine or project catalogue is installed here.

Run the portable owned-behavior tests with:

```sh
bun --config=/dev/null --no-env-file test tooling/herdr-issue-worktree
moon run repository:typescript-lint repository:typescript-typecheck repository:typescript-format-check
```

On 2026-10-07, macOS ARM64 with Herdr client/server 0.9.3 (protocol 22) and Bun 1.4.0:

- Native manifest linking and Moon's expanded task inspection succeeded; the full installation task
  was not run from this worktree.
- A real Ctrl-click in an isolated named session opened the provider popup with the full synthetic
  GitHub URL, query and fragment. Escape left the original pane, agent count and focus IDs unchanged.
  A second activation while the popup was open was refused without replacing it.
- Native source creation and metadata reporting retained the returned pane/cwd and all focus IDs.
- A new Codex launch accepted an encoded initial prompt containing shell syntax and control
  characters. It stopped at trust with the expected provider/cwd and unchanged focus. No trust UI was
  answered; prompt execution and directory confirmation after trust remain unverified.

Authenticated tracker round trips, the final agent-driven worktree handoff, live lifecycle writes and
automatic continuation are not certified by these observations. Linux CI covers portable code only;
native Linux, Windows/WSL, remote machines and other Herdr versions were not exercised.
