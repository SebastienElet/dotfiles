type JsonObject = Readonly<Record<string, unknown>>;

const codegraphEnvironment = {
  CODEGRAPH_NO_DOWNLOAD: "1",
  CODEGRAPH_NO_UPDATE_CHECK: "1",
  CODEGRAPH_TELEMETRY: "0",
} as const;
const configurationErrorExitCode = 2;

class ConfigurationError extends Error {
  public override readonly name = "ConfigurationError";
  public readonly exitCode = configurationErrorExitCode;
}

function parseJsonObject(content: string, label: string): JsonObject {
  const value = parseJson(content, label);
  if (!isJsonObject(value)) {
    throw new ConfigurationError(`invalid ${label} JSON`);
  }
  return value;
}

function parseJson(content: string, label: string): unknown {
  try {
    return JSON.parse(content);
  } catch {
    throw new ConfigurationError(`invalid ${label} JSON`);
  }
}

function configureCursor(current: JsonObject, command: string): JsonObject {
  const currentServers = current.mcpServers;
  if (
    currentServers !== undefined &&
    currentServers !== null &&
    !isJsonObject(currentServers)
  ) {
    throw new ConfigurationError("invalid Cursor MCP JSON");
  }
  return {
    ...current,
    mcpServers: {
      ...currentServers,
      codegraph: {
        args: ["serve", "--mcp", "--path", String.raw`\${workspaceFolder}`],
        command,
        env: codegraphEnvironment,
        type: "stdio",
      },
    },
  };
}

function isJsonObject(value: unknown): value is JsonObject {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

export {
  ConfigurationError,
  configureCursor,
  type JsonObject,
  parseJsonObject,
};
