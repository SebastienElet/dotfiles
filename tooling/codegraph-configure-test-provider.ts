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

const arguments_ = process.argv.slice(2);
const provider = identifyProvider(arguments_);
const operation = process.argv.slice(2, 4).join(" ");
const stateDirectory = requiredEnvironment("CODEGRAPH_TEST_STATE");
const logPath = requiredEnvironment("CODEGRAPH_TEST_LOG");
const markerPath = join(stateDirectory, provider);

mkdirSync(stateDirectory, { recursive: true });
appendFileSync(logPath, `${provider} ${process.argv.slice(2).join(" ")}\n`);

if (provider === "codegraph") {
  finish("telemetry off");
}

if (operation === "mcp get") {
  if (process.env.CODEGRAPH_TEST_UNEXPECTED_GET === provider) {
    console.error("unexpected provider response");
    process.exit(1);
  }
  if (existsSync(markerPath)) {
    process.exit(0);
  }
  console.error(
    provider === "claude"
      ? 'No MCP server named "codegraph".'
      : "Error: No MCP server named 'codegraph' found.",
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

process.exit(3);

function mutateConfiguration(value: string): void {
  const path =
    provider === "claude"
      ? requiredEnvironment("CODEGRAPH_CLAUDE_CONFIG")
      : requiredEnvironment("CODEGRAPH_CODEX_CONFIG");
  mkdirSync(dirname(path), { recursive: true });
  const current = existsSync(path) ? readFileSync(path, "utf8") : "";
  if (provider === "claude") {
    const configuration = current === "" ? {} : JSON.parse(current);
    const servers = configuration.mcpServers ?? {};
    if (value === "removed") {
      delete servers.codegraph;
    } else {
      servers.codegraph = { command: "codegraph", args: ["serve", "--mcp"] };
    }
    configuration.mcpServers = servers;
    writeFileSync(path, `${JSON.stringify(configuration)}\n`);
    return;
  }
  const unrelated = current
    .split("\n")
    .filter((line) => !line.startsWith(`${provider}:`))
    .filter((line, index, lines) => line !== "" || index < lines.length - 1)
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
    for (let attempt = 0; attempt < 1_000 && !existsSync(release); attempt++) {
      Bun.sleepSync(5);
    }
    if (!existsSync(release)) {
      process.exit(8);
    }
  }
  if (process.env.CODEGRAPH_TEST_EMIT === "1") {
    console.log(`out ${provider}:${expectedOperation}`);
    console.error(`err ${provider}:${expectedOperation}`);
  }
  if (process.env.CODEGRAPH_TEST_FAIL === `${provider}:${expectedOperation}`) {
    console.error(`failed ${provider}:${expectedOperation}`);
    process.exit(9);
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
  arguments_: string[],
): "claude" | "codex" | "codegraph" {
  if (arguments_[0] === "telemetry") {
    return "codegraph";
  }
  if (
    arguments_.includes("--json") ||
    arguments_.includes("--env") ||
    (arguments_[0] === "mcp" &&
      arguments_[1] === "remove" &&
      !arguments_.includes("--scope"))
  ) {
    return "codex";
  }
  return "claude";
}
