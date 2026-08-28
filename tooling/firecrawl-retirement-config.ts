import {
  type FileVersion,
  recoverInterruptedFileReplacement,
  replaceFileVersion,
} from "./firecrawl-retirement-file.ts";
import { type Stats, lstatSync, readFileSync } from "node:fs";
import { z } from "zod";

type JsonObject = Readonly<Record<string, unknown>>;
type AgentConfiguration = JsonObject &
  Readonly<{ mcpServers?: Readonly<Record<string, unknown>> | undefined }>;
const jsonObjectSchema: z.ZodType<JsonObject> = z.record(
  z.string(),
  z.unknown(),
);
const agentConfigurationSchema: z.ZodType<AgentConfiguration> =
  jsonObjectSchema.and(
    z.object({ mcpServers: jsonObjectSchema.optional() }).loose(),
  );
const jsonIndentSpaces = 2;

type ConfigurationFile = Readonly<{
  configuration: AgentConfiguration;
  label: string;
  original: FileVersion;
  path: string;
}>;
type ConfigurationMutation = Readonly<{
  file: ConfigurationFile;
  updated: FileVersion;
}>;
function inspectAgentConfiguration(
  path: string,
  label: string,
): ConfigurationFile | undefined {
  recoverInterruptedFileReplacement(path);
  const metadata = inspectMetadata(path);
  if (metadata === undefined) {
    return undefined;
  }
  if (!metadata.isFile() || metadata.isSymbolicLink() || metadata.nlink > 1) {
    throw new Error(`${label} configuration must be one regular file: ${path}`);
  }
  const bytes = readFileSync(path);
  const content = [...bytes];
  const parsed = parseJson(bytes.toString("utf8"), label);
  const result = agentConfigurationSchema.safeParse(parsed);
  if (!result.success) {
    throw new Error(`invalid ${label} MCP configuration JSON`);
  }
  return {
    configuration: result.data,
    label,
    original: version(metadata, content),
    path,
  };
}

function withoutFirecrawl(
  configuration: Readonly<AgentConfiguration>,
): AgentConfiguration {
  const servers = configuration.mcpServers;
  if (servers === undefined || !("firecrawl" in servers)) {
    return configuration;
  }
  const { firecrawl: _removed, ...remainingServers } = servers;
  return { ...configuration, mcpServers: remainingServers };
}

function updateAgentConfiguration(
  file: Readonly<ConfigurationFile>,
): ConfigurationMutation | undefined {
  const updated = withoutFirecrawl(file.configuration);
  if (updated === file.configuration) {
    return undefined;
  }
  const updatedContent = [
    ...Buffer.from(`${JSON.stringify(updated, undefined, jsonIndentSpaces)}\n`),
  ];
  return {
    file,
    updated: replaceFileVersion({
      content: updatedContent,
      expected: file.original,
      label: file.label,
      mode: file.original.mode,
      path: file.path,
      phase: "during retirement",
    }),
  };
}

function restoreAgentConfiguration(
  mutation: Readonly<ConfigurationMutation>,
): void {
  replaceFileVersion({
    content: mutation.file.original.content,
    expected: mutation.updated,
    label: mutation.file.label,
    mode: mutation.file.original.mode,
    path: mutation.file.path,
    phase: "after update",
  });
}

function version(
  metadata: Readonly<{
    dev: number;
    ino: number;
    mode: number;
  }>,
  content: readonly number[],
): FileVersion {
  return {
    content,
    device: metadata.dev,
    inode: metadata.ino,
    mode: metadata.mode,
  };
}

function inspectMetadata(path: string): Stats | undefined {
  try {
    return lstatSync(path);
  } catch (error) {
    if (isMissingFile(error)) {
      return undefined;
    }
    throw error;
  }
}

function parseJson(content: string, label: string): unknown {
  try {
    return JSON.parse(content);
  } catch {
    throw new Error(`invalid ${label} configuration JSON`);
  }
}

function isMissingFile(error: unknown): boolean {
  return error instanceof Error && "code" in error && error.code === "ENOENT";
}

export {
  type ConfigurationFile,
  type ConfigurationMutation,
  inspectAgentConfiguration,
  restoreAgentConfiguration,
  updateAgentConfiguration,
};
