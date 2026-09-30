import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { run } from "./remem-process.ts";
import { z } from "zod";

const argumentsSchema = z.tuple([
  z.enum(["install", "verify"]),
  z.string().min(1),
]);
const hookSchema = z.looseObject({ command: z.string() }).readonly();
const hookGroupSchema = z
  .looseObject({ hooks: z.array(hookSchema).readonly() })
  .readonly();
const hooksSchema = z.looseObject({
  hooks: z.record(z.string(), z.array(hookGroupSchema).readonly()),
});
const expectedRememHooks = { claude: 6, codex: 3 } as const;
const argumentOffset = 2;

function install(home: string): void {
  const remem = join(home, ".local", "bin", "remem");
  const configuration = join(home, ".remem", "config.toml");
  const tracked = readFileSync(configuration);
  try {
    run([remem, "install", "--target", "claude", "--hooks-only"], {
      discardStdout: true,
    });
    run([remem, "install", "--target", "codex", "--hooks-only"], {
      discardStdout: true,
    });
  } finally {
    // The remem install command rewrites config.toml through its deployed symlink, adding an api.anthropic.com profile to the tracked source.
    writeFileSync(configuration, tracked);
  }
}

function countRememHooks(path: string, remem: string): number {
  const document = hooksSchema.parse(JSON.parse(readFileSync(path, "utf8")));
  const groups = Object.values(document.hooks).flat();
  return groups
    .flatMap((group) => group.hooks)
    .filter((hook) => hook.command.includes(remem)).length;
}

function verify(home: string): void {
  const remem = join(home, ".local", "bin", "remem");
  const registrations = [
    ["claude", join(home, ".claude", "settings.json")],
    ["codex", join(home, ".codex", "hooks.json")],
  ] as const;
  for (const [agent, path] of registrations) {
    const count = countRememHooks(path, remem);
    if (count !== expectedRememHooks[agent]) {
      throw new Error(
        `${path} registers ${count} remem hooks, expected ${expectedRememHooks[agent]}`,
      );
    }
  }
}

if (import.meta.main) {
  try {
    const [command, home] = argumentsSchema.parse(
      process.argv.slice(argumentOffset),
    );
    if (command === "install") {
      install(home);
    } else {
      verify(home);
    }
  } catch (error) {
    process.stderr.write(
      `Error: ${error instanceof Error ? error.message : String(error)}\n`,
    );
    process.exitCode = 1;
  }
}
