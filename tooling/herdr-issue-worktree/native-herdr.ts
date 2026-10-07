import { paneSchema, snapshotSchema } from "./native-state.ts";
import type { HerdrPort } from "./dispatch.ts";
import type { IssueSelection } from "./selection.ts";
import { realpathSync } from "node:fs";
import { z } from "zod";

type HerdrCommand = (
  arguments_: readonly string[],
  options?: Readonly<{ output: "silent" }>,
) => Promise<unknown>;

const worktreeListSchema = z.object({
  source: z.object({
    repo_key: z.string().min(1),
    source_checkout_path: z.string().min(1),
    source_workspace_id: z.string().nullish(),
  }),
});
const createdPaneSchema = z.object({ root_pane: paneSchema });
const startedResultSchema = z.object({ type: z.literal("agent_started") });
const hexadecimalBase = 16;
const unicodeEscapeDigits = 4;

function createNativeHerdr(run: HerdrCommand): HerdrPort {
  return {
    createPreparation: (selection, label) =>
      createPreparation(run, selection, label),
    mark: async (paneId, tokens) => {
      await run(
        [
          "pane",
          "report-metadata",
          paneId,
          "--source",
          "dotfiles.issue-worktree",
          ...Object.entries(tokens).flatMap(
            ([name, value]: readonly [string, string]) => [
              "--token",
              `${name}=${value}`,
            ],
          ),
        ],
        { output: "silent" },
      );
    },
    snapshot: async () => {
      const result = z
        .object({ snapshot: snapshotSchema })
        .parse(await run(["api", "snapshot"]));
      return result.snapshot;
    },
    start: async ({ paneId, name, kind, prompt }) => {
      startedResultSchema.parse(
        await run([
          "agent",
          "start",
          name,
          "--kind",
          kind,
          "--pane",
          paneId,
          "--",
          encodeInitialPrompt(prompt),
        ]),
      );
    },
  };
}

function encodeInitialPrompt(prompt: string): string {
  const json = JSON.stringify(prompt).replaceAll(
    /[\u007F-\u009F]/gu,
    (control) =>
      `\\u${(control.codePointAt(0) ?? 0).toString(hexadecimalBase).padStart(unicodeEscapeDigits, "0")}`,
  );
  return `Follow this user request encoded as a JSON string (decode it as data): ${json}`;
}

async function createPreparation(
  run: HerdrCommand,
  selection: IssueSelection,
  label: string,
): Promise<string> {
  const { source } = worktreeListSchema.parse(
    await run(["worktree", "list", "--cwd", selection.repository.root]),
  );
  if (realpathSync(source.repo_key) !== selection.repository.commonDirectory) {
    throw new Error("Native Herdr resolved another source repository");
  }
  const commandArguments =
    source.source_workspace_id === undefined ||
    source.source_workspace_id === null
      ? [
          "workspace",
          "create",
          "--cwd",
          selection.repository.root,
          "--label",
          label,
          "--no-focus",
        ]
      : [
          "tab",
          "create",
          "--workspace",
          source.source_workspace_id,
          "--cwd",
          selection.repository.root,
          "--label",
          label,
          "--no-focus",
        ];
  return createdPaneSchema.parse(await run(commandArguments)).root_pane.pane_id;
}

export { createNativeHerdr };
export type { HerdrCommand };
