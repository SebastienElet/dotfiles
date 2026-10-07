import { z } from "zod";

const tokensSchema = z.record(z.string(), z.string()).readonly();
const agentSchema = z
  .object({
    agent: z.string().nullish(),
    agent_session: z
      .object({ value: z.string().min(1) })
      .readonly()
      .nullish(),
    agent_status: z.enum(["idle", "working", "blocked", "done", "unknown"]),
    foreground_cwd: z.string().nullish(),
    interactive_ready: z.boolean().optional(),
    name: z.string().nullish(),
    pane_id: z.string().min(1),
  })
  .readonly();
const paneSchema = z
  .object({
    agent: z.string().nullish(),
    cwd: z.string().nullish(),
    pane_id: z.string().min(1),
    tokens: tokensSchema.optional(),
    workspace_id: z.string().min(1),
  })
  .readonly();
const workspaceSchema = z
  .object({
    label: z.string(),
    workspace_id: z.string().min(1),
    worktree: z
      .object({
        checkout_path: z.string().min(1),
        is_linked_worktree: z.boolean(),
        repo_root: z.string().min(1),
      })
      .readonly()
      .nullish(),
  })
  .readonly();
const snapshotSchema = z
  .object({
    agents: z.array(agentSchema).readonly(),
    focused_pane_id: z.string().nullish(),
    focused_tab_id: z.string().nullish(),
    focused_workspace_id: z.string().nullish(),
    panes: z.array(paneSchema).readonly(),
    tabs: z
      .array(
        z
          .object({
            label: z.string(),
            tab_id: z.string(),
            workspace_id: z.string(),
          })
          .readonly(),
      )
      .readonly(),
    version: z.string(),
    workspaces: z.array(workspaceSchema).readonly(),
  })
  .readonly();

type NativeAgent = z.infer<typeof agentSchema>;
type NativePane = z.infer<typeof paneSchema>;
type NativeSnapshot = z.infer<typeof snapshotSchema>;

export { agentSchema, paneSchema, snapshotSchema, workspaceSchema };
export type { NativeAgent, NativePane, NativeSnapshot };
