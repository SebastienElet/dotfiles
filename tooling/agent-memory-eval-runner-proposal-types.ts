import type { Agent } from "./agent-memory-eval-process.ts";
import type { EvaluationFixture } from "./agent-memory-eval-root.ts";
import type { EvaluationScenario } from "./agent-memory-eval-scenario.ts";
import type { RunnerDependencies } from "./agent-memory-eval-runner-support.ts";
import type { validateProposalWithRuntime } from "./agent-memory-eval-evidence.ts";

type ProposalRequest = Readonly<{
  agent: Agent;
  dependencies: RunnerDependencies;
  fixture: EvaluationFixture;
  proposalScenario: EvaluationScenario;
  retrievalScenario: EvaluationScenario;
  version: string;
}>;
type ProposalOutcome = Readonly<{
  admissionEntryId: string;
  admissionObserved: boolean;
  admissionStored: boolean;
  afterProposal: string;
  proposalValidation: Awaited<ReturnType<typeof validateProposalWithRuntime>>;
  storeArtifactsValid: boolean;
}>;

export type { ProposalOutcome, ProposalRequest };
