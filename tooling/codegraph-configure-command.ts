import {
  ConfigurationError,
  isCurrentCodexConfiguration,
} from "./codegraph-config.ts";
import { accessSync, constants } from "node:fs";
import { ReportedCommandError } from "./codegraph-configure-reported-error.ts";

interface CommandResult {
  exitCode: number;
  output: string;
}
type Registration = "absent" | "current" | "registered";

class CommandError extends Error {
  public override readonly name = "CommandError";

  public constructor(
    public readonly exitCode: number,
    message: string,
  ) {
    super(message || `command failed with exit ${exitCode}`);
  }
}

function runCapturedCommand(
  binary: string,
  arguments_: readonly string[],
): CommandResult {
  const result = Bun.spawnSync([binary, ...arguments_], {
    stderr: "pipe",
    stdout: "pipe",
  });
  return {
    exitCode: result.exitCode,
    output: `${result.stdout.toString()}${result.stderr.toString()}`.trimEnd(),
  };
}

function probeClaude(binary: string): Registration {
  const result = runCapturedCommand(binary, ["mcp", "get", "codegraph"]);
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

function probeCodex(binary: string, codegraphBinary: string): Registration {
  const result = runCapturedCommand(binary, [
    "mcp",
    "get",
    "--json",
    "codegraph",
  ]);
  if (result.exitCode === 0) {
    return isCurrentCodexConfiguration(result.output, codegraphBinary)
      ? "current"
      : "registered";
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
    runMutationCommand(binary, [
      "mcp",
      "remove",
      "--scope",
      "user",
      "codegraph",
    ]);
  }
  runMutationCommand(binary, [
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
  ]);
}

function configureCodex(
  binary: string,
  codegraphBinary: string,
  registration: Registration,
): void {
  if (registration === "registered") {
    runMutationCommand(binary, ["mcp", "remove", "codegraph"]);
  }
  runMutationCommand(binary, [
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
  ]);
}

function runMutationCommand(
  binary: string,
  arguments_: readonly string[],
): void {
  const result = Bun.spawnSync([binary, ...arguments_], {
    stderr: "inherit",
    stdout: "inherit",
  });
  if (result.exitCode !== 0) {
    throw new ReportedCommandError(result.exitCode);
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

function commandExitCode(error: unknown): number {
  return error instanceof CommandError || error instanceof ReportedCommandError
    ? error.exitCode
    : 1;
}

export {
  CommandError,
  commandExitCode,
  configureClaude,
  configureCodex,
  probeClaude,
  probeCodex,
  requireExecutable,
  runMutationCommand,
};
