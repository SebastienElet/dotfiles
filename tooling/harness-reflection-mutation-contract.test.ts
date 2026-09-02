import {
  type HarnessReflectionSources,
  loadHarnessReflectionSources,
  parseHarnessReflectionContract,
  validateHarnessReflectionContract,
} from "./harness-reflection-contract.ts";
import { expect, test } from "bun:test";
import { mutateContract } from "./harness-reflection-contract-test-support.ts";
import { resolve } from "node:path";

const repositoryRoot = resolve(import.meta.dir, "..");
const contractFinding =
  "authoritative contract preserves exact workflow invariants";

const contractMutant = (
  sources: HarnessReflectionSources,
  path: readonly string[],
  mutate: (target: Readonly<Record<string, unknown>>) => void,
): HarnessReflectionSources => ({
  ...sources,
  reference: mutateContract(sources.reference, path, mutate),
});

test("orders proposal and exact approval before owner application", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  const contract = parseHarnessReflectionContract(sources.reference);

  expect(contract.decisionBranches).toEqual({
    link: [
      "prepare-link-proposal",
      "prepare-exact-registry-diff",
      "await-exact-manifest-approval",
    ],
    propose: [
      "select-and-propose-control-surface",
      "prepare-exact-surface-and-registry-diff",
      "await-exact-manifest-approval",
    ],
    skip: ["render-report"],
  });
  expect(
    contract.approvedChangeOrder.surfaceAndRegistry.indexOf(
      "present-exact-manifest-for-contextual-human-approval",
    ),
  ).toBeLessThan(
    contract.approvedChangeOrder.surfaceAndRegistry.indexOf(
      "apply-surface-with-required-owner",
    ),
  );
});

test("limits manifest validation to exact text and owner doctors", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  const contract = parseHarnessReflectionContract(sources.reference);

  expect(contract.manifestValidation).toEqual({
    appliesTo: ["always-loaded-instruction", "conditional-skill"],
    behavior: "read-only-no-file-writes",
    candidateTextRule: "exactly-added-for-promotion-and-removed-for-retirement",
    noOpRule: "every-approved-replacement-differs-from-preimage",
    semanticClaim: "exact-text-presence-and-absence-plus-owner-doctor-only",
    transitionKind: "derived-from-before-and-after",
  });
  expect(contract.retirement).toEqual({
    approval: "new-exact-attestation-recorded",
    historicalFields: "unchanged-except-approval-lifecycle-and-retirement",
    optionalFields: ["replacedBy"],
    requiredFields: ["retiredAt", "reason"],
    surfaceText: "exact-candidate-text-removed-by-required-owner",
  });
});

test.each([
  {
    key: "behavior",
    name: "surface writer",
    path: ["manifestValidation"],
    value: "writes-approved-files",
  },
  {
    key: "candidateTextRule",
    name: "unbound candidate text",
    path: ["manifestValidation"],
    value: "candidate-text-recorded-only",
  },
  {
    key: "surfaceAndRegistry",
    name: "application before approval",
    path: ["approvedChangeOrder"],
    value: [
      "apply-surface-with-required-owner",
      "present-exact-manifest-for-contextual-human-approval",
    ],
  },
  {
    key: "always-loaded-instruction",
    name: "missing instruction owner",
    path: ["surfaceOwners"],
    value: undefined,
  },
  {
    key: "historicalFields",
    name: "mutable retirement history",
    path: ["retirement"],
    value: "sources-may-change",
  },
] as const)("rejects a contract with $name", async (testCase) => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  const mutant = contractMutant(sources, testCase.path, (target): void => {
    if (testCase.value === undefined) {
      Reflect.deleteProperty(target, testCase.key);
    } else {
      Reflect.set(target, testCase.key, testCase.value);
    }
  });

  expect(validateHarnessReflectionContract(mutant)).toContain(contractFinding);
});

test("rejects reintroduction of a generic mutation execution contract", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  const mutant = contractMutant(sources, [], (contract): void => {
    Reflect.set(contract, "mutationExecution", {
      guarantee: "atomic-surface-and-registry-write",
    });
  });

  expect(validateHarnessReflectionContract(mutant)).toContain(contractFinding);
});
