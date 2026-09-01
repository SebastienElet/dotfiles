import type {
  Agent,
  AgentCondition,
  ProcessOutput,
} from "./agent-memory-eval-process.ts";
import type {
  EvaluationScenario,
  EvaluationScenarios,
} from "./agent-memory-eval-scenario.ts";
import type { EvaluationFixture } from "./agent-memory-eval-root.ts";
import { expectedCapabilities } from "./agent-memory-eval-domain.ts";
import { join } from "node:path";
import { treeDigest } from "./agent-memory-eval-fixture.ts";
import { writeFile } from "node:fs/promises";

type CapabilityResult = Readonly<{ capability: string; passed: boolean }>;
type RunnerDependencies = Readonly<{
  runAgent: (
    ...request: readonly [
      agent: Agent,
      repository: string,
      environment: Readonly<NodeJS.ProcessEnv>,
      condition: AgentCondition,
      prompt: string,
      traceRoot: string,
      tracePath: string,
      nonce?: string,
      store?: string,
    ]
  ) => Promise<ProcessOutput>;
  agentVersion: (
    agent: Agent,
    environment: Readonly<NodeJS.ProcessEnv>,
  ) => Promise<string>;
}>;

async function runSession(
  ...[dependencies, agent, fixture, condition, prompt, name, nonce]: readonly [
    Readonly<RunnerDependencies>,
    Agent,
    Readonly<EvaluationFixture>,
    AgentCondition,
    string,
    string,
    string?,
  ]
): Promise<ProcessOutput> {
  const output = await dependencies.runAgent(
    agent,
    fixture.repository,
    fixture.environment,
    condition,
    prompt,
    fixture.root,
    join(fixture.raw, `${name}-runtime.jsonl`),
    nonce,
    fixture.store,
  );
  await writeFile(
    join(fixture.raw, `${name}.jsonl`),
    `${output.stdout}\n${output.stderr}`,
    {
      mode: 0o600,
    },
  );
  return output;
}

function interpolateProposal(prompt: string, proposal: string): string {
  if (!prompt.includes("{{proposal}}")) {
    throw new Error("admission scenario lacks proposal slot");
  }
  return prompt.replace("{{proposal}}", `\`\`\`yaml\n${proposal}\n\`\`\``);
}

function requiredFollowUp(scenario: EvaluationScenario): string {
  if (scenario.followUpPrompt === undefined) {
    throw new Error(`${scenario.id} lacks follow-up prompt`);
  }
  return scenario.followUpPrompt;
}

function storedStatus(output: string): boolean {
  return output
    .split("\n")
    .map((line) => parseJsonLine(line))
    .some((parsed) => isRecord(parsed) && parsed.status === "stored");
}

function storedEntryId(output: string): string {
  const stored = output
    .split("\n")
    .map((line) => parseJsonLine(line))
    .find((value) => isRecord(value) && value.status === "stored");
  if (!isRecord(stored) || typeof stored.id !== "string") {
    throw new Error("stored response lacks entry identity");
  }
  return stored.id;
}

function parseJsonLine(line: string): unknown {
  try {
    return JSON.parse(line);
  } catch {
    return undefined;
  }
}

function assertCapabilityOwnership(scenarios: EvaluationScenarios): void {
  const declared = scenarios.scenarios
    .flatMap((scenario) => scenario.capabilities)
    .toSorted();
  const expected = [...expectedCapabilities].toSorted();
  if (
    declared.length !== new Set(declared).size ||
    declared.join("\n") !== expected.join("\n")
  ) {
    throw new Error(
      "scenario capabilities must own every expected capability exactly once",
    );
  }
}

function declaredResults(
  scenarios: EvaluationScenarios,
  checks: Readonly<Record<string, boolean>>,
): CapabilityResult[] {
  return scenarios.scenarios.flatMap((scenario) =>
    scenario.capabilities.map((capability) => ({
      capability,
      passed: checks[capability] === true,
    })),
  );
}

async function fixtureDigest(
  fixture: Readonly<EvaluationFixture>,
): Promise<string> {
  const [repositoryDigest, storeDigest] = await Promise.all([
    treeDigest(fixture.repository),
    treeDigest(fixture.store),
  ]);
  return `${repositoryDigest}:${storeDigest}`;
}

function isRecord(value: unknown): value is Readonly<Record<string, unknown>> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

export {
  assertCapabilityOwnership,
  declaredResults,
  fixtureDigest,
  interpolateProposal,
  requiredFollowUp,
  runSession,
  storedEntryId,
  storedStatus,
};
export type { CapabilityResult, RunnerDependencies };
