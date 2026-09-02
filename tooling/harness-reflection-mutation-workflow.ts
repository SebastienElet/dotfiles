import type {
  HarnessMutationRequest,
  MutationWorkflowResult,
} from "./harness-reflection-mutation-workflow-types.ts";
import { createRepositoryMutationAdapter } from "./harness-reflection-mutation-filesystem.ts";
import { executeHarnessMutationWorkflowCore } from "./harness-reflection-mutation-workflow-core.ts";
import { resolve } from "node:path";

const repositoryRoot = resolve(import.meta.dir, "..");
const registryPath = "harness/invariants/registry.json";

const executeHarnessMutationWorkflow = (
  request: HarnessMutationRequest,
): Promise<MutationWorkflowResult> =>
  executeHarnessMutationWorkflowCore(
    { ...request, registryPath },
    createRepositoryMutationAdapter(repositoryRoot),
  );

export { executeHarnessMutationWorkflow };
export type {
  HarnessMutationRequest,
  MutationWorkflowResult,
} from "./harness-reflection-mutation-workflow-types.ts";
