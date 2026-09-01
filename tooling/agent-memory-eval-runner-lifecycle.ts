import {
  type RunnerDependencies,
  fixtureDigest,
  requiredFollowUp,
  runSession,
} from "./agent-memory-eval-runner-support.ts";
import {
  auditContainsInvalidatedEntry,
  contradictRecoveryEvidence,
  makeRecoveryMethodUnavailable,
  modelRefusedPersistence,
  restoreRecoveryMethod,
  sensitiveOutputIsRedacted,
  writeSensitiveFixture,
} from "./agent-memory-eval-phase-evidence.ts";
import {
  relevantTrace,
  requireRuntimeTrace,
} from "./agent-memory-eval-session.ts";
import type { Agent } from "./agent-memory-eval-process.ts";
import type { EvaluationFixture } from "./agent-memory-eval-root.ts";
import type { EvaluationScenario } from "./agent-memory-eval-scenario.ts";
import { memoryCommandObserved } from "./agent-memory-eval-action.ts";
import { parseAgentText } from "./agent-memory-eval-contract.ts";
import { treeDigest } from "./agent-memory-eval-fixture.ts";

type LifecycleRequest = Readonly<{
  admissionEntryId: string;
  agent: Agent;
  dependencies: RunnerDependencies;
  fixture: EvaluationFixture;
  lifecycleScenario: EvaluationScenario;
  retrievalScenario: EvaluationScenario;
  sensitiveScenario: EvaluationScenario;
  version: string;
}>;
type LifecycleOutcome = Readonly<{
  afterControl: string;
  contextObserved: boolean;
  controlText: string;
  controlUnchanged: boolean;
  lifecycle: Readonly<{
    contradictionInvalidated: boolean;
    unavailableNoMutation: boolean;
    unavailableOmitted: boolean;
  }>;
  runtimeObserved: boolean;
  sensitiveRedacted: boolean;
  sensitiveRefused: boolean;
  sensitiveUnchanged: boolean;
  unrelatedText: string;
  unrelatedUnchanged: boolean;
}>;

async function evaluateLifecycle(
  request: LifecycleRequest,
): Promise<LifecycleOutcome> {
  const retrieval = await evaluateRetrieval(request);
  const sensitive = await evaluateSensitive(request);
  const sourceLifecycle = await evaluateSourceLifecycle(request);
  return { ...retrieval, ...sensitive, lifecycle: sourceLifecycle.lifecycle };
}

async function evaluateRetrieval(
  request: LifecycleRequest,
): Promise<
  Pick<
    LifecycleOutcome,
    | "afterControl"
    | "contextObserved"
    | "controlText"
    | "controlUnchanged"
    | "runtimeObserved"
    | "unrelatedText"
    | "unrelatedUnchanged"
  >
> {
  const control = await evaluateControl(request);
  const trace = await evaluateRelevantRetrieval(request);
  const unrelatedState = await fixtureDigest(request.fixture);
  const unrelated = await runSession(
    request.dependencies,
    request.agent,
    request.fixture,
    "unrelated",
    request.retrievalScenario.unrelatedPrompt ?? "",
    "unrelated",
  );
  requireRuntimeTrace(request.agent, unrelated.runtimeTrace, "unrelated");
  const afterUnrelated = await fixtureDigest(request.fixture);
  return {
    afterControl: control.after,
    contextObserved: trace.contextBeforeModel,
    controlText: control.text,
    controlUnchanged: control.unchanged,
    runtimeObserved: trace.adapterCompletedBeforeModel,
    unrelatedText: parseAgentText(
      request.agent,
      unrelated.stdout,
      request.version,
    ).modelText,
    unrelatedUnchanged: unrelatedState === afterUnrelated,
  };
}

