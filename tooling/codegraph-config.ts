import { z } from "zod";

type JsonObject = Readonly<Record<string, unknown>>;

const codegraphEnvironment = {
  CODEGRAPH_NO_DOWNLOAD: "1",
  CODEGRAPH_NO_UPDATE_CHECK: "1",
  CODEGRAPH_TELEMETRY: "0",
} as const;
const configurationErrorExitCode = 2;
const codegraphTransportSchema = z.object({
  args: z.tuple([z.literal("serve"), z.literal("--mcp")]),
  command: z.string(),
  env: z.object({
    CODEGRAPH_NO_DOWNLOAD: z.literal("1"),
    CODEGRAPH_NO_UPDATE_CHECK: z.literal("1"),
    CODEGRAPH_TELEMETRY: z.literal("0"),
  }),
  type: z.literal("stdio"),
});
const codexConfigurationSchema = z.object({
  enabled: z.literal(true),
  transport: codegraphTransportSchema,
});
const claudeConfigurationSchema = z
  .object({ mcpServers: z.object({ codegraph: codegraphTransportSchema }) })
  .loose();

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

function isCurrentClaudeConfiguration(
  configuration: JsonObject,
  command: string,
): boolean {
  const parsed = claudeConfigurationSchema.safeParse(configuration);
  return parsed.success && parsed.data.mcpServers.codegraph.command === command;
}

function isCurrentCodexConfiguration(
  content: string,
  command: string,
): boolean {
  const parsed = codexConfigurationSchema.safeParse(
    parseJson(content, "Codex MCP configuration"),
  );
  if (!parsed.success) {
    throw new ConfigurationError("invalid Codex MCP configuration");
  }
  return parsed.data.transport.command === command;
}

function isJsonObject(value: unknown): value is JsonObject {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

export {
  ConfigurationError,
  configureCursor,
  isCurrentClaudeConfiguration,
  isCurrentCodexConfiguration,
  type JsonObject,
  parseJsonObject,
};
