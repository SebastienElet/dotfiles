import { afterEach, expect, test } from "bun:test";
import {
  mainCheckoutState,
  workingIssueState,
} from "./session-test-support.ts";
import type { IssueFixture } from "./dispatch-test-support.ts";
import { issueFixture } from "./dispatch-test-support.ts";
import { verifyIssueSession } from "./sessions.ts";

const fixtures: IssueFixture[] = [];

afterEach(() => {
  for (const fixture of fixtures.splice(0)) {
    fixture.dispose();
  }
});

test("rejects the main checkout even when native metadata claims a linked worktree", () => {
  const fixture = issueFixture();
  fixtures.push(fixture);
  const state = mainCheckoutState(fixture);
  const [pane] = state.panes;
  if (pane === undefined) {
    throw new Error("Fixture pane missing");
  }
  expect(() => verifyIssueSession(fixture.selection, pane, state)).toThrow(
    "Git checkout is not a linked worktree",
  );
});

test("rejects disagreement between the pane occupant and the named agent", () => {
  const fixture = issueFixture();
  fixtures.push(fixture);
  const state = workingIssueState(fixture);
  const [pane] = state.panes;
  if (pane === undefined) {
    throw new Error("Fixture pane missing");
  }
  expect(() =>
    verifyIssueSession(fixture.selection, { ...pane, agent: "claude" }, state),
  ).toThrow("Pane occupant disagrees");
});

test("rejects a same-provider replacement retaining the native name", () => {
  const fixture = issueFixture();
  fixtures.push(fixture);
  const initial = workingIssueState(fixture);
  const [agent] = initial.agents;
  if (agent === undefined) {
    throw new Error("Fixture agent missing");
  }
  const state = {
    ...initial,
    agents: [{ ...agent, agent_session: { value: "replacement" } }],
  };
  const [pane] = state.panes;
  if (pane === undefined) {
    throw new Error("Fixture pane missing");
  }
  expect(() => verifyIssueSession(fixture.selection, pane, state)).toThrow(
    "session identity is unavailable or changed",
  );
});
