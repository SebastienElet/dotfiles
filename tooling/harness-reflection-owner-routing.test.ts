import { candidate, registry } from "./invariant-registry-test-support.ts";
import {
  consumerSurfaceDiagnostics,
  resolveSupportedTarget,
  surfaceRoutes,
} from "./harness-reflection-mutation-surfaces.ts";
import { expect, test } from "bun:test";
import {
  loadHarnessReflectionSources,
  parseHarnessReflectionContract,
} from "./harness-reflection-contract.ts";
import { invariantSurfaces } from "./invariant-registry-schema.ts";
import { resolve } from "node:path";

const repositoryRoot = resolve(import.meta.dir, "..");
type Contract = ReturnType<typeof parseHarnessReflectionContract>;
type Surface = keyof typeof surfaceRoutes;
type RouteOwners = (typeof surfaceRoutes)[Surface]["owners"];
const expectedSurfaceOwners: Contract["surfaceOwners"] = {
  "always-loaded-instruction": {
    owner: "agent-instructions",
    path: "harness/AGENTS.md",
    verification: "agent-instructions-contracts",
  },
  "conditional-skill": {
    owner: "skill-manager",
    pathFromRecord: "targetSkillPath",
    verification: "skill-manager-doctor-and-contracts",
  },
  "project-local-contract": {
    owner: "agent-instructions",
    path: "AGENTS.md",
    verification: "agent-instructions-contracts",
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
    "prepare-registry-only-proposal",
    "prepare-exact-registry-diff",
    "present-exact-manifest-for-contextual-human-approval",
    "validate-approved-manifest",
    "write-approved-registry-replacement-only",
    "run-registry-cli-and-declared-oracles",
    "render-report",
  ],
  surfaceAndRegistry: [
    "select-and-propose-control-surface",
    "prepare-route-specific-exact-manifest",
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

test("keeps every schema surface aligned across contract and route catalog", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  const contract = parseHarnessReflectionContract(sources.reference);
  const contractOwners: Readonly<Partial<Record<Surface, RouteOwners>>> = {
    "always-loaded-instruction": [
      contract.surfaceOwners["always-loaded-instruction"].owner,
    ],
    "architectural-test":
      contract.externalControlRoutes.surfaces["architectural-test"],
    "conditional-skill": [contract.surfaceOwners["conditional-skill"].owner],
    hook: contract.externalControlRoutes.surfaces.hook,
    lint: contract.externalControlRoutes.surfaces.lint,
    permission: contract.externalControlRoutes.surfaces.permission,
    "project-local-contract": [
      contract.surfaceOwners["project-local-contract"].owner,
    ],
    type: contract.externalControlRoutes.surfaces.type,
  };
  const missing = invariantSurfaces.filter(
    (surface) => contractOwners[surface] === undefined,
  );

  expect(missing).toEqual([]);
  for (const surface of invariantSurfaces) {
    const owners = contractOwners[surface];
    if (owners === undefined) {
      throw new Error(`missing-owner-route:${surface}`);
    }
    expect(surfaceRoutes[surface].owners).toEqual(owners);
  }
});

test("resolves the exact project-local instruction target", () => {
  const [record] = registry(
    candidate({
      consumers: {
        claude: {
          reason: "No declared project adapter.",
          state: "unsupported",
        },
        codex: {
          reason: "No declared project adapter.",
          state: "unsupported",
        },
        cursor: {
          reason: "No declared project adapter.",
          state: "unsupported",
        },
      },
      surface: "project-local-contract",
    }),
  ).invariants;
  if (record === undefined) {
    throw new Error("project-local-record-missing");
  }

  expect(consumerSurfaceDiagnostics(record, "invariants.0")).toEqual([]);
  expect(resolveSupportedTarget(record)).toEqual({
    consumers: {},
    owner: "agent-instructions",
    path: "AGENTS.md",
  });
});

test("resolves a conditional skill target from its strict record field", () => {
  const [record] = registry(
    candidate({
      consumers: {
        claude: { mechanism: "claude-user-skill", state: "supported" },
        codex: { mechanism: "codex-user-skill", state: "supported" },
        cursor: { mechanism: "cursor-user-skill", state: "supported" },
      },
      surface: "conditional-skill",
      targetSkillPath: "harness/skills/enforcement-code/SKILL.md",
    }),
  ).invariants;
  if (record === undefined) {
    throw new Error("conditional-record-missing");
  }

  expect(resolveSupportedTarget(record)).toEqual({
    consumers: {
      claude: "claude-user-skill",
      codex: "codex-user-skill",
      cursor: "cursor-user-skill",
    },
    owner: "skill-manager",
    path: "harness/skills/enforcement-code/SKILL.md",
  });
});
