import { expect, test } from "bun:test";
import {
  loadHarnessReflectionSources,
  validateHarnessReflectionContract,
} from "./harness-reflection-contract.ts";
import { resolve } from "node:path";

const repositoryRoot = resolve(import.meta.dir, "..");

test("routes factual PR evidence through the named registry", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  expect(validateHarnessReflectionContract(sources)).toEqual([]);
});

test("rejects missing, negated, and reordered registry-flow requirements", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  const missingHarnessGap = {
    ...sources,
    skill: sources.skill.replaceAll("harness-gap", "missing-gap"),
  };
  const negatedDecision = {
    ...sources,
    reference: sources.reference.replace(
      "Return exactly one decision:",
      "A decision may be returned:",
    ),
  };
  const reorderedFlow = {
    ...sources,
    skill: sources.skill.replace(
      "3. Classify the cause as",
      "6. For `harness-gap`, read",
    ),
  };

  expect(validateHarnessReflectionContract(missingHarnessGap)).toContain(
    "skill flow preserves the diagnostic class and harness-gap gate",
  );
  expect(validateHarnessReflectionContract(negatedDecision)).toContain(
    "reference returns exactly skip, link, or propose",
  );
  expect(validateHarnessReflectionContract(reorderedFlow)).toContain(
    "skill flow is ordered from diagnosis to registry decision",
  );
});

test("rejects an invalid eval shape and a missing link case", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);

  expect(
    validateHarnessReflectionContract({
      ...sources,
      evals: { version: "1.1" },
    }),
  ).toContain("evals contain a registry-link case");
  expect(
    validateHarnessReflectionContract({
      ...sources,
      evals: { version: "1.1" },
    }),
  ).toContain("evals have valid structured queries");
});
