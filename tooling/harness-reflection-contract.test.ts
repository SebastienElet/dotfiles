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
    "skill flow is ordered from diagnosis to registry decision",
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
  ).toContain("evals reject an incorrect registry-link query");
  expect(
    validateHarnessReflectionContract({
      ...sources,
      evals: { version: "1.1" },
    }),
  ).toContain("evals have valid structured queries");
});

test("rejects decision, registry-class, and surface mutants", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  const deferredDecision = {
    ...sources,
    reference: sources.reference.replace(
      "| Decisions | `skip`, `link`, `propose` |",
      "| Decisions | `skip`, `link`, `defer` |",
    ),
  };
  const sixthClass = {
    ...sources,
    reference: sources.reference.replace(
      "`not-applied`, `not-loaded`, `unknown`, `blind-spot`, `judgment`",
      "`not-applied`, `not-loaded`, `unknown`, `blind-spot`, `judgment`, `extra`",
    ),
  };
  const invalidSurface = {
    ...sources,
    reference: sources.reference.replace("`architectural-test`", "`defer`"),
  };
  expect(validateHarnessReflectionContract(deferredDecision)).toContain(
    "guidance closes the decision set",
  );
  expect(validateHarnessReflectionContract(sixthClass)).toContain(
    "guidance closes the registry-class set",
  );
  expect(validateHarnessReflectionContract(invalidSurface)).toContain(
    "guidance closes compatible control surfaces",
  );
});

test("rejects optional proof and an incorrect eval slug", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  const optionalProof = {
    ...sources,
    reference: sources.reference.replace(
      "concrete PR URLs required; missing evidence returns `skip`",
      "concrete PR URLs optional",
    ),
  };
  const invalidEvals = {
    ...sources,
    evals: { skill: "wrong-slug", version: "1.1", queries: [] },
  };
  const invalidQuery = {
    ...sources,
    evals: {
      skill: "harness-reflection",
      version: "1.1",
      queries: [
        { query: "wrong query", should_activate: true, reason: "mutation" },
        {
          query:
            "J'ai une préférence stylistique isolée pour ce nommage, change-le",
          should_activate: false,
          reason: "mutation",
        },
      ],
    },
  };

  expect(validateHarnessReflectionContract(optionalProof)).toContain(
    "reference requires concrete evidence before propose",
  );
  expect(validateHarnessReflectionContract(invalidEvals)).toContain(
    "evals use the harness-reflection slug",
  );
  expect(validateHarnessReflectionContract(invalidQuery)).toContain(
    "evals reject an incorrect registry-link query",
  );
});
