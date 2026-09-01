import { chmod, copyFile, mkdir, rm, writeFile } from "node:fs/promises";
import { realpathSync } from "node:fs";
import { homedir } from "node:os";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";

import {
  runEvaluationProcess,
  runManagedProcess,
  runManagedProcessToFile,
} from "./agent-memory-eval-process.ts";
import type { Agent, AgentCondition } from "./agent-memory-eval-process.ts";

const project = resolve(import.meta.dir, "..");

async function installCredential(
  agent: Agent,
  home: string,
  environment: NodeJS.ProcessEnv,
): Promise<void> {
  if (agent === "cursor") return;
  if (agent === "codex") {
    const destination = join(home, ".codex", "auth.json");
    await mkdir(dirname(destination), { recursive: true, mode: 0o700 });
    try {
      await copyFile(join(homedir(), ".codex", "auth.json"), destination);
    } catch {
      throw new Error("Codex authentication unavailable");
    }
    await chmod(destination, 0o600);
    return;
  }
  const destination = join(home, ".claude", ".credentials.json");
  await mkdir(dirname(destination), { recursive: true, mode: 0o700 });
  const account = (await runEvaluationProcess(["id", "-un"], environment, 5_000)).stdout.trim();
  try {
    await runManagedProcessToFile(
      [
        "/usr/bin/security",
        "find-generic-password",
        "-w",
        "-s",
        "Claude Code-credentials",
        "-a",
        account,
      ],
      process.env,
      destination,
      10_000,
    );
  } catch {
    await rm(destination, { force: true });
    throw new Error("Claude authentication unavailable");
  }
  await chmod(destination, 0o600);
}

async function installAdapter(agent: Agent, home: string, runtime: string): Promise<void> {
  const skillRoot = join(
    home,
    agent === "codex" ? ".agents" : agent === "claude" ? ".claude" : ".cursor",
    "skills/memory-governance",
  );
  await mkdir(join(skillRoot, "references"), { recursive: true, mode: 0o700 });
  const skill = join(skillRoot, "SKILL.md");
  const contract = join(skillRoot, "references/entry-contract.md");
  await copyFile(join(project, "harness/skills/memory-governance/SKILL.md"), skill);
  await copyFile(
    join(project, "harness/skills/memory-governance/references/entry-contract.md"),
    contract,
  );
  await Promise.all([chmod(skill, 0o600), chmod(contract, 0o600)]);
  if (agent === "cursor") return installCursorRule(home);
  const directory = join(home, agent === "codex" ? ".codex" : ".claude");
  const destination = join(directory, agent === "codex" ? "hooks.json" : "settings.json");
  await mkdir(directory, { recursive: true, mode: 0o700 });
  const command = `'${runtime.replaceAll("'", "'\\''")}' hook --agent ${agent}`;
  await writeFile(
    destination,
    `${JSON.stringify({ hooks: { UserPromptSubmit: [{ hooks: [{ command, timeout: 30, type: "command" }] }] } }, null, 2)}\n`,
    { mode: 0o600 },
  );
}

async function installCursorRule(home: string): Promise<void> {
  const rule = join(home, ".cursor/rules/memory-governance-cursor.mdc");
  await mkdir(dirname(rule), { recursive: true, mode: 0o700 });
  await copyFile(join(project, "harness/rules/memory-governance-cursor.mdc"), rule);
  await chmod(rule, 0o600);
}

async function prepareAgent(
  agent: Agent,
  home: string,
  runtime: string,
  environment: NodeJS.ProcessEnv,
): Promise<void> {
  await installCredential(agent, home, environment);
  await installAdapter(agent, home, runtime);
}

async function withCursorAuthentication<T>(
  environment: NodeJS.ProcessEnv,
  operation: (authenticated: NodeJS.ProcessEnv) => Promise<T>,
): Promise<T> {
  await runEvaluationProcess(
    ["cursor-agent", "status", "--format", "json"],
    process.env,
    10_000,
  );
  const credential = await runManagedProcess({
    command: [
      "/usr/bin/security",
      "find-generic-password",
      "-w",
      "-a",
      "cursor-user",
      "-s",
      "cursor-access-token",
    ],
    environment: process.env,
    timeoutMilliseconds: 10_000,
  });
  let token = credential.stdout.trim();
  if (token === "") throw new Error("Cursor authentication unavailable");
  try {
    const authenticated = { ...environment, CURSOR_AUTH_TOKEN: token };
    await runEvaluationProcess(
      ["cursor-agent", "status", "--format", "json"],
      authenticated,
      10_000,
    );
    return await operation(authenticated);
  } finally {
    token = "";
  }
}

function buildAgentCommand(
  agent: Agent,
  root: string,
  repository: string,
  condition: AgentCondition,
  prompt: string,
): string[] {
  const sandbox = ["/usr/bin/sandbox-exec", "-p", sandboxProfile(root, repository)];
  if (agent === "codex") {
    return [...sandbox,
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
  if (agent === "claude") {
    return [...sandbox,
      "claude",
      "-p",
      "--output-format",
      "stream-json",
      "--verbose",
      "--include-hook-events",
      "--no-session-persistence",
      "--permission-mode",
      condition === "admission" ? "dontAsk" : "plan",
      ...(condition === "admission" ? ["--allowedTools=Bash"] : []),
      prompt,
    ];
  }
  return [...sandbox,
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
  const path = (value: string) => value.replaceAll("\\", "\\\\").replaceAll('"', '\\"');
  const home = homedir();
  const roots = [...new Set([root, realpathSync(root)])];
  const parentDirectories = [...new Set(roots.flatMap(pathAncestors))];
  const claudeTemporaryRoot = `/private/tmp/claude-${process.getuid?.() ?? "unknown"}`;
  const claudeTemporary = claudeTemporaryDirectory(repository);
  return [
    "(version 1)",
    "(allow default)",
    `(deny file-read* file-write* (subpath "/Users") (subpath "/Volumes") (subpath "/private/tmp") (subpath "${path(realpathSync(tmpdir()))}"))`,
    `(allow file-read-metadata (subpath "/Users") (subpath "${path(realpathSync(tmpdir()))}"))`,
    `(allow file-read* ${parentDirectories.map((directory) => `(literal "${path(directory)}")`).join(" ")})`,
    ...roots.map(
      (allowedRoot) =>
        `(allow file-read* file-write* (subpath "${path(allowedRoot)}"))`,
    ),
    `(allow file-read* (subpath "${path(join(home, ".volta"))}"))`,
    `(allow file-read* (literal "${path(join(home, ".local/bin/claude"))}"))`,
    `(allow file-read* (subpath "${path(join(home, ".local/share/claude/versions"))}"))`,
    `(allow file-read* (literal "${path(claudeTemporaryRoot)}"))`,
    `(allow file-read* file-write* (subpath "${path(claudeTemporary)}"))`,
  ].join("");
}

function claudeTemporaryDirectory(repository: string): string {
  const root = `/private/tmp/claude-${process.getuid?.() ?? "unknown"}`;
  const name = realpathSync(repository).replaceAll(/[^a-zA-Z0-9]/gu, "-");
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

export {
  buildAgentCommand,
  claudeTemporaryDirectory,
  installAdapter,
  installCredential,
  prepareAgent,
  withCursorAuthentication,
};
