import { checkCommand, reportCheckFailure } from "./check-command.ts";
import { z } from "zod";

const pluginsSchema = z.array(
  z.object({
    id: z.string().min(1),
    scope: z.string().min(1),
    enabled: z.boolean(),
  }),
);

function companionInstalled(): boolean {
  const result = checkCommand(["claude", "plugin", "list", "--json"]);
  const plugins = pluginsSchema.parse(JSON.parse(result.stdout.toString()));
  const companion = plugins.find(
    (plugin) => plugin.id === "codex@openai-codex" && plugin.scope === "user",
  );
  if (companion !== undefined && !companion.enabled) {
    throw new Error(
      "The user Codex companion is disabled; enable it explicitly",
    );
  }
  return companion !== undefined;
}

function installCompanion(): void {
  if (companionInstalled()) {
    return;
  }
  const result = checkCommand([
    "claude",
    "plugin",
    "install",
    "codex",
    "--marketplace",
    "openai/codex-plugin-cc",
    "--scope",
    "user",
  ]);
  process.stdout.write(result.stdout);
  process.stderr.write(result.stderr);
  if (!companionInstalled()) {
    throw new Error("Claude did not install the user Codex companion");
  }
}

try {
  installCompanion();
} catch (error) {
  reportCheckFailure(error);
}
