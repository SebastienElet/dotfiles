# Herdr operations

## Capability discovery

Verified against CLI and running server 0.9.5, private protocol 22, on macOS on 2026-11-03.
The installed `herdr` skill owns terminal interaction details. Recheck `HERDR_ENV=1`,
`herdr --help`, `herdr status` and relevant command groups on the actual session before control.
No update or integration installation is implied. IDs/names are scoped to the selected machine
and session; retain that scope with every association.

Official sources for this version:

- [Automation](https://raw.githubusercontent.com/herdrdev/herdr/v0.9.5/docs/next/website/src/content/docs/agent-automation.mdx)
- [CLI reference](https://raw.githubusercontent.com/herdrdev/herdr/v0.9.5/docs/next/website/src/content/docs/cli-reference.mdx)
- [Documentation index](https://herdr.dev/llms.txt) for other installed versions.

## Operations to compose

| Purpose                 | Existing interface                                                                                                                                                 |
| ----------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Inventory               | `herdr workspace list`, `herdr worktree list --cwd <repository>`, `herdr agent list`, `herdr tab list --workspace <id>`                                            |
| Isolated task layout    | `herdr worktree create --cwd <repository> --branch <branch> --base <ref> --label <unit> --no-focus`; reuse through `worktree open` after verifying association     |
| Coordinator or task tab | `herdr tab create --workspace <id> --cwd <path> --label <label> --no-focus`                                                                                        |
| Executor startup        | `herdr agent start <unique-name> --kind claude\|codex --pane <returned-id>`; native model/effort args only after `--`, checked against that agent's installed help |
| Communicate             | `herdr agent prompt <name> <text> --wait --timeout 60000`                                                                                                          |
| Observe                 | `herdr agent get <name>`, `herdr agent read <name> --source recent-unwrapped --lines 80`, `herdr agent wait <name> --timeout 60000`                                |
| Dialog interaction      | `herdr agent send-keys <name> <keys...>` after inspecting UI and authority                                                                                         |
| Selective cleanup       | `herdr worktree remove --workspace <owned-task-workspace>`; `herdr tab close <owned-tab>` only if still present                                                    |

Command notation is illustrative: substitute verified IDs and quote text/paths safely.
Read creation results, never derive IDs from labels or sidebar position. `agent start` requires
an existing foreground shell; it creates no layout. `agent prompt` rejects a currently blocked
agent. Read the dialog before deliberate keys; do not resend a prompt blindly after an error.

## Actual limits

- Waits track lifecycle, not a particular response or issue. They default to idle/done/blocked;
  an already matching standalone wait returns immediately. Use short bounded waits across the
  batch and inspect output. Neither `unknown` nor a timeout proves success or non-delivery.
- Recent reads can be partial. Large alternate-screen history reads may require idle; use visible
  output for active agents. If necessary, request a temporary Markdown result and read its path
  on the same machine. Do not make every executor write a competing report initially.
- A live agent alias disappears on exit/replacement. Resumption must reconcile pane occupant and
  worktree; saved names alone cannot establish identity or prove an agent should be restarted.
- A persistent Herdr session keeps terminals available while its server runs; this does not make
  an ended coordinator turn wake itself. No watchdog or automatic coordinator resumption is used.
- `worktree remove` and closes mutate real resources. Check current installed help and ownership;
  do not infer safe deletion from a completion badge. Do not stop Herdr's server, use `--force`,
  close a workspace group or assume container cleanup or deployed-link repair is provided.
- Container ownership and dirty/unpublished Git work require separate inspection with existing
  project/container/Git tooling. Herdr is not an isolation boundary for shared databases or data.
