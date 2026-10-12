import { copyFileSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { z } from "zod";

const root = z.string().min(1).parse(process.env.GATE_FIXTURE);
const argumentOffset = 2;
const failureExitCode = 31;
const usageExitCode = 64;
const commandArguments = process.argv.slice(argumentOffset);
const [, operation] = commandArguments;
const failure = readFileSync(join(root, "failure"), "utf8");

if (operation === failure) {
  process.stderr.write("Claude command rejected\n");
  process.exit(failureExitCode);
}
if (
  JSON.stringify(commandArguments) ===
  JSON.stringify(["plugin", "list", "--json"])
) {
  process.stdout.write(readFileSync(join(root, "list.json")));
} else if (
  JSON.stringify(commandArguments) ===
  JSON.stringify([
    "plugin",
    "install",
    "codex",
    "--marketplace",
    "openai/codex-plugin-cc",
    "--scope",
    "user",
  ])
) {
  copyFileSync(join(root, "after-install.json"), join(root, "list.json"));
  process.stdout.write("Plugin installed or already installed\n");
} else {
  process.stderr.write("Unexpected Claude command\n");
  process.exit(usageExitCode);
}
