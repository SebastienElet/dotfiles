import { readFile, writeFile } from "node:fs/promises";
import { join, resolve } from "node:path";

import {
  memoryCommandErrorCode,
  memoryCommandObserved,
} from "./agent-memory-eval-action.ts";
import { prepareAgent } from "./agent-memory-eval-auth.ts";
import { parseAgentText } from "./agent-memory-eval-contract.ts";
import { capabilityChecks } from "./agent-memory-eval-domain.ts";
import {
  extractProposal,
  storedProposalEntryId,
  validateAdapterInstallation,
  validateProposalWithRuntime,
  validateStoreArtifacts,
  validateStoredProposal,
} from "./agent-memory-eval-evidence.ts";
import {
  acceptedRecoveryRelation,
  treeDigest,
} from "./agent-memory-eval-fixture.ts";
import {
  auditContainsInvalidatedEntry,
  contradictRecoveryEvidence,
  makeRecoveryMethodUnavailable,
  modelRefusedPersistence,
  restoreRecoveryMethod,
  sensitiveOutputIsRedacted,
  writeSensitiveFixture,
} from "./agent-memory-eval-phase-evidence.ts";
import type { Agent } from "./agent-memory-eval-process.ts";
import { withFreshEvaluationFixture } from "./agent-memory-eval-root.ts";
import type { EvaluationFixture } from "./agent-memory-eval-root.ts";
import {
  loadEvaluationScenarios,
  scenarioById,
} from "./agent-memory-eval-scenario.ts";
import type { EvaluationScenarios } from "./agent-memory-eval-scenario.ts";
import { relevantTrace, requireRuntimeTrace } from "./agent-memory-eval-session.ts";
import {
  assertCapabilityOwnership,
  declaredResults,
  fixtureDigest,
  interpolateProposal,
  requiredFollowUp,
  runSession,
} from "./agent-memory-eval-runner-support.ts";
import type {
  CapabilityResult,
  RunnerDependencies,
} from "./agent-memory-eval-runner-support.ts";

type ReplicateResult = Readonly<{
  replicate: number;
  version: string;
  capabilities: readonly CapabilityResult[];
}>;

const scenarioPath = resolve(import.meta.dir, "agent-memory-eval-scenarios.json");

async function runReplicate(
  agent: Agent,
  replicate: number,
  dependencies: RunnerDependencies,
): Promise<ReplicateResult> {
  const scenarios = await loadEvaluationScenarios(scenarioPath);
  assertCapabilityOwnership(scenarios);
  return withFreshEvaluationFixture(agent, replicate, (fixture) =>
    evaluateReplicate(agent, replicate, fixture, scenarios, dependencies),
  );
}

