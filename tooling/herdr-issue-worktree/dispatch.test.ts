import { afterEach, expect, test } from "bun:test";
import type { IssueFixture } from "./dispatch-test-support.ts";
import { MemoryHerdr } from "./memory-herdr-test-support.ts";
import { dispatchIssue } from "./dispatch.ts";
import { issueFixture } from "./dispatch-test-support.ts";
import { workingIssueState } from "./session-test-support.ts";

const fixtures: IssueFixture[] = [];
const sourceAndPreparationPaneCount = 2;

afterEach(() => {
  for (const fixture of fixtures.splice(0)) {
    fixture.dispose();
  }
});

test("does not send a prompt to an issue agent already working", async () => {
  const fixture = issueFixture();
  fixtures.push(fixture);
  const herdr = new MemoryHerdr(fixture, workingIssueState(fixture));
  const outcome = await dispatchIssue(fixture.selection, herdr);

  expect(outcome).toMatchObject({
    kind: "reused",
    paneId: "w2:p1",
    status: "working",
  });
  expect(herdr.initialPrompts).toEqual([]);
});

test("retains an idle existing issue agent without a non-atomic resubmission", async () => {
  const fixture = issueFixture();
  fixtures.push(fixture);
  const state = workingIssueState(fixture);
  const [agent] = state.agents;
  if (agent === undefined) {
    throw new Error("Fixture agent missing");
  }
  const herdr = new MemoryHerdr(fixture, {
    ...state,
    agents: [{ ...agent, agent_status: "idle" }],
  });
  expect(await dispatchIssue(fixture.selection, herdr)).toMatchObject({
    kind: "pending",
    paneId: "w2:p1",
  });
  expect(herdr.initialPrompts).toEqual([]);
});

test("starts preparation in a returned native pane with the full authorized context", async () => {
  const fixture = issueFixture();
  fixtures.push(fixture);
  const herdr = new MemoryHerdr(fixture);

  const outcome = await dispatchIssue(fixture.selection, herdr);

  expect(outcome).toMatchObject({ kind: "started", paneId: "preparation:p1" });
  expect(herdr.initialPrompts).toHaveLength(1);
  expect(herdr.initialPrompts[0]).toContain("herdr-issue-worktree");
  expect(herdr.initialPrompts[0]).toContain(fixture.selection.click.issue.url);
  expect(herdr.initialPrompts[0]).toContain(
    fixture.selection.repository.commonDirectory,
  );
  expect(herdr.state.focused_pane_id).toBe("source:p1");
});

test("keeps a trust-blocked preparation without restarting or sending interactive input", async () => {
  const fixture = issueFixture();
  fixtures.push(fixture);
  const herdr = new MemoryHerdr(fixture);
  herdr.startupBlocked = true;

  expect(await dispatchIssue(fixture.selection, herdr)).toMatchObject({
    kind: "blocked",
    paneId: "preparation:p1",
  });
  expect(herdr.initialPrompts).toHaveLength(1);
  expect(await dispatchIssue(fixture.selection, herdr)).toMatchObject({
    kind: "blocked",
    paneId: "preparation:p1",
  });
  expect(herdr.state.panes).toHaveLength(sourceAndPreparationPaneCount);
  expect(herdr.initialPrompts).toHaveLength(1);
});

test("does not resend an uncertain prompt on another click", async () => {
  const fixture = issueFixture();
  fixtures.push(fixture);
  const herdr = new MemoryHerdr(fixture);
  herdr.startupUncertain = true;

  expect(await dispatchIssue(fixture.selection, herdr)).toMatchObject({
    kind: "uncertain",
    paneId: "preparation:p1",
  });
  expect(await dispatchIssue(fixture.selection, herdr)).toMatchObject({
    kind: "reused",
    paneId: "preparation:p1",
  });
  expect(herdr.initialPrompts).toHaveLength(1);
  expect(herdr.state.panes).toHaveLength(sourceAndPreparationPaneCount);
});

test("reports a focus discrepancy instead of launching or restoring user focus", async () => {
  const fixture = issueFixture();
  fixtures.push(fixture);
  const herdr = new MemoryHerdr(fixture);
  herdr.focusChanged = true;

  expect(await dispatchIssue(fixture.selection, herdr)).toMatchObject({
    kind: "uncertain",
    paneId: "preparation:p1",
  });
  expect(herdr.initialPrompts).toEqual([]);
  expect(herdr.state.agents).toEqual([]);
  expect(herdr.state.focused_pane_id).toBe("preparation:p1");
});

test("reports trust blocking even before a native session reference is available", async () => {
  const fixture = issueFixture();
  fixtures.push(fixture);
  const herdr = new MemoryHerdr(fixture);
  herdr.startupBlocked = true;
  herdr.sessionAvailable = false;

  expect(await dispatchIssue(fixture.selection, herdr)).toMatchObject({
    kind: "blocked",
    paneId: "preparation:p1",
  });
  expect(herdr.initialPrompts).toHaveLength(1);
});

test("retains startup after trust is resolved without resending its initial task", async () => {
  const fixture = issueFixture();
  fixtures.push(fixture);
  const herdr = new MemoryHerdr(fixture);
  herdr.startupBlocked = true;
  herdr.sessionAvailable = false;
  await dispatchIssue(fixture.selection, herdr);
  herdr.state = {
    ...herdr.state,
    agents: herdr.state.agents.map((agent) => ({
      ...agent,
      agent_session: { value: "trusted-session" },
      agent_status: "idle",
      interactive_ready: true,
    })),
  };

  expect(await dispatchIssue(fixture.selection, herdr)).toMatchObject({
    kind: "pending",
    paneId: "preparation:p1",
  });
  expect(herdr.initialPrompts).toHaveLength(1);
  expect(herdr.state.panes).toHaveLength(sourceAndPreparationPaneCount);
});

for (const failure of [
  "creation",
  "metadata-before",
  "metadata-after",
  "snapshot-after-creation",
] as const) {
  test(`retains partial preparation and inspects before another click after ${failure}`, async () => {
    const fixture = issueFixture();
    fixtures.push(fixture);
    const herdr = new MemoryHerdr(fixture);
    herdr.failure = failure;
    expect(await dispatchIssue(fixture.selection, herdr)).toMatchObject({
      kind: "uncertain",
    });
    expect(herdr.state.panes).toHaveLength(sourceAndPreparationPaneCount);
    expect(herdr.snapshotReads).toBeGreaterThan(1);
    herdr.failure = null;
    const resumed = await dispatchIssue(fixture.selection, herdr);
    expect(["pending", "uncertain"]).toContain(resumed.kind);
    expect(herdr.state.panes).toHaveLength(sourceAndPreparationPaneCount);
    expect(herdr.state.agents).toEqual([]);
    expect(herdr.initialPrompts).toEqual([]);
  });
}
