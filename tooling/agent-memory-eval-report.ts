import { arch, platform } from "node:os";
import {
  chmod,
  mkdir,
  readFile,
  readdir,
  rename,
  writeFile,
} from "node:fs/promises";
import { dirname, join, relative } from "node:path";
import { createHash } from "node:crypto";

const DEFAULT_REPLICATES = 3;
const PRIVATE_DIRECTORY_MODE = 0o700;
const JSON_INDENT = 2;
const PRIVATE_FILE_MODE = 0o600;

type AgentReceipt = Readonly<{
  agent: "codex" | "claude" | "cursor";
  capabilities: Readonly<Record<string, number>>;
  cleanup: "complete" | "failed";
  completedReplicates: number;
  requestedReplicates: number;
  status: "complete" | "blocked" | "failed";
  version: string;
  errorClass?: string;
}>;

type EvaluationReceipt = Readonly<{
  agents: Readonly<Partial<Record<AgentReceipt["agent"], AgentReceipt>>>;
  candidateSha: string;
  createdAt: string;
  environment: string;
  runtimeSha: string;
  schemaVersion: 1;
}>;

async function mergeReceipt(
  ...[path, candidateSha, runtimeSha, agent]: readonly [
    string,
    string,
    string,
    AgentReceipt,
  ]
): Promise<EvaluationReceipt> {
  const existing = await readReceipt(path);
  if (
    existing !== undefined &&
    (existing.candidateSha !== candidateSha ||
      existing.runtimeSha !== runtimeSha)
  ) {
    throw new Error("receipt candidate does not match");
  }
  const receipt: EvaluationReceipt = {
    agents: { ...existing?.agents, [agent.agent]: agent },
    candidateSha,
    createdAt: existing?.createdAt ?? new Date().toISOString(),
    environment: existing?.environment ?? `${platform()} ${arch()}`,
    runtimeSha,
    schemaVersion: 1,
  };
  await mkdir(dirname(path), { recursive: true, mode: PRIVATE_DIRECTORY_MODE });
  const temporary = `${path}.${process.pid}.tmp`;
  await writeFile(
    temporary,
    `${JSON.stringify(receipt, null, JSON_INDENT)}\n`,
    {
      mode: PRIVATE_FILE_MODE,
    },
  );
  await rename(temporary, path);
  await chmod(path, PRIVATE_FILE_MODE);
  return receipt;
}

function renderNormalizedReport(receipt: EvaluationReceipt): string {
  const agents = ["codex", "claude", "cursor"] as const;
  const agentRows = agents.map((agent) =>
    agentRow(agent, receipt.agents[agent]),
  );
  const capabilities = [
    ...new Set(
      agents.flatMap((agent) =>
        Object.keys(receipt.agents[agent]?.capabilities ?? {}),
      ),
    ),
  ].toSorted();
  const capabilityRows = capabilities.map((capability) =>
    [
      capability,
      ...agents.map((agent) =>
        capabilityCell(receipt.agents[agent], capability),
      ),
    ].join(" | "),
  );
  return [
    "# Durable memory end-to-end validation",
    "",
    `- Candidate SHA-256: \`${receipt.candidateSha}\``,
    `- Runtime SHA-256: \`${receipt.runtimeSha}\``,
    `- Date: ${receipt.createdAt}`,
    `- Environment: ${receipt.environment}`,
    "- Timeout: 120 seconds per process, then TERM and bounded KILL",
    "",
    "Agent | Version | Status | Replicates | Failure | Cleanup",
    "--- | --- | --- | ---: | --- | ---",
    ...agentRows,
    "",
    "Capability | Codex | Claude | Cursor",
    "--- | ---: | ---: | ---:",
    ...capabilityRows,
    "",
    "## Commands",
    "",
    ...agents.map(
      (agent) =>
        `- \`bun tooling/agent-memory-eval.ts --agent ${agent} --replicates ${receipt.agents[agent]?.requestedReplicates ?? DEFAULT_REPLICATES}\``,
    ),
    "",
    "Cleanup is recorded only after fixture removal is verified.",
    "",
    "Missing or blocked agents establish no capability. This macOS arm64 evidence does not establish Linux behavior.",
    "",
  ].join("\n");
}

