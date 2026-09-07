import {
  type HarnessReflectionSources,
  loadHarnessReflectionSources,
  validateHarnessReflectionContract,
} from "./harness-reflection-contract.ts";
import {
  duplicateContractKey,
  mutateContract,
} from "./harness-reflection-contract-test-support.ts";
import { expect, test } from "bun:test";
import { resolve } from "node:path";

const repositoryRoot = resolve(import.meta.dir, "..");
type MutableEvals = Readonly<{
  evals: object;
  queries: unknown[];
}>;

const mutableEvals = (sources: HarnessReflectionSources): MutableEvals => {
  const evals: unknown = structuredClone(sources.evals);
  if (typeof evals !== "object" || evals === null) {
    throw new TypeError("eval queries missing");
  }
  const queryValue: unknown = Reflect.get(evals, "queries");
  if (!Array.isArray(queryValue)) {
    throw new TypeError("eval queries missing");
  }
  return {
    evals,
    queries: queryValue.map((query: unknown): unknown => query),
  };
};

const mutateEvalQuery = (
  sources: HarnessReflectionSources,
  queryText: string,
  replacement: string,
): unknown => {
  const { evals, queries } = mutableEvals(sources);
  const query: unknown = queries.find(
    (candidate) =>
      typeof candidate === "object" &&
      candidate !== null &&
      Reflect.get(candidate, "query") === queryText,
  );
  if (typeof query !== "object" || query === null) {
    throw new TypeError("eval query missing");
  }
  Reflect.set(query, "query", replacement);
  Reflect.set(evals, "queries", queries);
  return evals;
};

test("rejects a shadowed duplicate requiredBeforeMutation key", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  const mutant = {
    ...sources,
    reference: duplicateContractKey(sources.reference, {
      key: "requiredBeforeMutation",
      path: ["approval"],
      shadowedValue: false,
    }),
  };

  expect(validateHarnessReflectionContract(mutant)).toContain(
    "authoritative contract has unique JSON object keys",
  );
});

test("rejects a shadowed duplicate decisions key", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  const mutant = {
    ...sources,
    reference: duplicateContractKey(sources.reference, {
      key: "decisions",
      path: ["registry"],
      shadowedValue: ["defer"],
    }),
  };

  expect(validateHarnessReflectionContract(mutant)).toContain(
    "authoritative contract has unique JSON object keys",
  );
});

test("rejects approval-denying prose added to the skill", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  const mutant = {
    ...sources,
    skill: `${sources.skill}\nExplicit approval is not required.\n`,
  };

  expect(validateHarnessReflectionContract(mutant)).toContain(
    "skill preserves the closed router contract",
  );
});

test("rejects duplicate eval text with the opposite polarity", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  const { evals, queries } = mutableEvals(sources);
  queries.push({
    query:
      "Relie ces constats pr-feedback récurrents à l'invariant existant sans le dupliquer",
    reason: "opposite polarity mutant",
    should_activate: false,
  });
  Reflect.set(evals, "queries", queries);

  expect(validateHarnessReflectionContract({ ...sources, evals })).toContain(
    "eval queries have unique text and polarity",
  );
});

test.each([
  [
    "positive registry-link",
    {
      finding: "evals preserve the exact positive registry-link query",
      query:
        "Relie ces constats pr-feedback récurrents à l'invariant existant sans le dupliquer",
      replacement: "Relie ces constats à une règle",
    },
  ],
  [
    "negative stylistic",
    {
      finding: "evals preserve the exact negative stylistic query",
      query:
        "J'ai une préférence stylistique isolée pour ce nommage, change-le",
      replacement: "Je préfère ce nommage, change-le",
    },
  ],
] as const)("rejects an inexact %s query", async (_name, mutation) => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  const evals = mutateEvalQuery(sources, mutation.query, mutation.replacement);
  expect(validateHarnessReflectionContract({ ...sources, evals })).toContain(
    mutation.finding,
  );
});

test("structured mutants survive Markdown wrapping changes", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  const reformatted = sources.reference.replaceAll(", ", ",\n      ");
  const mutant = {
    ...sources,
    reference: mutateContract(
      reformatted,
      ["approval"],
      (approval: Readonly<Record<string, unknown>>): void => {
        Reflect.set(approval, "requiredBeforeMutation", false);
      },
    ),
  };

  expect(validateHarnessReflectionContract(mutant)).toContain(
    "authoritative contract preserves exact workflow invariants",
  );
});
