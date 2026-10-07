import type {
  NativeAgent,
  NativePane,
  NativeSnapshot,
} from "./native-state.ts";
import type { IssueSelection } from "./selection.ts";
import type { RepositoryContext } from "./repository.ts";
import { inspectRepository } from "./repository.ts";
import { isAbsolute } from "node:path";
import { issueNames } from "./prompts.ts";
import { realpathSync } from "node:fs";
import { z } from "zod";

const sessionTokensSchema = z
  .object({
    issue_agent: z.enum(["claude", "codex"]),
    issue_branch: z.string().min(1).optional(),
    issue_id: z.string().min(1).optional(),
    issue_name: z.string().min(1),
    issue_phase: z.enum(["pending", "starting", "submitting", "sent"]),
    issue_repo: z.string().refine(isAbsolute),
    issue_role: z.enum(["preparation", "work"]),
    issue_session: z.string().min(1).optional(),
    issue_title: z.string().min(1).optional(),
    issue_url: z.url(),
  })
  .readonly();

type IssueSession = Readonly<{
  agent: NativeAgent | undefined;
  checkout: RepositoryContext;
  pane: NativePane;
  tokens: z.infer<typeof sessionTokensSchema>;
}>;

function findIssuePane(
  selection: IssueSelection,
  state: NativeSnapshot,
): NativePane | undefined {
  const matches = state.panes.filter(
    ({ tokens }) =>
      tokens?.issue_url === selection.click.issue.identity &&
      tokens.issue_repo === selection.repository.commonDirectory,
  );
  const work = matches.filter(({ tokens }) => tokens?.issue_role === "work");
  const candidates = work.length === 0 ? matches : work;
  if (candidates.length > 1) {
    throw new Error(
      "Several panes claim this issue; inspect the collision before resuming",
    );
  }
  return candidates[0];
}

function verifyIssueSession(
  selection: IssueSelection,
  pane: NativePane,
  state: NativeSnapshot,
): IssueSession {
  const tokens = sessionTokensSchema.parse(pane.tokens);
  if (tokens.issue_agent !== selection.agent) {
    throw new Error(
      "Existing issue session uses another provider; it will not be replaced",
    );
  }
  const checkout = verifyCheckout(selection, state, { pane, tokens });
  const agent = state.agents.find(
    (candidate) => candidate.pane_id === pane.pane_id,
  );
  if (agent === undefined) {
    if (pane.agent !== undefined && pane.agent !== null) {
      throw new Error("Pane occupant is not a verified issue agent");
    }
    return { agent, checkout, pane, tokens };
  }
  verifyAgent(selection, agent, { pane, tokens });
  return { agent, checkout, pane, tokens };
}

function verifyAgent(
  selection: IssueSelection,
  agent: NativeAgent,
  candidate: Readonly<Pick<IssueSession, "pane" | "tokens">>,
): void {
  const { pane, tokens } = candidate;
  if (agent.name !== tokens.issue_name || agent.agent !== tokens.issue_agent) {
    throw new Error(
      "Issue pane has another occupant; it will not receive input",
    );
  }
  if (pane.agent !== agent.agent) {
    throw new Error("Pane occupant disagrees with the named issue agent");
  }
  if (
    agent.foreground_cwd === undefined ||
    agent.foreground_cwd === null ||
    realpathSync(agent.foreground_cwd) !== realpathSync(pane.cwd ?? "")
  ) {
    throw new Error("Issue agent working directory is not verified");
  }
  if (
    tokens.issue_session === undefined &&
    tokens.issue_phase === "starting" &&
    tokens.issue_role === "preparation" &&
    agent.name === issueNames(selection).preparation
  ) {
    return;
  }
  if (
    tokens.issue_session === undefined ||
    agent.agent_session?.value !== tokens.issue_session
  ) {
    throw new Error("Issue agent session identity is unavailable or changed");
  }
}

function verifyCheckout(
  selection: IssueSelection,
  state: NativeSnapshot,
  candidate: Readonly<{
    pane: NativePane;
    tokens: z.infer<typeof sessionTokensSchema>;
  }>,
): RepositoryContext {
  const { pane, tokens } = candidate;
  const checkout = inspectRepository(pane.cwd ?? "");
  if (checkout.commonDirectory !== selection.repository.commonDirectory) {
    throw new Error("Issue pane belongs to another Git repository");
  }
  if (tokens.issue_role === "preparation") {
    if (checkout.root !== selection.repository.root) {
      throw new Error("Preparation pane moved outside the selected checkout");
    }
    return checkout;
  }
  if (checkout.gitDirectory === checkout.commonDirectory) {
    throw new Error("Git checkout is not a linked worktree");
  }
  const workspace = state.workspaces.find(
    (workspaceEntry) => workspaceEntry.workspace_id === pane.workspace_id,
  );
  if (
    workspace?.worktree?.is_linked_worktree !== true ||
    realpathSync(workspace.worktree.checkout_path) !== checkout.root
  ) {
    throw new Error(
      "Issue workspace is not the verified native linked worktree",
    );
  }
  if (
    tokens.issue_branch !== checkout.branch ||
    tokens.issue_id === undefined ||
    tokens.issue_title === undefined ||
    workspace.label !== `${tokens.issue_id} — ${tokens.issue_title}`
  ) {
    throw new Error(
      "Issue worktree label or branch changed; inspect before resuming",
    );
  }
  return checkout;
}

export { findIssuePane, verifyIssueSession };
export type { IssueSession };