async function evaluatorCandidateSha(project: string): Promise<string> {
  const tooling = join(project, "tooling");
  const toolingEntries = await readdir(tooling);
  const evaluator = toolingEntries
    .filter((name) => name.startsWith("agent-memory-eval"))
    .map((name) => join(tooling, name));
  const runtime = await sourceFiles(join(tooling, "agent-memory"));
  const adapters = [
    join(project, "harness/skills/memory-governance/SKILL.md"),
    join(
      project,
      "harness/skills/memory-governance/references/entry-contract.md",
    ),
    join(project, "harness/rules/memory-governance-cursor.mdc"),
  ];
  const hash = createHash("sha256");
  for (const path of [...adapters, ...evaluator, ...runtime].toSorted()) {
    hash.update(relative(project, path));
    hash.update(await readFile(path));
  }
  return hash.digest("hex");
}

function assertCandidateIdentity(
  ...[
    initialCandidateSha,
    initialRuntimeSha,
    finalCandidateSha,
    finalRuntimeSha,
  ]: readonly [string, string, string, string]
): void {
  if (
    initialCandidateSha !== finalCandidateSha ||
    initialRuntimeSha !== finalRuntimeSha
  ) {
    throw new Error("candidate changed during evaluation");
  }
}

async function sourceFiles(root: string): Promise<string[]> {
  const files: string[] = [];
  for (const entry of await readdir(root, { withFileTypes: true })) {
    if (entry.name !== "target") {
      const path = join(root, entry.name);
      if (entry.isDirectory()) {
        files.push(...(await sourceFiles(path)));
      } else if (entry.isFile()) {
        files.push(path);
      }
    }
  }
  return files;
}

function agentRow(
  agent: AgentReceipt["agent"],
  result: AgentReceipt | undefined,
): string {
  const name = agentName(agent);
  if (result === undefined) {
    return `${name} | unavailable | missing | 0/0 | not_run | not_run`;
  }
  return `${name} | ${result.version} | ${result.status} | ${result.completedReplicates}/${result.requestedReplicates} | ${result.errorClass ?? "none"} | ${result.cleanup}`;
}

function agentName(agent: AgentReceipt["agent"]): string {
  if (agent === "codex") {
    return "Codex";
  }
  if (agent === "claude") {
    return "Claude";
  }
  return "Cursor";
}

function capabilityCell(
  result: AgentReceipt | undefined,
  capability: string,
): string {
  return result === undefined
    ? "not_run"
    : `${result.capabilities[capability] ?? 0}/${result.requestedReplicates}`;
}

async function readReceipt(
  path: string,
): Promise<EvaluationReceipt | undefined> {
  try {
    const parsed: unknown = JSON.parse(await readFile(path, "utf8"));
    if (!isReceipt(parsed)) {
      throw new Error("invalid receipt");
    }
    return parsed;
  } catch (error) {
    if (error instanceof Error && "code" in error && error.code === "ENOENT") {
      return undefined;
    }
    throw error;
  }
}

function isReceipt(value: unknown): value is EvaluationReceipt {
  if (!isRecord(value)) {
    return false;
  }
  const record = value;
  return (
    record.schemaVersion === 1 &&
    typeof record.candidateSha === "string" &&
    typeof record.createdAt === "string" &&
    typeof record.environment === "string" &&
    typeof record.runtimeSha === "string" &&
    record.agents !== null &&
    typeof record.agents === "object" &&
    !Array.isArray(record.agents)
  );
}

function isRecord(value: unknown): value is Readonly<Record<string, unknown>> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

export {
  assertCandidateIdentity,
  evaluatorCandidateSha,
  mergeReceipt,
  renderNormalizedReport,
};
export type { AgentReceipt, EvaluationReceipt };
