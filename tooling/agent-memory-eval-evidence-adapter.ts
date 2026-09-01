import { join, resolve } from "node:path";
import { lstat, readFile } from "node:fs/promises";

import type { Agent } from "./agent-memory-eval-process.ts";

const project = resolve(import.meta.dir, "..");
const DIRECTORY_MODE = 0o700;
const FILE_MODE = 0o600;
const MODE_MASK = 0o777;

async function validateAdapterInstallation(
  options: Readonly<{
    agent: Agent;
    home: string;
    runtime: string;
    runtimeSource: string;
  }>,
): Promise<boolean> {
  const runtimeMetadata = await lstat(options.runtime);
  const runtimeMode = runtimeMetadata.mode & MODE_MASK;
  if (runtimeMode !== DIRECTORY_MODE) {
    return false;
  }
  const [runtime, source] = await Promise.all([
    readFile(options.runtime),
    readFile(options.runtimeSource),
  ]);
  if (!runtime.equals(source)) {
    return false;
  }
  const skillRoot = join(
    options.home,
    agentSkillDirectory(options.agent),
    "skills/memory-governance",
  );
  if (
    !(await privateCanonicalFile(
      join(skillRoot, "SKILL.md"),
      "harness/skills/memory-governance/SKILL.md",
    ))
  ) {
    return false;
  }
  if (
    !(await privateCanonicalFile(
      join(skillRoot, "references/entry-contract.md"),
      "harness/skills/memory-governance/references/entry-contract.md",
    ))
  ) {
    return false;
  }
  if (options.agent === "cursor") {
    return privateCanonicalFile(
      join(options.home, ".cursor/rules/memory-governance-cursor.mdc"),
      "harness/rules/memory-governance-cursor.mdc",
    );
  }
  return exactHookConfig(options.agent, options.home, options.runtime);
}

function agentSkillDirectory(agent: Agent): string {
  if (agent === "codex") {
    return ".agents";
  }
  if (agent === "claude") {
    return ".claude";
  }
  return ".cursor";
}

async function privateCanonicalFile(
  path: string,
  source: string,
): Promise<boolean> {
  const metadata = await lstat(path);
  const mode = metadata.mode & MODE_MASK;
  if (mode !== FILE_MODE) {
    return false;
  }
  const [actual, canonical] = await Promise.all([
    readFile(path),
    readFile(join(project, source)),
  ]);
  return actual.equals(canonical);
}

async function exactHookConfig(
  agent: Agent,
  home: string,
  runtime: string,
): Promise<boolean> {
  const directory = join(home, agent === "codex" ? ".codex" : ".claude");
  const path = join(
    directory,
    agent === "codex" ? "hooks.json" : "settings.json",
  );
  const metadata = await lstat(path);
  if ((metadata.mode & MODE_MASK) !== FILE_MODE) {
    return false;
  }
  const command = `'${runtime.replaceAll("'", String.raw`'\\''`)}' hook --agent ${agent}`;
  const expected = `${JSON.stringify({ hooks: { UserPromptSubmit: [{ hooks: [{ command, timeout: 30, type: "command" }] }] } }, null, Number("2"))}\n`;
  return (await readFile(path, "utf8")) === expected;
}

export { validateAdapterInstallation };
