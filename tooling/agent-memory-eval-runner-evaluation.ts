import {
  type EvaluationScenarios,
  scenarioById,
} from "./agent-memory-eval-scenario.ts";
import {
  type RunnerDependencies,
  declaredResults,
  fixtureDigest,
} from "./agent-memory-eval-runner-support.ts";
import type { Agent } from "./agent-memory-eval-process.ts";
import type { EvaluationFixture } from "./agent-memory-eval-root.ts";
import type { ReplicateResult } from "./agent-memory-eval-runner.ts";
import { capabilityChecks } from "./agent-memory-eval-domain.ts";
import { evaluateLifecycle } from "./agent-memory-eval-runner-lifecycle.ts";
import { evaluateProposal } from "./agent-memory-eval-runner-proposal.ts";
import { prepareAgent } from "./agent-memory-eval-auth.ts";
import { validateAdapterInstallation } from "./agent-memory-eval-evidence.ts";

type EvaluateReplicateRequest = Readonly<{
  agent: Agent;
  dependencies: RunnerDependencies;
  fixture: EvaluationFixture;
  replicate: number;
  scenarios: EvaluationScenarios;
}>;

async function evaluateReplicate(
  request: EvaluateReplicateRequest,
): Promise<ReplicateResult> {
  const { agent, dependencies, fixture, replicate, scenarios } = request;
  await prepareAgent(agent, fixture.home, fixture.runtime, fixture.environment);
  const adapterValid = await validateAdapterInstallation({
    agent,
    home: fixture.home,
    runtime: fixture.runtime,
    runtimeSource: fixture.runtimeSource,
  });
  const version = await dependencies.agentVersion(agent, fixture.environment);
  const proposalState = await fixtureDigest(fixture);
  const proposal = await evaluateProposal({
    agent,
    dependencies,
    fixture,
    proposalScenario: scenarioById(scenarios, "propose-without-writing"),
    retrievalScenario: scenarioById(scenarios, "admit-and-retrieve"),
    version,
  });
  const lifecycle = await evaluateLifecycle({
    admissionEntryId: proposal.admissionEntryId,
    agent,
    dependencies,
    fixture,
    lifecycleScenario: scenarioById(scenarios, "source-lifecycle"),
    retrievalScenario: scenarioById(scenarios, "admit-and-retrieve"),
    sensitiveScenario: scenarioById(scenarios, "reject-sensitive"),
    version,
  });
  const checks = capabilityChecks(
    {
      adapterValid,
      ...proposal,
      ...lifecycle,
      proposalState,
    },
    fixture.nonce,
  );
  return {
    capabilities: declaredResults(scenarios, checks),
    replicate,
    version,
  };
}

export { evaluateReplicate };
