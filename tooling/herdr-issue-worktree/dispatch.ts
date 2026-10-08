import { findIssuePane, verifyIssueSession } from "./sessions.ts";
import { issueNames, preparationPrompt } from "./prompts.ts";
import type { IssueSelection } from "./selection.ts";
import type { IssueSession } from "./sessions.ts";
import type { NativeEnvironment } from "./herdr-command.ts";
import type { NativeSnapshot } from "./native-state.ts";
import { inspectRepository } from "./repository.ts";

type HerdrPort = Readonly<{
  bindingDirectory: string;
  nativeEnvironment: NativeEnvironment;
  createPreparation: (
    selection: IssueSelection,
    label: string,
  ) => Promise<string>;
  mark: (
    paneId: string,
    tokens: Readonly<Record<string, string>>,
  ) => Promise<void>;
  snapshot: () => Promise<NativeSnapshot>;
  start: (launch: AgentLaunch) => Promise<void>;
}>;

type AgentLaunch = Readonly<{
  paneId: string;
  name: string;
  kind: IssueSelection["agent"];
  prompt: string;
}>;

type DispatchOutcome = Readonly<{
  kind: "reused" | "started" | "blocked" | "uncertain" | "pending" | "rejected";
  paneId?: string;
  reason?: string;
  status?: string;
}>;

async function dispatchIssue(
  selection: IssueSelection,
  herdr: HerdrPort,
): Promise<DispatchOutcome> {
  const [initial] = await Promise.allSettled([
    Promise.resolve().then(() => herdr.snapshot()),
  ]);
  if (initial?.status !== "fulfilled") {
    return {
      kind: "rejected",
      reason: `Native inspection failed before creation; nothing launched. ${initial?.status === "rejected" ? errorMessage(initial.reason) : "Snapshot unavailable"}`,
    };
  }
  const state = initial.value;
  try {
    const pane = findIssuePane(selection, state);
    return pane === undefined
      ? await createPreparation(selection, herdr, state)
      : observeExisting(verifyIssueSession(selection, pane, state));
  } catch (error) {
    const [inspection] = await Promise.allSettled([herdr.snapshot()]);
    return {
      kind: "uncertain",
      reason: `${errorMessage(error)}. ${inspection?.status === "fulfilled" ? "Native state reread; existing resources retained" : "Native inspection unavailable; do not retry blindly"}`,
    };
  }
}

function observeExisting(session: IssueSession): DispatchOutcome {
  const { agent, pane, tokens } = session;
  if (tokens.issue_phase === "submitting") {
    return {
      kind: "uncertain",
      paneId: pane.pane_id,
      reason:
        "Previous delivery is uncertain; inspect the retained agent before retrying",
    };
  }
  if (tokens.issue_session !== undefined && agent?.agent_status === "working") {
    return { kind: "reused", paneId: pane.pane_id, status: "working" };
  }
  if (agent?.agent_status === "blocked") {
    return {
      kind: "blocked",
      paneId: pane.pane_id,
      reason:
        "Existing agent is waiting for user input or trust; no interactive input sent",
    };
  }
  return {
    kind: "pending",
    paneId: pane.pane_id,
    status: agent?.agent_status ?? "unavailable",
    reason:
      "Existing pane retained. Inspect it and continue there: Herdr 0.9.3 cannot atomically guard continuation by session identity and availability; no prompt resent",
  };
}

async function inspectExistingIssue(
  selection: IssueSelection,
  herdr: HerdrPort,
): Promise<DispatchOutcome | undefined> {
  const state = await herdr.snapshot();
  const pane = findIssuePane(selection, state);
  return pane === undefined
    ? undefined
    : observeExisting(verifyIssueSession(selection, pane, state));
}

async function createPreparation(
  selection: IssueSelection,
  herdr: HerdrPort,
  before: NativeSnapshot,
): Promise<DispatchOutcome> {
  const names = issueNames(selection);
  if (preparationExists(before, names)) {
    return {
      kind: "uncertain",
      reason:
        "Existing preparation or agent has incomplete metadata; inspect before creating anything",
    };
  }
  const failure = sourcePreflightFailure(selection);
  if (failure !== undefined) {
    return { kind: "rejected", reason: failure };
  }
  const paneId = await herdr.createPreparation(selection, names.label);
  const after = await herdr.snapshot();
  if (focusKey(before) !== focusKey(after)) {
    return {
      kind: "uncertain",
      paneId,
      reason:
        "Focus changed during creation; no agent launched and no compensating focus command sent",
    };
  }
  assertAvailablePreparation(selection, after, paneId);
  await herdr.mark(paneId, {
    issue_agent: selection.agent,
    issue_checkout: selection.repository.root,
    issue_name: names.preparation,
    issue_phase: "starting",
    issue_repo: selection.repository.commonDirectory,
    issue_role: "preparation",
    issue_url: selection.click.issue.identity,
    issue_workspace:
      after.panes.find(({ pane_id }) => pane_id === paneId)?.workspace_id ?? "",
  });
  return startPreparation(selection, herdr, paneId);
}