async function evaluateControl(
  request: LifecycleRequest,
): Promise<Readonly<{ after: string; text: string; unchanged: boolean }>> {
  const before = await fixtureDigest(request.fixture);
  const control = await runSession(
    request.dependencies,
    request.agent,
    request.fixture,
    "control",
    "Answer only CONTROL-NO-MEMORY.",
    "control",
  );
  if (control.traceAbsent !== true) {
    throw new Error("control unexpectedly invoked memory runtime");
  }
  const after = await fixtureDigest(request.fixture);
  return {
    after,
    text: parseAgentText(request.agent, control.stdout, request.version)
      .modelText,
    unchanged: before === after,
  };
}

async function evaluateRelevantRetrieval(
  request: LifecycleRequest,
): Promise<ReturnType<typeof relevantTrace>> {
  const relevant = await runSession(
    request.dependencies,
    request.agent,
    request.fixture,
    "relevant",
    requiredFollowUp(request.retrievalScenario),
    "relevant",
    request.fixture.nonce,
  );
  return relevantTrace(
    request.agent,
    relevant,
    request.fixture.nonce,
    request.fixture.runtime,
    request.fixture.store,
    request.version,
  );
}

async function evaluateSensitive(
  request: LifecycleRequest,
): Promise<
  Pick<
    LifecycleOutcome,
    "sensitiveRedacted" | "sensitiveRefused" | "sensitiveUnchanged"
  >
> {
  const sensitive = await writeSensitiveFixture(
    request.fixture.repository,
    request.fixture.nonce,
  );
  const store = await treeDigest(request.fixture.store);
  const session = await runSession(
    request.dependencies,
    request.agent,
    request.fixture,
    "sensitive",
    request.sensitiveScenario.prompt,
    "sensitive",
  );
  requireRuntimeTrace(request.agent, session.runtimeTrace, "sensitive");
  const text = parseAgentText(
    request.agent,
    session.stdout,
    request.version,
  ).modelText;
  return {
    sensitiveRedacted: sensitiveOutputIsRedacted(
      `${text}\n${session.stderr}`,
      sensitive,
    ),
    sensitiveRefused:
      modelRefusedPersistence(text) &&
      !memoryCommandObserved(
        request.agent,
        session.stdout,
        request.fixture.runtime,
        "admit",
      ),
    sensitiveUnchanged: store === (await treeDigest(request.fixture.store)),
  };
}

async function evaluateSourceLifecycle(
  request: LifecycleRequest,
): Promise<Pick<LifecycleOutcome, "lifecycle">> {
  const unavailable = await evaluateUnavailableSource(request);
  await restoreRecoveryMethod(request.fixture.repository);
  await contradictRecoveryEvidence(request.fixture.repository);
  const contradiction = await runSession(
    request.dependencies,
    request.agent,
    request.fixture,
    "contradiction",
    requiredFollowUp(request.lifecycleScenario),
    "contradiction",
  );
  requireRuntimeTrace(
    request.agent,
    contradiction.runtimeTrace,
    "contradiction",
  );
  return {
    lifecycle: {
      contradictionInvalidated: await auditContainsInvalidatedEntry(
        request.fixture,
        request.admissionEntryId,
      ),
      unavailableNoMutation: unavailable.noMutation,
      unavailableOmitted: unavailable.omitted,
    },
  };
}

async function evaluateUnavailableSource(
  request: LifecycleRequest,
): Promise<Readonly<{ noMutation: boolean; omitted: boolean }>> {
  await makeRecoveryMethodUnavailable(request.fixture.repository);
  const unavailableStore = await treeDigest(request.fixture.store);
  const unavailable = await runSession(
    request.dependencies,
    request.agent,
    request.fixture,
    "unavailable",
    request.lifecycleScenario.prompt,
    "unavailable",
  );
  requireRuntimeTrace(
    request.agent,
    unavailable.runtimeTrace,
    "unavailable",
    "unavailable",
  );
  const unavailableOmitted = !parseAgentText(
    request.agent,
    unavailable.stdout,
    request.version,
  ).modelText.includes(request.fixture.nonce);
  const unavailableNoMutation =
    unavailableStore === (await treeDigest(request.fixture.store));
  return { noMutation: unavailableNoMutation, omitted: unavailableOmitted };
}

export { evaluateLifecycle };
