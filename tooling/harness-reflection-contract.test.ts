import {
  type HarnessReflectionSources,
  loadHarnessReflectionSources,
  validateHarnessReflectionContract,
} from "./harness-reflection-contract.ts";
import { expect, test } from "bun:test";
import { resolve } from "node:path";

const repositoryRoot = resolve(import.meta.dir, "..");
const contractFinding =
  "authoritative contract preserves exact workflow invariants";

const replaceRequired = (
  source: string,
  current: string,
  replacement: string,
): string => {
  expect(source).toContain(current);
  return source.replace(current, replacement);
};

const mutateReference = (
  sources: HarnessReflectionSources,
  current: string,
  replacement: string,
): HarnessReflectionSources => ({
  ...sources,
  reference: replaceRequired(sources.reference, current, replacement),
});

const expectContractRejection = (sources: HarnessReflectionSources): void => {
  expect(validateHarnessReflectionContract(sources)).toContain(contractFinding);
};

const mutateEvalQuery = (
  sources: HarnessReflectionSources,
  current: string,
  replacement: string,
): HarnessReflectionSources => {
  const serialized = replaceRequired(
    JSON.stringify(sources.evals),
    current,
    replacement,
  );
  return { ...sources, evals: JSON.parse(serialized) };
};

test("routes factual PR evidence through the named registry", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  expect(validateHarnessReflectionContract(sources)).toEqual([]);
});

test("keeps link deduplication and report scope in the consumed contract", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  expectContractRejection(
    mutateReference(
      sources,
      '    "linkEffect": "add-source-without-duplicate-record",\n',
      "",
    ),
  );
  expectContractRejection(
    mutateReference(
      sources,
      '    "appliesToDecisions": ["skip", "link", "propose"],\n',
      "",
    ),
  );
});

test("rejects a defer decision", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  expectContractRejection(
    mutateReference(
      sources,
      '"decisions": ["skip", "link", "propose"]',
      '"decisions": ["skip", "link", "propose", "defer"]',
    ),
  );
});

test("rejects a sixth registry class", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  expectContractRejection(
    mutateReference(
      sources,
      '"classes": ["not-applied", "not-loaded", "unknown", "blind-spot", "judgment"]',
      '"classes": ["not-applied", "not-loaded", "unknown", "blind-spot", "judgment", "deferred"]',
    ),
  );
});

test("rejects a sixth diagnostic class", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  const mutant = {
    ...sources,
    skill: replaceRequired(
      sources.skill,
      "or `harness-gap`. Choose",
      "or `harness-gap`, or `deferred`. Choose",
    ),
  };
  expect(validateHarnessReflectionContract(mutant)).toContain(
    "skill preserves the diagnostic classes before the harness-gap gate",
  );
});

test("rejects a sixth compatible surface", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  expectContractRejection(
    mutateReference(
      sources,
      '"enforceable": ["hook", "permission", "lint", "type", "architectural-test"]',
      '"enforceable": ["hook", "permission", "lint", "type", "architectural-test", "runtime-policy"]',
    ),
  );
});

test("rejects optional concrete proof", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  expectContractRejection(
    mutateReference(
      sources,
      '"concretePrUrls": "required"',
      '"concretePrUrls": "optional"',
    ),
  );
});

test("rejects approval denial", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  expectContractRejection(
    mutateReference(
      sources,
      '"requiredBeforeMutation": true',
      '"requiredBeforeMutation": false',
    ),
  );
});

test("rejects removal of the skill-manager route", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  expectContractRejection(
    mutateReference(sources, '    "skillChange": "skill-manager",\n', ""),
  );
});

test("rejects removal of the agent-instructions route", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  expectContractRejection(
    mutateReference(
      sources,
      '    "skillChange": "skill-manager",\n    "instructionChange": "agent-instructions"\n',
      '    "skillChange": "skill-manager"\n',
    ),
  );
});

test("rejects removal of the three consumers", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  expectContractRejection(
    mutateReference(
      sources,
      '    "required": ["claude", "codex", "cursor"],\n',
      "",
    ),
  );
});

test("rejects removal of the oracle requirement", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  expectContractRejection(
    mutateReference(sources, '    "requiredAfterApproval": true,\n', ""),
  );
});

test("rejects removal of retirement fields", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  expectContractRejection(
    mutateReference(
      sources,
      '    "requiredFields": ["retiredAt", "reason"],\n',
      "",
    ),
  );
});

test("rejects delayed CLI verification", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  expectContractRejection(
    mutateReference(
      sources,
      '"timing": "immediately-before-report"',
      '"timing": "eventually-before-report"',
    ),
  );
});

test("rejects contradictory prose outside the consumed contract", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  const contradictory = {
    ...sources,
    reference: `${sources.reference}\nExplicit approval is not required.`,
  };
  expect(validateHarnessReflectionContract(contradictory)).toContain(
    "reference contains no parallel or contradictory authority",
  );
});

test("rejects removal of the harness-gap contract route", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  const missingRoute = {
    ...sources,
    skill: replaceRequired(
      sources.skill,
      "execute its single authoritative JSON contract in the declared order",
      "read the reference when useful",
    ),
  };
  expect(validateHarnessReflectionContract(missingRoute)).toContain(
    "skill consumes the authoritative contract in order",
  );
});

test("rejects an inexact positive registry-link query", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  const mutant = mutateEvalQuery(
    sources,
    "Relie ces constats pr-feedback récurrents à l'invariant existant sans le dupliquer",
    "Relie ces constats à une règle",
  );
  expect(validateHarnessReflectionContract(mutant)).toContain(
    "evals preserve the exact positive registry-link query",
  );
});

test("rejects an inexact negative stylistic query", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  const mutant = mutateEvalQuery(
    sources,
    "J'ai une préférence stylistique isolée pour ce nommage, change-le",
    "Je préfère ce nommage, change-le",
  );
  expect(validateHarnessReflectionContract(mutant)).toContain(
    "evals preserve the exact negative stylistic query",
  );
});