async function startPreparation(
  selection: IssueSelection,
  herdr: HerdrPort,
  paneId: string,
): Promise<DispatchOutcome> {
  const name = issueNames(selection).preparation;
  let startupFailure: unknown = undefined;
  try {
    await herdr.start({
      paneId,
      name,
      kind: selection.agent,
      prompt: preparationPrompt(selection, herdr),
    });
  } catch (error) {
    startupFailure = error;
  }
  const state = await herdr.snapshot();
  const agent = state.agents.find(
    (candidate) =>
      candidate.name === name &&
      candidate.pane_id === paneId &&
      candidate.agent === selection.agent,
  );
  if (agent?.agent_session === undefined || agent.agent_session === null) {
    return {
      kind: agent?.agent_status === "blocked" ? "blocked" : "uncertain",
      paneId,
      reason: `Startup identity unavailable; inspect before retrying. ${errorMessage(startupFailure)}`,
    };
  }
  await herdr.mark(paneId, { issue_session: agent.agent_session.value });
  return observeStartup(selection, herdr, { paneId, startupFailure });
}

async function observeStartup(
  selection: IssueSelection,
  herdr: HerdrPort,
  launch: Readonly<{ paneId: string; startupFailure: unknown }>,
): Promise<DispatchOutcome> {
  const { paneId, startupFailure } = launch;
  const updated = await herdr.snapshot();
  const pane = updated.panes.find((candidate) => candidate.pane_id === paneId);
  if (pane === undefined) {
    throw new Error("Preparation pane disappeared during startup");
  }
  const session = verifyIssueSession(selection, pane, updated);
  if (session.agent?.agent_status === "blocked") {
    return {
      kind: "blocked",
      paneId,
      reason:
        "Startup is blocked; initial prompt may be retained by the provider. Resolve trust in that pane; no retry or additional input sent",
    };
  }
  if (
    startupFailure !== undefined ||
    session.agent?.agent_status !== "working"
  ) {
    return {
      kind: "uncertain",
      paneId,
      status: session.agent?.agent_status ?? "unavailable",
      reason: `${errorMessage(startupFailure)}. Initial prompt may already be executing; inspect this agent, never resend blindly`,
    };
  }
  await herdr.mark(paneId, { issue_phase: "sent" });
  return { kind: "started", paneId, status: "preparation-working" };
}

function preparationExists(
  state: NativeSnapshot,
  names: ReturnType<typeof issueNames>,
): boolean {
  return (
    state.tabs.some(({ label }) => label === names.label) ||
    state.workspaces.some(({ label }) => label === names.label) ||
    state.agents.some(
      ({ name }) => name === names.preparation || name === names.worker,
    )
  );
}

function sourcePreflightFailure(selection: IssueSelection): string | undefined {
  try {
    const source = inspectRepository(selection.repository.root);
    if (
      source.commonDirectory !== selection.repository.commonDirectory ||
      source.branch !== selection.repository.branch ||
      source.head !== selection.repository.head
    ) {
      return "Selected Git context changed before preparation";
    }
  } catch (error) {
    return errorMessage(error);
  }
  return undefined;
}

function assertAvailablePreparation(
  selection: IssueSelection,
  state: NativeSnapshot,
  paneId: string,
): void {
  const pane = state.panes.find((candidate) => candidate.pane_id === paneId);
  if (
    pane?.cwd !== selection.repository.root ||
    (pane.agent !== undefined && pane.agent !== null)
  ) {
    throw new Error(
      "Returned preparation pane is unavailable or in the wrong directory",
    );
  }
}

function focusKey(snapshot: NativeSnapshot): string {
  return JSON.stringify([
    snapshot.focused_workspace_id,
    snapshot.focused_tab_id,
    snapshot.focused_pane_id,
  ]);
}

function errorMessage(error: unknown): string {
  return error instanceof Error
    ? error.message
    : "Native startup activity is not confirmed";
}

export { dispatchIssue, inspectExistingIssue };
export type { AgentLaunch, DispatchOutcome, HerdrPort };
