import {
  ConfigurationError,
  type JsonObject,
  configureCursor,
  isCurrentClaudeConfiguration,
} from "./codegraph-config.ts";
import {
  type ConfigurationSnapshot,
  inspectConfiguration,
  restoreConfigurations,
  withConfigurationLocks,
  writeJsonAtomically,
} from "./codegraph-config-files.ts";
import {
  commandExitCode,
  configureClaude,
  configureCodex,
  probeClaude,
  probeCodex,
  requireExecutable,
  runMutationCommand,
} from "./codegraph-configure-command.ts";
import { ReportedCommandError } from "./codegraph-configure-reported-error.ts";
import { join } from "node:path";

interface RequestedConfiguration {
  readonly claudeBinary: string;
  readonly claudePath: string;
  readonly codexBinary: string;
  readonly codexPath: string;
  readonly codegraphBinary: string;
  readonly cursorPath: string;
  readonly includeCursor: boolean;
}
interface CurrentConfiguration {
  readonly claude: JsonObject;
  readonly codex: JsonObject;
  readonly cursor: JsonObject | undefined;
}

function main(): number {
  try {
    configureCodegraph();
    return 0;
  } catch (error) {
    if (!(error instanceof ReportedCommandError)) {
      process.stderr.write(`${errorMessage(error)}\n`);
    }
    return error instanceof ConfigurationError
      ? error.exitCode
      : commandExitCode(error);
  }
}

function configureCodegraph(): void {
  const home = requiredEnvironment("HOME");
  const requested: RequestedConfiguration = {
    claudeBinary: process.env.CODEGRAPH_CLAUDE_BIN ?? "claude",
    claudePath:
      process.env.CODEGRAPH_CLAUDE_CONFIG ?? join(home, ".claude.json"),
    codexBinary: process.env.CODEGRAPH_CODEX_BIN ?? "codex",
    codexPath:
      process.env.CODEGRAPH_CODEX_CONFIG ??
      join(process.env.CODEX_HOME ?? join(home, ".codex"), "config.toml"),
    codegraphBinary: process.env.CODEGRAPH_BIN ?? "codegraph",
    cursorPath:
      process.env.CODEGRAPH_CURSOR_CONFIG ?? join(home, ".cursor", "mcp.json"),
    includeCursor: process.env.CODEGRAPH_INCLUDE_CURSOR === "1",
  };
  for (const binary of [
    requested.claudeBinary,
    requested.codexBinary,
    requested.codegraphBinary,
  ]) {
    requireExecutable(binary);
  }
  const paths = requested.includeCursor
    ? [requested.claudePath, requested.codexPath, requested.cursorPath]
    : [requested.claudePath, requested.codexPath];
  withConfigurationLocks(paths, () => {
    reconcileConfiguration(requested);
  });
}

function reconcileConfiguration(requested: RequestedConfiguration): void {
  const claude = inspectConfiguration(
    requested.claudePath,
    "agent config",
    "Claude configuration",
  );
  const codex = inspectConfiguration(requested.codexPath, "agent config");
  const cursor = requested.includeCursor
    ? inspectConfiguration(
        requested.cursorPath,
        "Cursor MCP config",
        "Cursor MCP",
      )
    : undefined;
  const snapshots = [
    claude.snapshot,
    codex.snapshot,
    ...(cursor === undefined ? [] : [cursor.snapshot]),
  ];

  runTransaction(snapshots, () => {
    applyConfiguration(requested, {
      claude: claude.parsed ?? {},
      codex: codex.parsed ?? {},
      cursor: cursor?.parsed,
    });
  });
}

function applyConfiguration(
  requested: RequestedConfiguration,
  current: CurrentConfiguration,
): void {
  const claudeRegistration = isCurrentClaudeConfiguration(
    current.claude,
    requested.codegraphBinary,
  )
    ? "current"
    : probeClaude(requested.claudeBinary);
  const codexRegistration = probeCodex(
    requested.codexBinary,
    requested.codegraphBinary,
  );
  const desiredCursor =
    current.cursor === undefined
      ? undefined
      : configureCursor(current.cursor, requested.codegraphBinary);
  const cursorCurrent =
    desiredCursor === undefined ||
    Bun.deepEquals(current.cursor, desiredCursor);
  if (
    claudeRegistration === "current" &&
    codexRegistration === "current" &&
    cursorCurrent
  ) {
    return;
  }
  runMutationCommand(requested.codegraphBinary, ["telemetry", "off"]);
  if (claudeRegistration !== "current") {
    configureClaude(
      requested.claudeBinary,
      requested.codegraphBinary,
      claudeRegistration,
    );
  }
  if (codexRegistration !== "current") {
    configureCodex(
      requested.codexBinary,
      requested.codegraphBinary,
      codexRegistration,
    );
  }
  if (!cursorCurrent && desiredCursor !== undefined) {
    writeJsonAtomically(requested.cursorPath, desiredCursor);
  }
}

function runTransaction(
  snapshots: readonly ConfigurationSnapshot[],
  mutation: () => void,
): void {
  try {
    mutation();
  } catch (error) {
    try {
      restoreConfigurations(snapshots);
    } catch (rollbackError) {
      throw new Error(
        `${errorMessage(error)}\n${errorMessage(rollbackError)}`,
        { cause: rollbackError },
      );
    }
    throw error;
  }
}

function requiredEnvironment(name: string): string {
  const value = process.env[name];
  if (value === undefined || value === "") {
    throw new ConfigurationError(`missing environment: ${name}`);
  }
  return value;
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export { main };
