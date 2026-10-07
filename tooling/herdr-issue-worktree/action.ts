import type { HerdrCommand } from "./native-herdr.ts";
import { createHerdrCommand } from "./herdr-command.ts";
import { parseIssueClick } from "./click.ts";
import { z } from "zod";

const actionEnvironmentSchema = z
  .object({
    HERDR_PLUGIN_CONTEXT_JSON: z.string().min(1),
    HERDR_PLUGIN_ID: z.literal("dotfiles.issue-worktree"),
  })
  .readonly();

async function openIssuePicker(
  environment: Readonly<Record<string, string | undefined>>,
  nativeCommand?: HerdrCommand,
): Promise<void> {
  const configuration = actionEnvironmentSchema.parse(environment);
  const click = parseIssueClick(configuration.HERDR_PLUGIN_CONTEXT_JSON);
  const run = nativeCommand ?? createHerdrCommand(environment);
  const commandArguments = [
    "plugin",
    "pane",
    "open",
    "--plugin",
    configuration.HERDR_PLUGIN_ID,
    "--entrypoint",
    "picker",
    "--env",
    `DOTFILES_ISSUE_CLICK_JSON=${JSON.stringify(click.context)}`,
    "--no-focus",
  ];
  z.object({ type: z.literal("ok") }).parse(await run(commandArguments));
}

if (import.meta.main) {
  try {
    await openIssuePicker(process.env);
  } catch (error) {
    process.stderr.write(
      `${JSON.stringify({ state: "picker-unavailable", reason: error instanceof Error ? error.message : String(error) })}\n`,
    );
    process.exitCode = 1;
  }
}

export { openIssuePicker };
