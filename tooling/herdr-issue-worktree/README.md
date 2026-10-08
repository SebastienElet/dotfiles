# Issue Worktree

Ctrl-click a GitHub, GitLab or Linear issue URL in Herdr to choose Claude or Codex.
Escape or an empty provider answer cancels. Confirm a matching repository suggestion, or enter an
absolute repository path; cancellation before confirmation starts no agent and creates no checkout.
GitHub repository names and issue keys compare without owner/project casing differences;
the clicked URL and context retain their original text. Other hosting paths keep their exact comparison.
Linear always requires an explicit repository, followed by the composed workflow's read-only preflight.

The plugin retains the complete clicked URL and workspace context as JSON data. Git inspection uses
argv, canonical paths, verified remotes and bounded subprocesses, with inherited Git overrides removed.
The selected provider runs a preparation agent in a new native source tab or workspace, preserving
focus. Preparation reads the authenticated tracker and composes the installed
`herdr-issue-worktree` skill. That workflow owns the final native linked worktree, current identifier
and exact title, checkout policy and implementation handoff. Repository instructions and composed
workflows retain branch, eligibility, pull-request and lifecycle decisions.

New agents receive a short private-file reference through native `agent start --` arguments. The
complete request is a JSON string in a mode-0600 file inside a mode-0700 per-launch directory under
the binding directory. Only the base64-encoded JSON path travels through the shell; the agent reads
and parses the full request. References exceeding 768 UTF-8 bytes are refused before launch. This
avoids the long-input truncation reported while Herdr 0.9.3 shells start
([upstream #4950](https://github.com/herdrdev/herdr/issues/4950)). Files remain available for manual
inspection; do not remove a request while its agent may still need it. A timeout can occur after
execution has begun; startup errors trigger inspection, never another launch or prompt.
No trust dialog is answered. The picker reports preparation state, not verified implementation success.

Complete identities live in the Rust companion's `bindings.json`, under the per-socket dispatch
directory. Native metadata carries only bounded display hints: Herdr 0.9.3 normalizes values to
80 characters and discards these tokens on restart. Display-hint failures emit a structured warning
after durable persistence and do not abort startup; persistence failures still stop it. Registration preserves canonical issue identity,
Git paths, title and native handles. Unknown versions, duplicate keys, corrupt state and state
symlinks are refused without replacing the original data. Writes compare the complete prior map
under an exclusive file lock; another pane cannot register the same issue and role. Definite busy/stale
writes retry at most three times after rereading and rechecking ownership, while requiring the target
pane binding to remain unchanged. Unknown write outcomes, changed occupants/owners and exhausted
contention retain state for inspection.

Repeated clicks inspect durable bindings and native state and check Git common directory, linked checkout,
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
Read-only reuse runs before acquiring a creation reservation, so a retained reservation does not
prevent inspection of the existing work. Preparation and registration use the captured native socket
explicitly; an agent tool's ambient environment can belong to another session.
A source-only preflight rejection creates no pane or agent and releases its unused reservation;
refresh the selection before another click. Once a native creation or startup may have occurred,
uncertainty retains the reservation and resources. SCP hostnames use the same lowercase identity
as URL hostnames; repository paths retain their provider-specific comparison.
Closed native panes/workspaces retain their bindings and block another launch for manual inspection.
This conservative policy was explicitly retained: Herdr 0.9.3 can finish a
[queued worktree creation](https://github.com/herdrdev/herdr/blob/v0.9.3/src/app/api/worktrees/deferred.rs#L372)
after its source workspace closes. Missing resources alone do not prove that all work stopped.
Collisions, incompatible repositories, changed occupants and unavailable identity stop the launch.

## Installation and validation

`moon run repository:herdr-plugin` links this plugin alongside worktree cleanup on macOS, using the
existing optional Herdr installation. Bun, authenticated tracker access and the composed skills must
be available to the selected provider. No tracker engine or project catalogue is installed here.

Run the portable owned-behavior tests with:

```sh
moon run herdr-issue:build
bun --config=/dev/null --no-env-file test tooling/herdr-issue-worktree
moon run repository:typescript-lint repository:typescript-typecheck repository:typescript-format-check
moon run herdr-issue:fmt herdr-issue:check herdr-issue:clippy herdr-issue:test herdr-issue:doc
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

On 2026-10-08, a disposable authenticated local tracker returned a synthetic issue and a title longer
than 80 characters. After the user approved source-repository trust, Codex provisioned the native
linked worktree, registered its complete binding, and launched a final Codex agent with the full URL
and verified Git context. Its transcript confirmed directory, top-level and branch before creating
the requested fixture file. Independent rereads confirmed the exact file bytes, title, branch,
common directory, unchanged source HEAD and focus. A second dispatch issued only snapshot reads and
retained the same pane. After restarting the owned test server, native tokens were empty and the
complete binding still identified the same linked checkout without creation or input.

A subsequent audit reproduced nested JSON escape loss through Herdr 0.9.3's
[POSIX quoting path](https://github.com/herdrdev/herdr/blob/v0.9.3/src/platform/macos.rs#L208) under Fish.
The base64 transport was then exercised through a new native Codex launch in a ready Fish pane:
its incoming request matched all 1,018 original bytes, and the final 409-byte fixture matched exactly,
including nested JSON, quotes, backslashes, control escapes and Unicode. Source HEAD and focus stayed
unchanged; data containing shell syntax created no side effect.

The installed runtime did not expose `agent_session` in these Codex observations. The plugin retained
an explicit pending state instead of claiming verified working-session reuse. Verified working reuse
with a native session reference is covered by the owned tests. Earlier native experiments exposed
ambient socket drift; the corrected prompt and registration command pass the intended connection
explicitly, and the corrected disposable scenario stayed in its named session.

Live tracker lifecycle writes and automatic continuation were not exercised. Linux CI covers portable code only;
native Linux, Windows/WSL, remote machines and other Herdr versions were not exercised.
