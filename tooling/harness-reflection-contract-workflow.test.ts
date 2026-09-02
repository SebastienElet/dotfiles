import { expect, test } from "bun:test";
import {
  loadHarnessReflectionSources,
  validateHarnessReflectionContract,
} from "./harness-reflection-contract.ts";
import { mutateContract } from "./harness-reflection-contract-test-support.ts";
import { resolve } from "node:path";

const repositoryRoot = resolve(import.meta.dir, "..");
const contractFinding =
  "authoritative contract preserves exact workflow invariants";

test.each([
  [
    "evaluates evidence before recording the registry lookup",
    [
      "read-authoritative-reference",
      "evaluate-concrete-evidence",
      "search-registry",
      "record-registry-lookup",
      "branch-on-evidence",
    ],
  ],
  [
    "skips recording the registry lookup when evidence is missing",
    [
      "read-authoritative-reference",
      "search-registry",
      "evaluate-concrete-evidence",
      "branch-on-evidence",
    ],
  ],
] as const)("rejects a harness-gap workflow that %s", async (_name, order) => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  const mutant = {
    ...sources,
    reference: mutateContract(
      sources.reference,
      [],
      (contract: Readonly<Record<string, unknown>>): void => {
        Reflect.set(contract, "harnessGapWorkflowOrder", order);
      },
    ),
  };

  expect(validateHarnessReflectionContract(mutant)).toContain(contractFinding);
});

test("requires a recorded lookup in every harness-gap report", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  const mutant = {
    ...sources,
    reference: mutateContract(
      sources.reference,
      ["report"],
      (report: Readonly<Record<string, unknown>>): void => {
        Reflect.set(
          report,
          "registryLookupAfterHarnessGap",
          "optional-when-evidence-missing",
        );
      },
    ),
  };

  expect(validateHarnessReflectionContract(mutant)).toContain(contractFinding);
});

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
