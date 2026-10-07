import type { AgentLaunch, HerdrPort } from "./dispatch.ts";
import { HerdrFailureError } from "./herdr-command.ts";
import type { IssueFixture } from "./dispatch-test-support.ts";
import type { IssueSelection } from "./selection.ts";
import type { NativeSnapshot } from "./native-state.ts";

class MemoryHerdr implements HerdrPort {
  public state: NativeSnapshot;
  public readonly initialPrompts: string[] = [];
  public startupBlocked = false;
  public sessionAvailable = true;
  public startupUncertain = false;
  public focusChanged = false;
  public snapshotReads = 0;
  public failure:
    | "creation"
    | "metadata-before"
    | "metadata-after"
    | "snapshot-after-creation"
    | null = null;

  public constructor(
    public readonly fixture: IssueFixture,
    state?: NativeSnapshot,
  ) {
    this.state = state ?? {
      agents: [],
      focused_pane_id: "source:p1",
      focused_tab_id: "source:t1",
      focused_workspace_id: "source",
      panes: [
        {
          cwd: fixture.selection.repository.root,
          pane_id: "source:p1",
          workspace_id: "source",
        },
      ],
      tabs: [{ label: "source", tab_id: "source:t1", workspace_id: "source" }],
      version: "0.9.3",
      workspaces: [{ label: "source", workspace_id: "source" }],
    };
  }

  public snapshot(): Promise<NativeSnapshot> {
    this.snapshotReads += 1;
    if (
      this.failure === "snapshot-after-creation" &&
      this.state.panes.some(({ pane_id }) => pane_id === "preparation:p1")
    ) {
      return Promise.reject(
        new Error("Native snapshot unavailable after creation"),
      );
    }
    return Promise.resolve(this.state);
  }

  public createPreparation(
    selection: IssueSelection,
    label: string,
  ): Promise<string> {
    this.state = {
      ...this.state,
      focused_pane_id: this.focusChanged
        ? "preparation:p1"
        : this.state.focused_pane_id,
      panes: [
        ...this.state.panes,
        {
          cwd: selection.repository.root,
          pane_id: "preparation:p1",
          workspace_id: "source",
        },
      ],
      tabs: [
        ...this.state.tabs,
        { label, tab_id: "preparation:t1", workspace_id: "source" },
      ],
    };
    return this.failure === "creation"
      ? Promise.reject(
          new Error("Creation response unavailable after mutation"),
        )
      : Promise.resolve("preparation:p1");
  }

  public mark(
    paneId: string,
    tokens: Readonly<Record<string, string>>,
  ): Promise<void> {
    if (this.failure === "metadata-before") {
      return Promise.reject(
        new Error("Metadata reporting failed before applying tokens"),
      );
    }
    this.state = {
      ...this.state,
      panes: this.state.panes.map((pane) =>
        pane.pane_id === paneId
          ? { ...pane, tokens: { ...pane.tokens, ...tokens } }
          : pane,
      ),
    };
    return this.failure === "metadata-after"
      ? Promise.reject(
          new Error("Metadata response unavailable after mutation"),
        )
      : Promise.resolve();
  }

  public start({ paneId, name, kind, prompt }: AgentLaunch): Promise<void> {
    this.initialPrompts.push(prompt);
    this.state = {
      ...this.state,
      agents: [
        ...this.state.agents,
        {
          agent: kind,
          agent_session: this.sessionAvailable
            ? { value: "preparation-session" }
            : null,
          agent_status: this.startupBlocked ? "blocked" : "working",
          foreground_cwd: this.fixture.selection.repository.root,
          interactive_ready: !this.startupBlocked,
          name,
          pane_id: paneId,
        },
      ],
      panes: this.state.panes.map((pane) =>
        pane.pane_id === paneId ? { ...pane, agent: kind } : pane,
      ),
    };
    if (this.startupBlocked) {
      return Promise.reject(
        new HerdrFailureError(
          "agent_not_ready",
          "Trust UI is blocking startup",
        ),
      );
    }
    if (this.startupUncertain) {
      return Promise.reject(
        new HerdrFailureError("timeout", "Startup readiness was not observed"),
      );
    }
    return Promise.resolve();
  }
}

export { MemoryHerdr };
