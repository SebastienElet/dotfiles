import { expect, test } from "bun:test";
import {
  loadHarnessReflectionSources,
  validateHarnessReflectionContract,
} from "./harness-reflection-contract.ts";
import { mutateContract } from "./harness-reflection-contract-test-support.ts";
import { resolve } from "node:path";

const repositoryRoot = resolve(import.meta.dir, "..");

test("separates decision reporting from approved mutation work", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  const skipContinuesIntoMutation = {
    ...sources,
    reference: mutateContract(
      sources.reference,
      ["decisionBranches"],
      (branches: Readonly<Record<string, unknown>>): void => {
        Reflect.set(branches, "skip", ["require-approval", "render-report"]);
      },
    ),
  };
  const mutationBeforeApproval = {
    ...sources,
    reference: mutateContract(
      sources.reference,
      [],
      (contract: Readonly<Record<string, unknown>>): void => {
        Reflect.set(contract, "approvedMutationOrder", [
          "select-control-surface",
          "require-approval",
          "declare-consumers",
          "require-oracle",
          "run-cli",
          "render-report",
        ]);
      },
    ),
  };
  expect(
    validateHarnessReflectionContract(skipContinuesIntoMutation),
  ).toContain("authoritative contract preserves exact workflow invariants");
  expect(validateHarnessReflectionContract(mutationBeforeApproval)).toContain(
    "authoritative contract preserves exact workflow invariants",
  );
});
