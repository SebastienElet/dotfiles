import { expect, test } from "bun:test";
import {
  loadHarnessReflectionSources,
  validateHarnessReflectionContract,
} from "./harness-reflection-contract.ts";
import { resolve } from "node:path";

const repositoryRoot = resolve(import.meta.dir, "..");
const routerFinding = "skill preserves the closed router contract";

test("does not treat the whole skill hash as the router contract", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  const findings = validateHarnessReflectionContract({
    ...sources,
    skill: `${sources.skill}\n`,
  });

  expect(findings).not.toContain(routerFinding);
  expect(findings).toContain(
    "promotion workflow results match the skill and reference digest",
  );
});

test("rejects removal of the owner manifest-validation route", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  const mutant = {
    ...sources,
    skill: sources.skill.replace(
      "Then resolve `workflowRoutes.manifestValidation`",
      "Then continue the workflow",
    ),
  };

  expect(validateHarnessReflectionContract(mutant)).toContain(routerFinding);
});
