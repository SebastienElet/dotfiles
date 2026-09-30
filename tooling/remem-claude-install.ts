import { dirname, join } from "node:path";
import {
  existsSync,
  mkdirSync,
  readFileSync,
  renameSync,
  writeFileSync,
} from "node:fs";
import { mcpServerArguments, run, succeeds } from "./remem-process.ts";
import { z } from "zod";

const argumentsSchema = z.tuple([
  z.enum(["install", "verify"]),
  z.string().min(1),
]);
const settingsSchema = z.looseObject({});
const argumentOffset = 2;
const jsonIndentation = 2;

function settingsPath(home: string): string {
  return join(home, ".claude", "settings.json");
}

function readSettings(home: string): Record<string, unknown> {
  if (!existsSync(settingsPath(home))) {
    return {};
  }
  return settingsSchema.parse(
    JSON.parse(readFileSync(settingsPath(home), "utf8")),
  );
}

function writeSettings(
  home: string,
  settings: Readonly<Record<string, unknown>>,
): void {
  const path = settingsPath(home);
  const temporary = `${path}.${process.pid}.tmp`;
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(
    temporary,
    `${JSON.stringify(settings, null, jsonIndentation)}\n`,
  );
  renameSync(temporary, path);
}

function install(home: string): void {
  const settings = readSettings(home);
  if (succeeds(["claude", "mcp", "get", "remem"])) {
    run(["claude", "mcp", "remove", "--scope", "user", "remem"], {
      discardStdout: false,
    });
  }
  run(
    ["claude", "mcp", "add", "--scope", "user", "remem", ...mcpServerArguments],
    {
      discardStdout: false,
    },
  );
  writeSettings(home, { ...settings, autoMemoryEnabled: false });
}

function verify(home: string): void {
  if (readSettings(home).autoMemoryEnabled !== false) {
    throw new Error("Claude native memory is not disabled");
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
