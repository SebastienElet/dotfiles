import type { Agent, AgentCondition } from "./agent-memory-eval-process.ts";
import { type PathLike, realpathSync } from "node:fs";
import { dirname, join } from "node:path";
import { homedir, tmpdir } from "node:os";

function buildAgentCommand(
  ...[agent, root, repository, condition, prompt]: readonly [
    Agent,
    string,
    string,
    AgentCondition,
    string,
  ]
): string[] {
  const sandbox = [
    "/usr/bin/sandbox-exec",
    "-p",
    sandboxProfile(root, repository),
  ];
  if (agent === "codex") {
    return codexCommand(sandbox, repository, prompt);
  }
  if (agent === "claude") {
    return claudeCommand(sandbox, condition, prompt);
  }
  return cursorCommand(sandbox, repository, condition, prompt);
}

function codexCommand(
  sandbox: readonly string[],
  repository: string,
  prompt: string,
): string[] {
  return [
    ...sandbox,
    "codex",
    "exec",
    "--ephemeral",
    "--json",
    "--ignore-user-config",
    "--disable",
    "apps",
    "--disable",
    "plugins",
    "--disable",
    "browser_use",
    "--disable",
    "multi_agent",
    "-c",
    'model_reasoning_effort="low"',
    "--sandbox",
    "danger-full-access",
    "--dangerously-bypass-hook-trust",
    "-C",
    repository,
    prompt,
  ];
}

function claudeCommand(
  sandbox: readonly string[],
  condition: AgentCondition,
  prompt: string,
): string[] {
  const admission = condition === "admission";
  return [
    ...sandbox,
    "claude",
    "-p",
    "--output-format",
    "stream-json",
    "--verbose",
    "--include-hook-events",
    "--no-session-persistence",
    "--permission-mode",
    admission ? "dontAsk" : "plan",
    ...(admission ? ["--allowedTools=Bash"] : []),
    prompt,
  ];
}

function cursorCommand(
  ...[sandbox, repository, condition, prompt]: readonly [
    readonly string[],
    string,
    AgentCondition,
    string,
  ]
): string[] {
  return [
    ...sandbox,
    "cursor-agent",
    "-p",
    "--output-format",
    "stream-json",
    "--mode",
    condition === "admission" ? "agent" : "plan",
    "--workspace",
    repository,
    prompt,
  ];
}

function sandboxProfile(root: string, repository: string): string {
  const path = (value: string): string =>
    value.replaceAll("\\", String.raw`\\`).replaceAll('"', String.raw`\"`);
  const roots = [...new Set([root, canonicalPath(root)])];
  const parentDirectories = [
    ...new Set(roots.flatMap((rootPath) => pathAncestors(rootPath))),
  ];
  const claudeTemporaryRoot = `/private/tmp/claude-${process.getuid?.() ?? "unknown"}`;
  const claudeTemporary = claudeTemporaryDirectory(repository);
  return [
    "(version 1)",
    "(allow default)",
    `(deny file-read* file-write* (subpath "/Users") (subpath "/Volumes") (subpath "/private/tmp") (subpath "${path(canonicalPath(tmpdir()))}"))`,
    `(allow file-read-metadata (subpath "/Users") (subpath "${path(canonicalPath(tmpdir()))}"))`,
    `(allow file-read* ${parentDirectories.map((directory) => `(literal "${path(directory)}")`).join(" ")})`,
    ...roots.map(
      (allowedRoot) =>
        `(allow file-read* file-write* (subpath "${path(allowedRoot)}"))`,
    ),
    `(allow file-read* (subpath "${path(join(homedir(), ".volta"))}"))`,
    `(allow file-read* (literal "${path(join(homedir(), ".local/bin/claude"))}"))`,
    `(allow file-read* (subpath "${path(join(homedir(), ".local/share/claude/versions"))}"))`,
    `(allow file-read* (literal "${path(claudeTemporaryRoot)}"))`,
    `(allow file-read* file-write* (subpath "${path(claudeTemporary)}"))`,
  ].join("");
}

function claudeTemporaryDirectory(repository: string): string {
  const root = `/private/tmp/claude-${process.getuid?.() ?? "unknown"}`;
  const name = canonicalPath(repository).replaceAll(/[^a-zA-Z0-9]/gu, "-");
  return join(root, name);
}

function pathAncestors(value: string): string[] {
  const ancestors: string[] = [];
  let current = dirname(value);
  while (current !== dirname(current)) {
    ancestors.push(current);
    current = dirname(current);
  }
  return ancestors;
}

function canonicalPath(
  ...[path]: readonly [Extract<PathLike, string>]
): string {
  return realpathSync(path);
}

export { buildAgentCommand, claudeTemporaryDirectory };
