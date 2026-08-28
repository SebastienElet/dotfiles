import {
  type ConfigurationFile,
  type ConfigurationMutation,
  inspectAgentConfiguration,
  restoreAgentConfiguration,
  updateAgentConfiguration,
} from "./firecrawl-retirement-config.ts";
import {
  inspectFirecrawlDocker,
  removeFirecrawlDocker,
} from "./firecrawl-retirement-docker.ts";
import { join } from "node:path";
import { z } from "zod";

const successExitCode = 0;
const failureExitCode = 1;
type CodexServer = Readonly<{ name: string }>;
const codexServerListSchema: z.ZodType<readonly CodexServer[]> = z.array(
  z.object({ name: z.string().min(1) }).loose(),
);

function runFirecrawlRetirement(): number {
  try {
    retireFirecrawl();
    return successExitCode;
  } catch (error) {
    process.stderr.write(`firecrawl-retirement: ${errorMessage(error)}\n`);
    return failureExitCode;
  }
}

function retireFirecrawl(): void {
  const home = requiredEnvironment("HOME");
  const dockerBinary = process.env.FIRECRAWL_RETIREMENT_DOCKER_BIN ?? "docker";
  const codexBinary = process.env.FIRECRAWL_RETIREMENT_CODEX_BIN ?? "codex";
  const configurations = [
    inspectAgentConfiguration(join(home, ".claude.json"), "Claude"),
    inspectAgentConfiguration(join(home, ".cursor", "mcp.json"), "Cursor"),
  ].filter((file): file is ConfigurationFile => isConfigurationFile(file));
  const codexHasFirecrawl = inspectCodex(codexBinary);
  const docker = inspectFirecrawlDocker(
    dockerBinary,
    process.env.DOCKER_UNAVAILABLE_POLICY ?? "require-docker",
  );
  updateConfigurations(configurations, codexBinary, codexHasFirecrawl);
  if (docker.state === "skipped") {
    process.stdout.write(
      "firecrawl-retirement result=skipped reason=docker-daemon-unavailable\n",
    );
    return;
  }
  removeFirecrawlDocker(dockerBinary, docker);
  const remainingDocker = inspectFirecrawlDocker(
    dockerBinary,
    "require-docker",
  );
  if (
    remainingDocker.state === "skipped" ||
    remainingDocker.containers.length > 0 ||
    remainingDocker.volumes.length > 0
  ) {
    throw new Error("Firecrawl Docker artifacts remain after removal");
  }
  process.stdout.write(
    `firecrawl-retirement result=removed containers=${docker.containers.length} volumes=${docker.volumes.length}\n`,
  );
  if (remainingDocker.images.length > 0) {
    process.stdout.write(
      `retained-images cleanup: docker image rm -- ${remainingDocker.images.join(" ")}\n`,
    );
  }
}

function inspectCodex(binary: string): boolean {
  const result = Bun.spawnSync([binary, "mcp", "list", "--json"], {
    stderr: "pipe",
    stdout: "pipe",
  });
  if (result.exitCode !== successExitCode) {
    process.stdout.write(result.stdout);
    process.stderr.write(result.stderr);
    throw new Error("Codex MCP inspection failed");
  }
  const parsed = parseCodexJson(result.stdout.toString());
  return codexServerListSchema
    .parse(parsed)
    .some((server: Readonly<CodexServer>) => server.name === "firecrawl");
}

function updateConfigurations(
  files: readonly ConfigurationFile[],
  codexBinary: string,
  codexHasFirecrawl: boolean,
): void {
  const updated: ConfigurationMutation[] = [];
  try {
    for (const file of files) {
      const mutation = updateAgentConfiguration(file);
      if (mutation !== undefined) {
        updated.push(mutation);
      }
    }
    if (codexHasFirecrawl) {
      removeCodexRegistration(codexBinary);
    }
  } catch (error) {
    for (const mutation of updated.toReversed()) {
      restoreAgentConfiguration(mutation);
    }
    throw error;
  }
}

function isConfigurationFile(
  file: Readonly<ConfigurationFile> | undefined,
): file is ConfigurationFile {
  return file !== undefined;
}

function parseCodexJson(content: string): unknown {
  try {
    return JSON.parse(content);
  } catch {
    throw new Error("invalid Codex MCP JSON");
  }
}

function removeCodexRegistration(binary: string): void {
  const result = Bun.spawnSync([binary, "mcp", "remove", "firecrawl"], {
    stderr: "inherit",
    stdout: "inherit",
  });
  if (result.exitCode !== successExitCode) {
    throw new Error("Codex MCP removal failed");
  }
}

function requiredEnvironment(name: string): string {
  const value = process.env[name];
  if (value === undefined || value.length === 0) {
    throw new Error(`${name} is required`);
  }
  return value;
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export { runFirecrawlRetirement };
