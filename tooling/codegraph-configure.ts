import { accessSync, constants } from "node:fs";
import { join } from "node:path";
import { configureCursor, ConfigurationError } from "./codegraph-config.ts";
import {
  inspectConfiguration,
  restoreConfigurations,
  withConfigurationLocks,
  writeJsonAtomically,
  type ConfigurationSnapshot,
} from "./codegraph-config-files.ts";

type CommandResult = { exitCode: number; output: string };
type Registration = "absent" | "registered";

export function main(): number {
  try {
    configureCodegraph();
    return 0;
  } catch (error) {
    console.error(errorMessage(error));
    return error instanceof ConfigurationError
      ? error.exitCode
      : commandExitCode(error);
  }
}

function configureCodegraph(): void {
  const home = requiredEnvironment("HOME");
  const claudeBinary = process.env.CODEGRAPH_CLAUDE_BIN ?? "claude";
  const codexBinary = process.env.CODEGRAPH_CODEX_BIN ?? "codex";
  const codegraphBinary = process.env.CODEGRAPH_BIN ?? "codegraph";
  const cursorPath =
    process.env.CODEGRAPH_CURSOR_CONFIG ?? join(home, ".cursor", "mcp.json");
  const claudePath =
    process.env.CODEGRAPH_CLAUDE_CONFIG ?? join(home, ".claude.json");
  const codexPath =
    process.env.CODEGRAPH_CODEX_CONFIG ??
    join(process.env.CODEX_HOME ?? join(home, ".codex"), "config.toml");

  for (const binary of [claudeBinary, codexBinary, codegraphBinary]) {
    requireExecutable(binary);
  }

  withConfigurationLocks([claudePath, codexPath, cursorPath], () => {
    const claude = inspectConfiguration(
      claudePath,
      "agent config",
      "Claude configuration",
    );
    const codex = inspectConfiguration(codexPath, "agent config");
    const cursor = inspectConfiguration(
      cursorPath,
      "Cursor MCP config",
      "Cursor MCP",
    );
    const snapshots = [claude.snapshot, codex.snapshot, cursor.snapshot];

    runTransaction(snapshots, () => {
      const claudeRegistration = probeClaude(claudeBinary);
      const codexRegistration = probeCodex(codexBinary);
      requireSuccess(runCommand(codegraphBinary, ["telemetry", "off"]));
      configureClaude(claudeBinary, codegraphBinary, claudeRegistration);
      configureCodex(codexBinary, codegraphBinary, codexRegistration);
      writeJsonAtomically(
        cursorPath,
        configureCursor(cursor.parsed ?? {}, codegraphBinary),
      );
    });
  });
}

function runTransaction(
  snapshots: ConfigurationSnapshot[],
  mutation: () => void,
): void {
  try {
    mutation();
  } catch (error) {
    try {
      restoreConfigurations(snapshots);
    } catch (rollbackError) {
      throw new Error(`${errorMessage(error)}\n${errorMessage(rollbackError)}`);
    }
    throw error;
  }
}

function probeClaude(binary: string): Registration {
  const result = runCommand(binary, ["mcp", "get", "codegraph"]);
  if (result.exitCode === 0) {
    return "registered";
  }
  const absentMessages = [
    'No MCP server named "codegraph".',
    'No MCP server named "codegraph". .mcp.json servers are awaiting approval — run `claude` in this directory to review them.',
  ];
  if (result.exitCode === 1 && absentMessages.includes(result.output)) {
    return "absent";
  }
  throw new CommandError(result.exitCode, result.output);
}

function probeCodex(binary: string): Registration {
  const result = runCommand(binary, ["mcp", "get", "--json", "codegraph"]);
  if (result.exitCode === 0) {
    return "registered";
  }
  if (
    result.exitCode === 1 &&
    result.output === "Error: No MCP server named 'codegraph' found."
  ) {
    return "absent";
  }
  throw new CommandError(result.exitCode, result.output);
}

function configureClaude(
  binary: string,
  codegraphBinary: string,
  registration: Registration,
): void {
  if (registration === "registered") {
    requireSuccess(
      runCommand(binary, ["mcp", "remove", "--scope", "user", "codegraph"]),
    );
  }
  requireSuccess(
    runCommand(binary, [
      "mcp",
      "add",
      "--scope",
      "user",
      "codegraph",
      "-e",
      "CODEGRAPH_TELEMETRY=0",
      "-e",
      "CODEGRAPH_NO_UPDATE_CHECK=1",
      "-e",
      "CODEGRAPH_NO_DOWNLOAD=1",
      "--",
      codegraphBinary,
      "serve",
      "--mcp",
    ]),
  );
}

function configureCodex(
  binary: string,
  codegraphBinary: string,
  registration: Registration,
): void {
  if (registration === "registered") {
    requireSuccess(runCommand(binary, ["mcp", "remove", "codegraph"]));
  }
  requireSuccess(
    runCommand(binary, [
      "mcp",
      "add",
      "codegraph",
      "--env",
      "CODEGRAPH_TELEMETRY=0",
      "--env",
      "CODEGRAPH_NO_UPDATE_CHECK=1",
      "--env",
      "CODEGRAPH_NO_DOWNLOAD=1",
      "--",
      codegraphBinary,
      "serve",
      "--mcp",
    ]),
  );
}

function runCommand(binary: string, arguments_: string[]): CommandResult {
  const result = Bun.spawnSync([binary, ...arguments_], {
    stdout: "pipe",
    stderr: "pipe",
  });
  return {
    exitCode: result.exitCode,
    output: `${result.stdout.toString()}${result.stderr.toString()}`.trimEnd(),
  };
}

function requireSuccess(result: CommandResult): void {
  if (result.exitCode !== 0) {
    throw new CommandError(result.exitCode, result.output);
  }
}

function requireExecutable(binary: string): void {
  const found = binary.includes("/") ? binary : Bun.which(binary);
  try {
    accessSync(found ?? binary, constants.X_OK);
  } catch {
    throw new ConfigurationError(`missing executable: ${binary}`);
  }
}

function requiredEnvironment(name: string): string {
  const value = process.env[name];
  if (value === undefined || value === "") {
    throw new ConfigurationError(`missing environment: ${name}`);
  }
  return value;
}

class CommandError extends Error {
  constructor(
    readonly exitCode: number,
    message: string,
  ) {
    super(message || `command failed with exit ${exitCode}`);
  }
}

function commandExitCode(error: unknown): number {
  return error instanceof CommandError ? error.exitCode : 1;
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
