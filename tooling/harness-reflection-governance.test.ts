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
