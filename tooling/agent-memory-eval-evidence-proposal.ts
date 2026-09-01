import {
  type ProposalValidation,
  type RecoveryRelation,
  isRecord,
  proposalStatesAcceptedRecovery,
} from "./agent-memory-eval-evidence-types.ts";
import { runtimeCommand, treeDigest } from "./agent-memory-eval-fixture.ts";

function extractProposal(text: string): string {
  const matches = [
    ...text.matchAll(/```ya?ml\s*\n(?<proposal>[\s\S]*?)\n```/gu),
  ];
  if (matches.length !== 1) {
    throw new Error("expected exactly one fenced YAML proposal");
  }
  const proposal = matches[0]?.groups?.proposal?.trim();
  if (proposal === undefined || proposal === "") {
    throw new Error("empty YAML proposal");
  }
  return proposal;
}

async function validateProposalWithRuntime(
  options: Readonly<{
    environment: Readonly<NodeJS.ProcessEnv>;
    evaluatedStore: string;
    expectedRelation: RecoveryRelation;
    proposal: string;
    repository: string;
    runtime: string;
    validationStore: string;
  }>,
): Promise<ProposalValidation> {
  const before = await treeDigest(options.evaluatedStore);
  const output = await runtimeCommand(
    options.runtime,
    ["admit", "--format", "json"],
    options.proposal,
    options.repository,
    { ...options.environment, AGENT_MEMORY_ROOT: options.validationStore },
  );
  const response: unknown = JSON.parse(output.stdout);
  return {
    evaluatedStoreUnchanged:
      before === (await treeDigest(options.evaluatedStore)),
    statementDetected: proposalStatesAcceptedRecovery(
      options.proposal,
      options.expectedRelation,
    ),
    stored: isRecord(response) && response.status === "stored",
  };
}

export { extractProposal, validateProposalWithRuntime };
