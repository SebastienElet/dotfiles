import type {
  ProposalOutcome,
  ProposalRequest,
} from "./agent-memory-eval-runner-proposal-types.ts";
import {
  extractProposal,
  storedProposalEntryId,
  validateProposalWithRuntime,
  validateStoreArtifacts,
  validateStoredProposal,
} from "./agent-memory-eval-evidence.ts";
import {
  fixtureDigest,
  interpolateProposal,
  runSession,
} from "./agent-memory-eval-runner-support.ts";
import {
  memoryCommandErrorCode,
  memoryCommandObserved,
} from "./agent-memory-eval-action.ts";
import { readFile, writeFile } from "node:fs/promises";
import type { Agent } from "./agent-memory-eval-process.ts";
import { acceptedRecoveryRelation } from "./agent-memory-eval-fixture.ts";
import { join } from "node:path";
import { parseAgentText } from "./agent-memory-eval-contract.ts";
import { requireRuntimeTrace } from "./agent-memory-eval-session.ts";

const privateFileMode = 0o600;

async function evaluateProposal(
  request: ProposalRequest,
): Promise<ProposalOutcome> {
  const proposal = await createProposal(request);
  const admission = await admitProposal(request, proposal);
  return {
    admissionEntryId: admission.entryId,
    admissionObserved: admission.observed,
    admissionStored: admission.stored,
    afterProposal: proposal.afterProposal,
    proposalValidation: proposal.valid,
    storeArtifactsValid: admission.storeArtifactsValid,
  };
}

async function createProposal(request: ProposalRequest): Promise<
  Readonly<{
    afterProposal: string;
    proposal: string;
    valid: Awaited<ReturnType<typeof validateProposalWithRuntime>>;
  }>
> {
  const before = await fixtureDigest(request.fixture);
  const session = await runSession(
    request.dependencies,
    request.agent,
    request.fixture,
    "proposal",
    request.proposalScenario.prompt,
    "proposal",
  );
  requireRuntimeTrace(request.agent, session.runtimeTrace, "proposal");
  const text = parseAgentText(
    request.agent,
    session.stdout,
    request.version,
  ).modelText;
  const proposal = extractProposal(text);
  const valid = await validateProposalWithRuntime({
    environment: request.fixture.environment,
    evaluatedStore: request.fixture.store,
    expectedRelation: await acceptedRecoveryRelation(
      request.fixture.repository,
    ),
    proposal,
    repository: request.fixture.repository,
    runtime: request.fixture.runtime,
    validationStore: request.fixture.validationStore,
  });
  const afterProposal = await fixtureDigest(request.fixture);
  if (before === "") {
    throw new Error("fixture digest is unavailable");
  }
  return { afterProposal, proposal, valid };
}

async function admitProposal(
  request: ProposalRequest,
  proposal: Readonly<{ proposal: string }>,
): Promise<
  Readonly<{
    entryId: string;
    observed: boolean;
    storeArtifactsValid: boolean;
    stored: Awaited<ReturnType<typeof validateStoredProposal>>;
  }>
> {
  const draft = join(request.fixture.raw, "admission-proposal.yaml");
  const trace = join(request.fixture.raw, "admission-action-runtime.jsonl");
  await writeFile(draft, `${proposal.proposal}\n`, { mode: privateFileMode });
  const prompt = admissionPrompt({
    draft,
    prompt: request.retrievalScenario.prompt,
    proposal: proposal.proposal,
    runtime: request.fixture.runtime,
    trace,
  });
  const session = await runSession(
    request.dependencies,
    request.agent,
    request.fixture,
    "admission",
    prompt,
    "admission",
  );
  requireRuntimeTrace(request.agent, session.runtimeTrace, "admission hook");
  const actionTrace = await readFile(trace, "utf8");
  assertAdmissionAction(request.agent, actionTrace, session.stdout);
  return proposalAdmissionOutcome(request, proposal.proposal, session.stdout);
}

async function proposalAdmissionOutcome(
  request: ProposalRequest,
  proposal: string,
  stdout: string,
): Promise<Awaited<ReturnType<typeof admitProposal>>> {
  const entryId = await storedProposalEntryId(request.fixture.store, proposal);
  return {
    entryId,
    observed: memoryCommandObserved(
      request.agent,
      stdout,
      request.fixture.runtime,
      "admit",
    ),
    storeArtifactsValid: await validateStoreArtifacts(request.fixture.store),
    stored: await validateStoredProposal(
      request.fixture.store,
      entryId,
      proposal,
    ),
  };
}

function admissionPrompt(
  request: Readonly<{
    draft: string;
    prompt: string;
    proposal: string;
    runtime: string;
    trace: string;
  }>,
): string {
  const command = [
    `AGENT_MEMORY_EVAL_TRACE=${shellQuote(request.trace)}`,
    shellQuote(request.runtime),
    "admit --format json",
    `< ${shellQuote(request.draft)}`,
  ].join(" ");
  return `${interpolateProposal(request.prompt, request.proposal)}\n\nRun exactly this shell command once:\n\`\`\`sh\n${command}\n\`\`\``;
}

function assertAdmissionAction(
  agent: Agent,
  trace: string,
  stdout: string,
): void {
  if (trace.includes('"event":"error"')) {
    throw new Error(
      `admission action failed: ${memoryCommandErrorCode(stdout) ?? "unavailable"}`,
    );
  }
  requireRuntimeTrace(agent, trace, "admission action", "success", "admit");
}

function shellQuote(value: string): string {
  return `'${value.replaceAll("'", String.raw`'\''`)}'`;
}

export { evaluateProposal };
