#!/usr/bin/env bun

import {
  appendFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { dirname, join } from "node:path";
import { z } from "zod";

const jsonObjectSchema = z.record(z.string(), z.unknown());
const commandArgumentOffset = 2;
const operationArgumentEnd = 4;
const pauseAttemptLimit = 1000;
const pausePollMilliseconds = 5;
const pauseTimeoutExitCode = 8;
const providerFailureExitCode = 9;
const unsupportedOperationExitCode = 3;

const providerArguments = process.argv.slice(commandArgumentOffset);
const provider = identifyProvider(providerArguments);
const operation = process.argv
  .slice(commandArgumentOffset, operationArgumentEnd)
  .join(" ");
const stateDirectory = requiredEnvironment("CODEGRAPH_TEST_STATE");
const logPath = requiredEnvironment("CODEGRAPH_TEST_LOG");
const markerPath = join(stateDirectory, provider);

mkdirSync(stateDirectory, { recursive: true });
appendFileSync(
  logPath,
  `${provider} ${process.argv.slice(commandArgumentOffset).join(" ")}\n`,
);

if (provider === "codegraph") {
  finish("telemetry off");
}

if (operation === "mcp get") {
  if (process.env.CODEGRAPH_TEST_UNEXPECTED_GET === provider) {
    process.stderr.write("unexpected provider response\n");
    process.exit(1);
  }
  if (existsSync(markerPath)) {
    process.exit(0);
  }
  process.stderr.write(
    provider === "claude"
      ? 'No MCP server named "codegraph".\n'
      : "Error: No MCP server named 'codegraph' found.\n",
  );
  process.exit(1);
}

if (operation === "mcp remove") {
  mutateConfiguration("removed");
  rmSync(markerPath, { force: true });
  finish("mcp remove");
}

if (operation === "mcp add") {
  mutateConfiguration("added");
  writeFileSync(markerPath, "registered\n");
  finish("mcp add");
}

process.exit(unsupportedOperationExitCode);

function mutateConfiguration(value: string): void {
  const path =
    provider === "claude"
      ? requiredEnvironment("CODEGRAPH_CLAUDE_CONFIG")
      : requiredEnvironment("CODEGRAPH_CODEX_CONFIG");
  mkdirSync(dirname(path), { recursive: true });
  const current = existsSync(path) ? readFileSync(path, "utf8") : "";
  if (provider === "claude") {
    const configuration =
      current === "" ? {} : jsonObjectSchema.parse(JSON.parse(current));
    const serversValue = configuration.mcpServers;
    const servers =
      serversValue === undefined ? {} : jsonObjectSchema.parse(serversValue);
    if (value === "removed") {
      delete servers.codegraph;
    } else {
      servers.codegraph = { args: ["serve", "--mcp"], command: "codegraph" };
    }
    configuration.mcpServers = servers;
    writeFileSync(path, `${JSON.stringify(configuration)}\n`);
    return;
  }
  const unrelated = current
    .split("\n")
    .filter((line) => !line.startsWith(`${provider}:`))
    .filter(
      (line, index, lines: readonly string[]) =>
        line !== "" || index < lines.length - 1,
    )
    .join("\n");
  writeFileSync(
    path,
    `${unrelated}${unrelated.endsWith("\n") ? "" : "\n"}${provider}:${value}\n`,
  );
}

function finish(expectedOperation: string): never {
  if (process.env.CODEGRAPH_TEST_PAUSE === `${provider}:${expectedOperation}`) {
    const ready = requiredEnvironment("CODEGRAPH_TEST_PAUSE_READY");
    const release = requiredEnvironment("CODEGRAPH_TEST_PAUSE_RELEASE");
    writeFileSync(ready, "ready\n");
    for (
      let attempt = 0;
      attempt < pauseAttemptLimit && !existsSync(release);
      attempt += 1
    ) {
      Bun.sleepSync(pausePollMilliseconds);
    }
    if (!existsSync(release)) {
      process.exit(pauseTimeoutExitCode);
    }
  }
  if (process.env.CODEGRAPH_TEST_EMIT === "1") {
    process.stdout.write(`out ${provider}:${expectedOperation}\n`);
    process.stderr.write(`err ${provider}:${expectedOperation}\n`);
  }
  if (process.env.CODEGRAPH_TEST_FAIL === `${provider}:${expectedOperation}`) {
    process.stderr.write(`failed ${provider}:${expectedOperation}\n`);
    process.exit(providerFailureExitCode);
  }
  process.exit(0);
}

function requiredEnvironment(name: string): string {
  const value = process.env[name];
  if (value === undefined || value === "") {
    throw new Error(`missing test environment: ${name}`);
  }
  return value;
}

function identifyProvider(
  commandArguments: readonly string[],
): "claude" | "codex" | "codegraph" {
  if (commandArguments[0] === "telemetry") {
    return "codegraph";
  }
  if (
    commandArguments.includes("--json") ||
    commandArguments.includes("--env") ||
    (commandArguments[0] === "mcp" &&
      commandArguments[1] === "remove" &&
      !commandArguments.includes("--scope"))
  ) {
    return "codex";
  }
  return "claude";
}
