import { afterEach, expect, test } from "bun:test";
import type { IssueFixture } from "./dispatch-test-support.ts";
import { MemoryHerdr } from "./memory-herdr-test-support.ts";
import type { NativePane } from "./native-state.ts";
import { dispatchIssue } from "./dispatch.ts";
import { issueFixture } from "./dispatch-test-support.ts";
import { registerIssueWork } from "./register-work.ts";
import { workingIssueState } from "./session-test-support.ts";

const fixtures: IssueFixture[] = [];
const longTitleLength = 150;
afterEach(() => {
  for (const fixture of fixtures.splice(0)) {
    fixture.dispose();
  }
});
function registration(fixture: IssueFixture, title = "Fixture issue"): string {
  return JSON.stringify({
    provider: "codex",
    clickJson: JSON.stringify(fixture.selection.click.context),
    sourceRoot: fixture.selection.repository.root,
    sourceCommonDirectory: fixture.selection.repository.commonDirectory,
    paneId: "w2:p1",
    issueId: "GH-17",
    issueTitle: title,
    branch: "issue-work",
  });
}

test("registers a full final title and native session after startup, then reuses without input", async () => {
  const fixture = issueFixture();
  fixtures.push(fixture);
  const title = ` ${"é".repeat(longTitleLength)}\n\u007F `;
  const state = workingIssueState(fixture);
  const herdr = new MemoryHerdr(fixture, {
    ...state,
    bindings: {},
    agents: [],
    panes: state.panes.map((pane) => ({ ...pane, agent: null, tokens: {} })),
    workspaces: state.workspaces.map((workspace) => ({
      ...workspace,
      label: `GH-17 — ${title}`,
    })),
  });
  await registerIssueWork(registration(fixture, title), herdr);
  herdr.state = {
    ...herdr.state,
    panes: herdr.state.panes.map((pane) => ({
      ...pane,
      agent: "codex",
      tokens: {},
    })),
    agents: state.agents,
  };
  await registerIssueWork(registration(fixture, title), herdr);
  expect(await dispatchIssue(fixture.selection, herdr)).toMatchObject({
    kind: "reused",
    paneId: "w2:p1",
  });
  expect(herdr.state.bindings?.["w2:p1"]?.issue_title).toBe(title);
  expect(herdr.initialPrompts).toEqual([]);
});

test("refuses an occupied worktree without any binding write", async () => {
  const fixture = issueFixture();
  fixtures.push(fixture);
  const herdr = new MemoryHerdr(fixture, {
    ...workingIssueState(fixture),
    bindings: {},
  });
  const failure = await registerIssueWork(registration(fixture), herdr).catch(
    (error: unknown) => error,
  );
  expect(failure).toMatchObject({
    message: "Issue pane has an occupant without a durable binding",
  });
  expect(herdr.state.bindings).toEqual({});
  expect(herdr.initialPrompts).toEqual([]);
});

test("refuses a mismatched label before registration", async () => {
  const fixture = issueFixture();
  fixtures.push(fixture);
  const state = workingIssueState(fixture);
  const herdr = new MemoryHerdr(fixture, {
    ...state,
    bindings: {},
    agents: [],
    panes: state.panes.map(unoccupiedPane),
  });
  const failure = await registerIssueWork(
    registration(fixture, "Different"),
    herdr,
  ).catch((error: unknown) => error);
  expect(failure).toMatchObject({
    message: "Issue worktree label or branch changed; inspect before resuming",
  });
  expect(herdr.state.bindings).toEqual({});
});

function unoccupiedPane(pane: NativePane): NativePane {
  return { ...pane, agent: null };
}
