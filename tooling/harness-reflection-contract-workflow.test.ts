import { expect, test } from "bun:test";
import {
  loadHarnessReflectionSources,
  validateHarnessReflectionContract,
} from "./harness-reflection-contract.ts";
import { resolve } from "node:path";

const repositoryRoot = resolve(import.meta.dir, "..");

test("separates decision reporting from approved mutation work", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  const skipContinuesIntoMutation = {
    ...sources,
    reference: sources.reference.replace(
      '"skip": ["render-report"]',
      '"skip": ["require-approval", "render-report"]',
    ),
  };
  const mutationBeforeApproval = {
    ...sources,
    reference: sources.reference.replace(
      '"approvedMutationOrder": [\n    "require-approval",\n    "select-control-surface",',
      '"approvedMutationOrder": [\n    "select-control-surface",\n    "require-approval",',
    ),
  };

  expect(skipContinuesIntoMutation.reference).not.toBe(sources.reference);
  expect(mutationBeforeApproval.reference).not.toBe(sources.reference);
  expect(
    validateHarnessReflectionContract(skipContinuesIntoMutation),
  ).toContain("authoritative contract preserves exact workflow invariants");
  expect(validateHarnessReflectionContract(mutationBeforeApproval)).toContain(
    "authoritative contract preserves exact workflow invariants",
  );
});
