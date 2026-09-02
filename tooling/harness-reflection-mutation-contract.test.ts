import {
  type HarnessReflectionSources,
  loadHarnessReflectionSources,
  validateHarnessReflectionContract,
} from "./harness-reflection-contract.ts";
import { expect, test } from "bun:test";
import { mutateContract } from "./harness-reflection-contract-test-support.ts";
import { resolve } from "node:path";

const repositoryRoot = resolve(import.meta.dir, "..");
const contractFinding =
  "authoritative contract preserves exact workflow invariants";
const isRecord = (value: unknown): value is Readonly<Record<string, unknown>> =>
  typeof value === "object" && value !== null && !Array.isArray(value);

const contractMutant = (
  sources: HarnessReflectionSources,
  path: readonly string[],
  mutate: (target: Readonly<Record<string, unknown>>) => void,
): HarnessReflectionSources => ({
  ...sources,
  reference: mutateContract(sources.reference, path, mutate),
});

const authoritativeContract = async (): Promise<
  Readonly<Record<string, unknown>>
> => {
  const { reference } = await loadHarnessReflectionSources(repositoryRoot);
  const json = /```json\n(?<json>[\s\S]*?)\n```/u.exec(reference)?.groups?.json;
  if (json === undefined) {
    throw new Error("authoritative contract missing");
  }
  const parsed: unknown = JSON.parse(json);
  if (!isRecord(parsed)) {
    throw new TypeError("authoritative contract is not an object");
  }
  return parsed;
};

const nestedRecord = (
  source: Readonly<Record<string, unknown>>,
  key: string,
): Readonly<Record<string, unknown>> => {
  const value = Reflect.get(source, key);
  if (!isRecord(value)) {
    throw new TypeError(`contract object missing: ${key}`);
  }
  return value;
};

test("owns validation, approval matching and compensated mutation", async () => {
  const contract = await authoritativeContract();

  expect(Reflect.get(contract, "decisionBranches")).toEqual({
    link: ["hold-session-local", "await-explicit-approval"],
    propose: ["hold-session-local", "await-explicit-approval"],
    skip: ["render-report"],
  });
  expect(Reflect.get(contract, "approvalBranches")).toEqual({
    absent: ["render-report-without-mutation"],
    granted: ["execute-approved-compensated-mutation"],
  });
  expect(Reflect.get(contract, "approvedMutation")).toEqual({
    guarantee: "best-effort-compensation-not-atomic",
    interruptionLimit: "hard-interruption-has-no-output-or-recovery-guarantee",
    crashRecovery: "inspect-git-and-reconcile-before-retry",
    applyOrder: [
      "confirm-all-file-preimages",
      "apply-each-file-with-compare-and-swap",
      "validate-applied-coherent-change",
    ],
    onAnyError: [
      "reconcile-ambiguous-file-outcome",
      "compensate-applied-files-with-compare-and-swap",
      "report-unresolved-files",
      "report-failure",
    ],
    prepareOrder: [
      "select-control-surface",
      "declare-consumers",
      "require-control-oracle",
      "prepare-selected-control-surface",
      "prepare-registry",
      "capture-all-file-preimages",
    ],
    validationOrder: [
      "validate-prepared-selected-control-surface-with-owned-adapter",
      "validate-prepared-registry-with-owned-schema-and-policy",
      "validate-persisted-approval-matches-human-context",
    ],
    successOrder: ["render-report"],
  });
});

test("preserves the complete record history during retirement", async () => {
  const contract = await authoritativeContract();
  const retirement = nestedRecord(contract, "retirement");

  expect(retirement).toEqual({
    guarantee: "best-effort-compensation-not-atomic",
    interruptionLimit: "hard-interruption-has-no-output-or-recovery-guarantee",
    crashRecovery: "inspect-git-and-reconcile-before-retry",
    requiredFields: ["retiredAt", "reason"],
    optionalFields: ["replacedBy"],
    prepareOrder: [
      "require-approval",
      "lookup-existing-invariant",
      "prepare-retired-registry-copy",
      "preserve-complete-record-history-in-prepared-registry",
      "set-retired-at-in-prepared-registry",
      "set-retirement-reason-in-prepared-registry",
      "handle-optional-replaced-by-in-prepared-registry",
      "prepare-selected-control-surface-copy-if-touched",
      "capture-all-file-preimages",
    ],
    validationOrder: [
      "validate-complete-record-history-unchanged",
      "validate-prepared-selected-control-surface-if-touched-with-owned-adapter",
      "validate-prepared-retired-registry-with-owned-schema-and-policy",
      "validate-persisted-approval-matches-human-context",
    ],
    applyOrder: [
      "confirm-all-file-preimages",
      "apply-each-file-with-compare-and-swap",
      "validate-applied-coherent-change",
    ],
    onAnyError: [
      "reconcile-ambiguous-file-outcome",
      "compensate-applied-files-with-compare-and-swap",
      "report-unresolved-files",
      "report-failure",
    ],
    successOrder: ["render-report"],
  });
});

test.each([
  {
    name: "link approval route",
    path: ["decisionBranches"],
    key: "link",
    value: ["hold-session-local"],
  },
  {
    name: "granted approval route",
    path: ["approvalBranches"],
    key: "granted",
    value: ["render-report-without-mutation"],
  },
  {
    name: "absence refusal",
    path: ["approvalBranches"],
    key: "absent",
    value: ["execute-approved-compensated-mutation"],
  },
  {
    name: "owned registry validation",
    path: ["approvedMutation"],
    key: "validationOrder",
    value: ["validate-prepared-selected-control-surface-with-owned-adapter"],
  },
  {
    name: "approval matching",
    path: ["approvedMutation"],
    key: "validationOrder",
    value: [
      "validate-prepared-selected-control-surface-with-owned-adapter",
      "validate-prepared-registry-with-owned-schema-and-policy",
    ],
  },
  {
    name: "complete retirement history",
    path: ["retirement"],
    key: "validationOrder",
    value: ["validate-all-source-history-unchanged"],
  },
  {
    name: "ambiguous outcome reconciliation",
    path: ["retirement"],
    key: "onAnyError",
    value: [
      "compensate-applied-files-with-compare-and-swap",
      "report-unresolved-files",
      "report-failure",
    ],
  },
  {
    name: "per-file compare-and-swap",
    path: ["approvedMutation"],
    key: "applyOrder",
    value: [
      "apply-selected-control-surface",
      "apply-registry",
      "validate-applied-coherent-change",
    ],
  },
  {
    name: "hard interruption limit",
    path: ["retirement"],
    key: "interruptionLimit",
    value: "process-interruption-may-leave-partial-change",
  },
] as const)("rejects a mutation contract without $name", async (testCase) => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  const mutant = contractMutant(sources, testCase.path, (target): void => {
    Reflect.set(target, testCase.key, testCase.value);
  });

  expect(validateHarnessReflectionContract(mutant)).toContain(contractFinding);
});
