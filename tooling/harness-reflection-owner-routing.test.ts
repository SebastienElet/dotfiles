import { expect, test } from "bun:test";
import {
  loadHarnessReflectionSources,
  parseHarnessReflectionContract,
} from "./harness-reflection-contract.ts";
import { invariantSurfaces } from "./invariant-registry-schema.ts";
import { resolve } from "node:path";
import { surfaceRoutes } from "./harness-reflection-mutation-surfaces.ts";

const repositoryRoot = resolve(import.meta.dir, "..");
type Contract = ReturnType<typeof parseHarnessReflectionContract>;
const expectedSurfaceOwners: Contract["surfaceOwners"] = {
  "always-loaded-instruction": {
    owner: "agent-instructions",
    path: "harness/AGENTS.md",
    verification: "agent-instructions-contracts",
  },
  "conditional-skill": {
    owner: "skill-manager",
    path: "harness/skills/harness-reflection/SKILL.md",
    verification: "skill-manager-doctor-and-contracts",
  },
};
const expectedExternalRoutes: Contract["externalControlRoutes"] = {
  application:
    "owner-specific-exact-diff-and-contract-before-registry-recording",
  genericManifestValidator: "not-applicable",
  surfaces: {
    "architectural-test": ["enforcement-code"],
    hook: ["scripts", "enforcement-code"],
    lint: ["scripts", "enforcement-code"],
    permission: ["enforcement-code"],
    type: ["enforcement-code"],
  },
};
const expectedChangeOrder: Contract["approvedChangeOrder"] = {
  registryOnly: [
    "prepare-link-proposal",
    "prepare-exact-registry-diff",
    "present-exact-manifest-for-contextual-human-approval",
    "validate-approved-manifest",
    "write-approved-registry-replacement-only",
    "run-registry-cli-and-declared-oracles",
    "render-report",
  ],
  surfaceAndRegistry: [
    "select-and-propose-control-surface",
    "prepare-exact-surface-and-registry-diff",
    "present-exact-manifest-for-contextual-human-approval",
    "apply-surface-with-required-owner",
    "run-required-owner-doctor-and-contracts",
    "validate-approved-manifest-and-applied-surface",
    "write-approved-registry-replacement-only",
    "run-registry-cli-and-declared-oracles",
    "render-report",
  ],
};

test("routes surface application through its required owner before registry recording", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  const contract = parseHarnessReflectionContract(sources.reference);

  expect(Reflect.get(contract, "surfaceOwners")).toEqual(expectedSurfaceOwners);
  expect(Reflect.get(contract, "externalControlRoutes")).toEqual(
    expectedExternalRoutes,
  );
  expect(Reflect.get(contract, "approvedChangeOrder")).toEqual(
    expectedChangeOrder,
  );
  expect(Reflect.has(contract, "mutationExecution")).toBeFalse();
});

test("routes only to a read-only manifest validator", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  const contract = parseHarnessReflectionContract(sources.reference);
  const routes = Reflect.get(contract, "workflowRoutes");

  expect(routes).toEqual({
    manifestValidation: {
      export: "validateAppliedHarnessMutation",
      module: "tooling/harness-reflection-mutation-validation.ts",
    },
    registryValidation: {
      command: "bun tooling/invariant-registry-cli.ts",
    },
  });
});

test("does not ship the generic surface-writing engine", async () => {
  const writerModules = [
    "harness-reflection-mutation-atomic-file.ts",
    "harness-reflection-mutation-filesystem.ts",
    "harness-reflection-mutation-lock.ts",
    "harness-reflection-mutation-staged-files.ts",
    "harness-reflection-mutation-workflow-compensation.ts",
    "harness-reflection-mutation-workflow.ts",
  ];

  for (const module of writerModules) {
    expect(
      await Bun.file(resolve(import.meta.dir, module)).exists(),
    ).toBeFalse();
  }
});

test("routes every registry surface to at least one real owner", () => {
  expect(Object.keys(surfaceRoutes)).toEqual([...invariantSurfaces]);
  for (const surface of invariantSurfaces) {
    expect(surfaceRoutes[surface].owners.length).toBeGreaterThan(0);
  }
  expect(surfaceRoutes["project-local-contract"].owners).toEqual([
    "agent-instructions",
  ]);
});
