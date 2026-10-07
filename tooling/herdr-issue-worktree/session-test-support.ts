import type { IssueFixture } from "./dispatch-test-support.ts";
import type { NativeSnapshot } from "./native-state.ts";

function workingIssueState(fixture: IssueFixture): NativeSnapshot {
  const { selection, checkout } = fixture;
  return {
    agents: [
      {
        agent: "codex",
        agent_status: "working",
        foreground_cwd: checkout,
        agent_session: { value: "session-17" },
        name: "worker",
        pane_id: "w2:p1",
        interactive_ready: true,
      },
    ],
    panes: [
      {
        agent: "codex",
        cwd: checkout,
        pane_id: "w2:p1",
        workspace_id: "w2",
        tokens: {
          issue_url: selection.click.issue.identity,
          issue_repo: selection.repository.commonDirectory,
          issue_role: "work",
          issue_name: "worker",
          issue_agent: "codex",
          issue_session: "session-17",
          issue_phase: "sent",
          issue_branch: "issue-work",
          issue_id: "GH-17",
          issue_title: "Fixture issue",
        },
      },
    ],
    tabs: [],
    version: "0.9.3",
    workspaces: [
      {
        workspace_id: "w2",
        label: "GH-17 — Fixture issue",
        worktree: {
          checkout_path: checkout,
          is_linked_worktree: true,
          repo_root: selection.repository.root,
        },
      },
    ],
  };
}

function mainCheckoutState(fixture: IssueFixture): NativeSnapshot {
  const state = workingIssueState(fixture);
  const [agent] = state.agents;
  const [pane] = state.panes;
  const [workspace] = state.workspaces;
  if (agent === undefined || pane === undefined || workspace === undefined) {
    throw new Error("Fixture state incomplete");
  }
  const { root } = fixture.selection.repository;
  return {
    ...state,
    agents: [{ ...agent, foreground_cwd: root }],
    panes: [
      { ...pane, cwd: root, tokens: { ...pane.tokens, issue_branch: "main" } },
    ],
    workspaces: [
      {
        ...workspace,
        worktree: {
          checkout_path: root,
          is_linked_worktree: true,
          repo_root: root,
        },
      },
    ],
  };
}

export { mainCheckoutState, workingIssueState };
