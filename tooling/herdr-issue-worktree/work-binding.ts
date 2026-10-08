import type { NativePane, NativeSnapshot } from "./native-state.ts";
import { findIssuePane, verifyIssueSession } from "./sessions.ts";
import type { HerdrPort } from "./dispatch.ts";
import type { IssueSelection } from "./selection.ts";
import { inspectRepository } from "./repository.ts";
import { isAbsolute } from "node:path";
import { issueNames } from "./prompts.ts";
import { parseIssueClick } from "./click.ts";
import { z } from "zod";

const registrationSchema = z
  .object({
    provider: z.enum(["claude", "codex"]),
    clickJson: z.string(),
    sourceRoot: z.string().refine(isAbsolute),
    sourceCommonDirectory: z.string().refine(isAbsolute),
    paneId: z.string().min(1),
    issueId: z.string().min(1),
    issueTitle: z.string().min(1),
    branch: z.string().min(1),
  })
  .readonly();

async function registerIssueWork(
  inputJson: string,
  herdr: HerdrPort,
): Promise<void> {
  const input = registrationSchema.parse(JSON.parse(inputJson));
  const selection = registrationSelection(input);
  const { repository } = selection;
  const state = await herdr.snapshot();
  const pane = state.panes.find(({ pane_id }) => pane_id === input.paneId);
  if (pane === undefined) {
    throw new Error("Returned issue pane is unavailable");
  }
  const name = issueNames(selection).worker;
  const tokens = {
    issue_agent: selection.agent,
    issue_name: name,
    issue_repo: repository.commonDirectory,
    issue_role: "work",
    issue_url: selection.click.issue.identity,
    issue_workspace: pane.workspace_id,
    issue_checkout: inspectRepository(pane.cwd ?? "").root,
    issue_id: input.issueId,
    issue_title: input.issueTitle,
    issue_branch: input.branch,
    issue_phase: "starting",
  };
  verifyRegistrationTarget(selection, pane, state);
  const agent = state.agents.find(({ pane_id }) => pane_id === pane.pane_id);
  const reference = agent?.agent_session;
  const updated = {
    ...tokens,
    ...(reference === undefined || reference === null
      ? {}
      : { issue_session: reference.value }),
    issue_phase:
      agent?.agent_status === "working" &&
      reference !== undefined &&
      reference !== null
        ? "sent"
        : "starting",
  };
  verifyIssueSession(selection, pane, {
    ...state,
    bindings: { ...state.bindings, [pane.pane_id]: updated },
  });
  await herdr.mark(pane.pane_id, updated);
}

function verifyRegistrationTarget(
  selection: IssueSelection,
  pane: NativePane,
  state: NativeSnapshot,
): void {
  const previous = state.bindings?.[pane.pane_id];
  const existing = findIssuePane(selection, state);
  if (
    existing !== undefined &&
    existing.pane_id !== pane.pane_id &&
    state.bindings?.[existing.pane_id]?.issue_role === "work"
  ) {
    throw new Error("Another registered work pane already owns this issue");
  }
  if (previous !== undefined) {
    verifyIssueSession(selection, pane, state);
  }
  const agent = state.agents.find(({ pane_id }) => pane_id === pane.pane_id);
  if (
    previous === undefined &&
    (agent !== undefined || (pane.agent !== undefined && pane.agent !== null))
  ) {
    throw new Error("Issue pane has an occupant without a durable binding");
  }
}

function registrationSelection(
  input: z.infer<typeof registrationSchema>,
): IssueSelection {
  const repository = inspectRepository(input.sourceRoot);
  if (repository.commonDirectory !== input.sourceCommonDirectory) {
    throw new Error("Selected source repository changed before registration");
  }
  return {
    agent: input.provider,
    click: parseIssueClick(input.clickJson),
    repository,
  };
}

export { registerIssueWork };
