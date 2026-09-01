import {
  type CapabilityResult,
  type RunnerDependencies,
  assertCapabilityOwnership,
} from "./agent-memory-eval-runner-support.ts";
import type { Agent } from "./agent-memory-eval-process.ts";
import type { EvaluationFixture } from "./agent-memory-eval-root.ts";
import { evaluateReplicate } from "./agent-memory-eval-runner-evaluation.ts";
import { loadEvaluationScenarios } from "./agent-memory-eval-scenario.ts";
import { resolve } from "node:path";
import { withFreshEvaluationFixture } from "./agent-memory-eval-root.ts";

type ReplicateResult = Readonly<{
  capabilities: readonly CapabilityResult[];
  replicate: number;
  version: string;
}>;

const scenarioPath = resolve(
  import.meta.dir,
  "agent-memory-eval-scenarios.json",
);

async function runReplicate(
  agent: Agent,
  replicate: number,
  dependencies: RunnerDependencies,
): Promise<ReplicateResult> {
  const scenarios = await loadEvaluationScenarios(scenarioPath);
  assertCapabilityOwnership(scenarios);
  return withFreshEvaluationFixture(
    agent,
    replicate,
    (fixture: Readonly<EvaluationFixture>) =>
      evaluateReplicate({ agent, dependencies, fixture, replicate, scenarios }),
  );
}

export { runReplicate };
export type { ReplicateResult };
