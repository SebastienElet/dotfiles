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

test("requires controlled marginal ablation for probabilistic promotion", async () => {
  const contract = await authoritativeContract();
  const controls = nestedRecord(contract, "controls");

  expect(Reflect.get(controls, "probabilisticPromotion")).toEqual({
    activationMeasurementForConditionalSkill: "required",
    conditions: ["with-exact-candidate-text", "without-candidate-text"],
    controlledConstants: ["scenarios", "environments", "replicates"],
    observableDelta: "required",
    protocol: "controlled-marginal-ablation",
    withOnlyRuns: "never-sufficient",
  });
});

test.each([
  [
    "removes the without condition",
    "conditions",
    ["with-exact-candidate-text"],
  ],
  [
    "drops controlled replicates",
    "controlledConstants",
    ["scenarios", "environments"],
  ],
  ["makes the delta optional", "observableDelta", "optional"],
  ["accepts with-only runs", "withOnlyRuns", "sufficient"],
  [
    "skips conditional-skill activation measurement",
    "activationMeasurementForConditionalSkill",
    "optional",
  ],
] as const)("rejects ablation that %s", async (_name, key, value) => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  const mutant = contractMutant(
    sources,
    ["controls", "probabilisticPromotion"],
    (promotion): void => {
      Reflect.set(promotion, key, value);
    },
  );

  expect(validateHarnessReflectionContract(mutant)).toContain(contractFinding);
});

test("makes approved link and propose atomic mutation reachable", async () => {
  const contract = await authoritativeContract();

  expect(Reflect.get(contract, "decisionBranches")).toEqual({
    link: ["hold-session-local", "await-explicit-approval"],
    propose: ["hold-session-local", "await-explicit-approval"],
    skip: ["render-report"],
  });
  expect(Reflect.get(contract, "approvalBranches")).toEqual({
    absent: ["render-report-without-mutation"],
    granted: ["execute-approved-atomic-mutation"],
  });
  expect(Reflect.get(contract, "approvedMutation")).toEqual({
    applyOrder: [
      "apply-selected-control-surface-and-registry-as-coherent-change",
      "validate-applied-coherent-change",
    ],
    onAnyError: ["restore-all-touched-files", "report-failure"],
    prepareOrder: [
      "select-control-surface",
      "declare-consumers",
      "require-control-oracle",
      "prepare-selected-control-surface",
      "prepare-registry",
    ],
    validationOrder: [
      "validate-prepared-selected-control-surface",
      "validate-prepared-registry-with-cli-on-temporary-copy",
    ],
    successOrder: ["render-report"],
  });
});

test.each([
  {
    name: "disconnects link from approval",
    path: ["decisionBranches"],
    key: "link",
    value: ["hold-session-local"],
  },
  {
    name: "disconnects granted approval from mutation",
    path: ["approvalBranches"],
    key: "granted",
    value: ["render-report-without-mutation"],
  },
  {
    name: "mutates without approval",
    path: ["approvalBranches"],
    key: "absent",
    value: ["execute-approved-mutation-order"],
  },
  {
    name: "skips prepared registry CLI prevalidation",
    path: ["approvedMutation"],
    key: "validationOrder",
    value: ["validate-prepared-selected-control-surface"],
  },
  {
    name: "fails to compensate every touched file",
    path: ["approvedMutation"],
    key: "onAnyError",
    value: ["report-failure"],
  },
  {
    name: "applies the surface and registry separately",
    path: ["approvedMutation"],
    key: "applyOrder",
    value: [
      "apply-selected-control-surface",
      "apply-registry",
      "validate-applied-coherent-change",
    ],
  },
] as const)("rejects a workflow graph that $name", async (testCase) => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  const mutant = contractMutant(sources, testCase.path, (target): void => {
    Reflect.set(target, testCase.key, testCase.value);
  });

  expect(validateHarnessReflectionContract(mutant)).toContain(contractFinding);
});

test("makes history-preserving retirement reachable", async () => {
  const contract = await authoritativeContract();
  const retirement = nestedRecord(contract, "retirement");

  expect(Reflect.get(retirement, "workflowOrder")).toEqual([
    "require-approval",
    "lookup-existing-invariant",
    "preserve-history",
    "set-retired-at",
    "set-retirement-reason",
    "handle-optional-replaced-by",
    "mutate-registry",
    "run-cli",
    "render-report",
  ]);
});

test.each(["preserve-history", "mutate-registry"] as const)(
  "rejects retirement without %s",
  async (step) => {
    const sources = await loadHarnessReflectionSources(repositoryRoot);
    const mutant = contractMutant(
      sources,
      ["retirement"],
      (retirement): void => {
        const order = Reflect.get(retirement, "workflowOrder");
        if (!Array.isArray(order)) {
          throw new TypeError("retirement workflow missing");
        }
        Reflect.set(
          retirement,
          "workflowOrder",
          order.filter((candidate) => candidate !== step),
        );
      },
    );

    expect(validateHarnessReflectionContract(mutant)).toContain(
      contractFinding,
    );
  },
);

test("closes the factual pr-feedback input boundary", async () => {
  const contract = await authoritativeContract();
  const evidence = nestedRecord(contract, "evidence");

  expect(Reflect.get(evidence, "prFeedbackBoundary")).toEqual({
    collectionRole: "none",
    directForgeIngestion: "forbidden",
    historicalReconstruction: "forbidden",
    input: "provided-factual-report-only",
  });
});

test.each([
  ["directForgeIngestion", "allowed"],
  ["historicalReconstruction", "allowed"],
  ["collectionRole", "collector"],
] as const)("rejects weakened pr-feedback boundary %s", async (key, value) => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  const mutant = contractMutant(
    sources,
    ["evidence", "prFeedbackBoundary"],
    (boundary): void => {
      Reflect.set(boundary, key, value);
    },
  );

  expect(validateHarnessReflectionContract(mutant)).toContain(contractFinding);
});