async function evaluateReplicate(
  agent: Agent,
  replicate: number,
  fixture: EvaluationFixture,
  scenarios: EvaluationScenarios,
  dependencies: RunnerDependencies,
): Promise<ReplicateResult> {
  await prepareAgent(agent, fixture.home, fixture.runtime, fixture.environment);
  const adapterValid = await validateAdapterInstallation({
    agent,
    home: fixture.home,
    runtime: fixture.runtime,
    runtimeSource: fixture.runtimeSource,
  });
  const version = await dependencies.agentVersion(agent, fixture.environment);
  const proposalScenario = scenarioById(scenarios, "propose-without-writing");
  const retrievalScenario = scenarioById(scenarios, "admit-and-retrieve");
  const sensitiveScenario = scenarioById(scenarios, "reject-sensitive");
  const lifecycleScenario = scenarioById(scenarios, "source-lifecycle");
  const proposalState = await fixtureDigest(fixture);
  const proposalSession = await runSession(
    dependencies,
    agent,
    fixture,
    "proposal",
    proposalScenario.prompt,
    "proposal",
  );
  requireRuntimeTrace(agent, proposalSession.runtimeTrace, "proposal");
  const proposalText = parseAgentText(agent, proposalSession.stdout, version).modelText;
  const proposal = extractProposal(proposalText);
  const proposalValidation = await validateProposalWithRuntime({
    environment: fixture.environment,
    evaluatedStore: fixture.store,
    expectedRelation: await acceptedRecoveryRelation(fixture.repository),
    proposal,
    repository: fixture.repository,
    runtime: fixture.runtime,
    validationStore: fixture.validationStore,
  });
  const afterProposal = await fixtureDigest(fixture);
  const admissionDraft = join(fixture.raw, "admission-proposal.yaml");
  const admissionActionTrace = join(fixture.raw, "admission-action-runtime.jsonl");
  await writeFile(admissionDraft, `${proposal}\n`, { mode: 0o600 });
  const admissionCommand = [
    `AGENT_MEMORY_EVAL_TRACE=${shellQuote(admissionActionTrace)}`,
    shellQuote(fixture.runtime),
    "admit --format json",
    `< ${shellQuote(admissionDraft)}`,
  ].join(" ");
  const admissionPrompt = `${interpolateProposal(retrievalScenario.prompt, proposal)}\n\nRun exactly this shell command once:\n\`\`\`sh\n${admissionCommand}\n\`\`\``;
  const admissionSession = await runSession(
    dependencies,
    agent,
    fixture,
    "admission",
    admissionPrompt,
    "admission",
  );
  requireRuntimeTrace(agent, admissionSession.runtimeTrace, "admission hook");
  const admissionActionRuntimeTrace = await readFile(admissionActionTrace, "utf8");
  if (admissionActionRuntimeTrace.includes('"event":"error"')) {
    throw new Error(
      `admission action failed: ${memoryCommandErrorCode(admissionSession.stdout) ?? "unavailable"}`,
    );
  }
  requireRuntimeTrace(
    agent,
    admissionActionRuntimeTrace,
    "admission action",
    "success",
    "admit",
  );
  const admissionEntryId = await storedProposalEntryId(fixture.store, proposal);
  const admissionStored = await validateStoredProposal(
    fixture.store,
    admissionEntryId,
    proposal,
  );
  const admissionObserved = memoryCommandObserved(
    agent,
    admissionSession.stdout,
    fixture.runtime,
    "admit",
  );
  const storeArtifactsValid = await validateStoreArtifacts(fixture.store);
  const controlState = await fixtureDigest(fixture);
  const control = await runSession(
    dependencies,
    agent,
    fixture,
    "control",
    "Answer only CONTROL-NO-MEMORY.",
    "control",
  );
  if (control.traceAbsent !== true) throw new Error("control unexpectedly invoked memory runtime");
  const afterControl = await fixtureDigest(fixture);
  const relevant = await runSession(
    dependencies,
    agent,
    fixture,
    "relevant",
    requiredFollowUp(retrievalScenario),
    "relevant",
    fixture.nonce,
  );
  const deployedTrace = relevantTrace(
    agent,
    relevant,
    fixture.nonce,
    fixture.runtime,
    fixture.store,
    version,
  );
  const unrelatedState = await fixtureDigest(fixture);
  const unrelated = await runSession(
    dependencies,
    agent,
    fixture,
    "unrelated",
    retrievalScenario.unrelatedPrompt ?? "",
    "unrelated",
  );
  requireRuntimeTrace(agent, unrelated.runtimeTrace, "unrelated");
  const afterUnrelated = await fixtureDigest(fixture);
  const sensitive = await writeSensitiveFixture(fixture.repository, fixture.nonce);
  const sensitiveStore = await treeDigest(fixture.store);
  const sensitiveSession = await runSession(
    dependencies,
    agent,
    fixture,
    "sensitive",
    sensitiveScenario.prompt,
    "sensitive",
  );
  requireRuntimeTrace(agent, sensitiveSession.runtimeTrace, "sensitive");
  const sensitiveText = parseAgentText(agent, sensitiveSession.stdout, version).modelText;
  const sensitiveOutput = `${sensitiveText}\n${sensitiveSession.stderr}`;
  const sensitiveUnchanged = sensitiveStore === (await treeDigest(fixture.store));
  await makeRecoveryMethodUnavailable(fixture.repository);
  const unavailableStore = await treeDigest(fixture.store);
  const unavailable = await runSession(
    dependencies,
    agent,
    fixture,
    "unavailable",
    lifecycleScenario.prompt,
    "unavailable",
  );
  requireRuntimeTrace(agent, unavailable.runtimeTrace, "unavailable", "unavailable");
  const unavailableText = parseAgentText(agent, unavailable.stdout, version).modelText;
  const unavailableNoMutation = unavailableStore === (await treeDigest(fixture.store));
  await restoreRecoveryMethod(fixture.repository);
  await contradictRecoveryEvidence(fixture.repository);
  const contradiction = await runSession(
    dependencies,
    agent,
    fixture,
    "contradiction",
    requiredFollowUp(lifecycleScenario),
    "contradiction",
  );
  requireRuntimeTrace(agent, contradiction.runtimeTrace, "contradiction");
  const lifecycle = {
    contradictionInvalidated: await auditContainsInvalidatedEntry(fixture, admissionEntryId),
    unavailableNoMutation,
    unavailableOmitted: !unavailableText.includes(fixture.nonce),
  };
  const checks = capabilityChecks({
    adapterValid,
    admissionObserved,
    admissionStored,
    afterControl,
    afterProposal,
    contextObserved: deployedTrace.contextBeforeModel,
    controlText: parseAgentText(agent, control.stdout, version).modelText,
    controlUnchanged: controlState === afterControl,
    lifecycle,
    nonce: fixture.nonce,
    proposalState,
    proposalValidation,
    runtimeObserved: deployedTrace.adapterCompletedBeforeModel,
    sensitiveRedacted: sensitiveOutputIsRedacted(sensitiveOutput, sensitive),
    sensitiveRefused:
      modelRefusedPersistence(sensitiveText) &&
      !memoryCommandObserved(agent, sensitiveSession.stdout, fixture.runtime, "admit"),
    sensitiveUnchanged,
    storeArtifactsValid,
    unrelatedText: parseAgentText(agent, unrelated.stdout, version).modelText,
    unrelatedUnchanged: unrelatedState === afterUnrelated,
  });
  return {
    capabilities: declaredResults(scenarios, checks),
    replicate,
    version,
  };
}

export { runReplicate };
export type { ReplicateResult };

function shellQuote(value: string): string {
  return `'${value.replaceAll("'", "'\\''")}'`;
}
