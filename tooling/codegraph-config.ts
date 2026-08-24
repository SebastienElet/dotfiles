export type JsonObject = Record<string, unknown>;

const codegraphEnvironment = {
  CODEGRAPH_TELEMETRY: "0",
  CODEGRAPH_NO_UPDATE_CHECK: "1",
  CODEGRAPH_NO_DOWNLOAD: "1",
} as const;

export function parseJsonObject(content: string, label: string): JsonObject {
  let value: unknown;
  try {
    value = JSON.parse(content);
  } catch {
    throw new ConfigurationError(`invalid ${label} JSON`);
  }
  if (!isJsonObject(value)) {
    throw new ConfigurationError(`invalid ${label} JSON`);
  }
  return value;
}

export function configureCursor(
  current: JsonObject,
  command: string,
): JsonObject {
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
      ...(currentServers ?? {}),
      codegraph: {
        type: "stdio",
        command,
        args: ["serve", "--mcp", "--path", "${workspaceFolder}"],
        env: codegraphEnvironment,
      },
    },
  };
}

export class ConfigurationError extends Error {
  readonly exitCode = 2;
}

function isJsonObject(value: unknown): value is JsonObject {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
