import { chmod, copyFile, mkdir, writeFile } from "node:fs/promises";
import { dirname, join, resolve } from "node:path";

import type { Agent } from "./agent-memory-eval-process.ts";

const project = resolve(import.meta.dir, "..");
const privateFileMode = 0o600;
const jsonIndent = 2;

async function installAdapter(
  agent: Agent,
  home: string,
  runtime: string,
): Promise<void> {
  const skillRoot = join(
    home,
    agentConfigurationRoot(agent),
    "skills/memory-governance",
  );
  await mkdir(join(skillRoot, "references"), { mode: 0o700, recursive: true });
  await copyGovernanceSkill(skillRoot);
  return agent === "cursor"
    ? installCursorRule(home)
    : installPromptHook(agent, home, runtime);
}

async function copyGovernanceSkill(skillRoot: string): Promise<void> {
  const skill = join(skillRoot, "SKILL.md");
  const contract = join(skillRoot, "references/entry-contract.md");
  await copyFile(
    join(project, "harness/skills/memory-governance/SKILL.md"),
    skill,
  );
  await copyFile(
    join(
      project,
      "harness/skills/memory-governance/references/entry-contract.md",
    ),
    contract,
  );
  await Promise.all([
    chmod(skill, privateFileMode),
    chmod(contract, privateFileMode),
  ]);
}

async function installCursorRule(home: string): Promise<void> {
  const rule = join(home, ".cursor/rules/memory-governance-cursor.mdc");
  await mkdir(dirname(rule), { mode: 0o700, recursive: true });
  await copyFile(
    join(project, "harness/rules/memory-governance-cursor.mdc"),
    rule,
  );
  await chmod(rule, privateFileMode);
}

async function installPromptHook(
  agent: Exclude<Agent, "cursor">,
  home: string,
  runtime: string,
): Promise<void> {
  const directory = join(home, agent === "codex" ? ".codex" : ".claude");
  const destination = join(
    directory,
    agent === "codex" ? "hooks.json" : "settings.json",
  );
  await mkdir(directory, { mode: 0o700, recursive: true });
  const command = `'${runtime.replaceAll("'", String.raw`'\''`)}' hook --agent ${agent}`;
  const hooks = {
    hooks: {
      UserPromptSubmit: [
        { hooks: [{ command, timeout: 30, type: "command" }] },
      ],
    },
  };
  await writeFile(destination, `${JSON.stringify(hooks, null, jsonIndent)}\n`, {
    mode: privateFileMode,
  });
}

function agentConfigurationRoot(agent: Agent): string {
  if (agent === "codex") {
    return ".agents";
  }
  return agent === "claude" ? ".claude" : ".cursor";
}

export { installAdapter };
