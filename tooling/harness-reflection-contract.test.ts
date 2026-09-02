import {
  type HarnessReflectionSources,
  loadHarnessReflectionSources,
  validateHarnessReflectionContract,
} from "./harness-reflection-contract.ts";
import {
  cspellWorkflowFindings,
  invariantRegistryCspellPaths,
} from "./harness-reflection-cspell-contract.ts";
import { expect, test } from "bun:test";
import { mutateContract } from "./harness-reflection-contract-test-support.ts";
import { resolve } from "node:path";

const repositoryRoot = resolve(import.meta.dir, "..");
const lintWorkflowPath = resolve(repositoryRoot, ".github/workflows/lint.yml");
const contractFinding =
  "authoritative contract preserves exact workflow invariants";

const contractMutant = (
  sources: HarnessReflectionSources,
  path: readonly string[],
  mutate: (target: Readonly<Record<string, unknown>>) => void,
): HarnessReflectionSources => ({
  ...sources,
  reference: mutateContract(sources.reference, path, mutate),
});

const expectContractRejection = (sources: HarnessReflectionSources): void => {
  expect(validateHarnessReflectionContract(sources)).toContain(contractFinding);
};

test("routes factual PR evidence through the named registry", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  expect(validateHarnessReflectionContract(sources)).toEqual([]);
});

test("checks invariant registry sources with CSpell", async () => {
  const lintWorkflow = await Bun.file(lintWorkflowPath).text();
  expect(cspellWorkflowFindings(lintWorkflow)).toEqual([]);
});

test("rejects an invariant registry path moved outside CSpell lint", async () => {
  const lintWorkflow = await Bun.file(lintWorkflowPath).text();
  const registryPathLine = "            harness/invariants/registry.json \\\n";
  const mutant = `${lintWorkflow.replace(registryPathLine, "")}\n# ${invariantRegistryCspellPaths[3]}\n`;

  expect(cspellWorkflowFindings(mutant)).toContain(
    "CSpell lint command includes invariant registry sources",
  );
});

test("rejects a CSpell trace canary moved outside its YAML block", async () => {
  const lintWorkflow = await Bun.file(lintWorkflowPath).text();
  const frenchDictionaryGrep =
    '          printf \'%s\\n\' "$trace" | grep -F "@cspell/dict-fr-fr"\n';
  const mutant = `${lintWorkflow.replace(frenchDictionaryGrep, "")}# ${frenchDictionaryGrep.trim()}\n`;

  expect(cspellWorkflowFindings(mutant)).toContain(
    "CSpell trace confirms the French dictionary",
  );
});

test("keeps link deduplication and report scope", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  expectContractRejection(
    contractMutant(
      sources,
      ["registry"],
      (registry: Readonly<Record<string, unknown>>): void => {
        Reflect.deleteProperty(registry, "linkEffect");
      },
    ),
  );
  expectContractRejection(
    contractMutant(
      sources,
      ["report"],
      (report: Readonly<Record<string, unknown>>): void => {
        Reflect.deleteProperty(report, "appliesToDecisions");
      },
    ),
  );
});

test("rejects a defer decision", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  expectContractRejection(
    contractMutant(
      sources,
      ["registry"],
      (registry: Readonly<Record<string, unknown>>): void => {
        Reflect.set(registry, "decisions", [
          "skip",
          "link",
          "propose",
          "defer",
        ]);
      },
    ),
  );
});

test("rejects a sixth registry class", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  expectContractRejection(
    contractMutant(
      sources,
      ["registry"],
      (registry: Readonly<Record<string, unknown>>): void => {
        Reflect.set(registry, "classes", [
          "not-applied",
          "not-loaded",
          "unknown",
          "blind-spot",
          "judgment",
          "deferred",
        ]);
      },
    ),
  );
});

test("rejects a sixth diagnostic class", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  expectContractRejection(
    contractMutant(
      sources,
      ["diagnostic"],
      (diagnostic: Readonly<Record<string, unknown>>): void => {
        Reflect.set(diagnostic, "classes", [
          "task-specific",
          "owned-defect",
          "external-transient",
          "missing-capability",
          "harness-gap",
          "deferred",
        ]);
      },
    ),
  );
});

test("rejects a sixth compatible surface", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  expectContractRejection(
    contractMutant(
      sources,
      ["controls"],
      (controls: Readonly<Record<string, unknown>>): void => {
        Reflect.set(controls, "enforceable", [
          "hook",
          "permission",
          "lint",
          "type",
          "architectural-test",
          "runtime-policy",
        ]);
      },
    ),
  );
});

test.each([
  [
    "optional concrete proof",
    { key: "concretePrUrls", path: ["evidence"], value: "optional" },
  ],
  [
    "approval denial",
    { key: "requiredBeforeMutation", path: ["approval"], value: false },
  ],
  [
    "delayed CLI verification",
    { key: "timing", path: ["cli"], value: "eventually-before-report" },
  ],
] as const)("rejects %s", async (_name, mutation) => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  expectContractRejection(
    contractMutant(
      sources,
      mutation.path,
      (target: Readonly<Record<string, unknown>>): void => {
        Reflect.set(target, mutation.key, mutation.value);
      },
    ),
  );
});

test.each([
  ["skill-manager route", { key: "skillChange", path: ["routes"] }],
  ["agent-instructions route", { key: "instructionChange", path: ["routes"] }],
  ["three consumers", { key: "required", path: ["consumers"] }],
  ["oracle requirement", { key: "requiredAfterApproval", path: ["oracle"] }],
  ["retirement fields", { key: "requiredFields", path: ["retirement"] }],
] as const)("rejects removal of %s", async (_name, mutation) => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  expectContractRejection(
    contractMutant(
      sources,
      mutation.path,
      (target: Readonly<Record<string, unknown>>): void => {
        Reflect.deleteProperty(target, mutation.key);
      },
    ),
  );
});

test("rejects contradictory prose outside the contract", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  const mutant = {
    ...sources,
    reference: `${sources.reference}\nExplicit approval is not required.\n`,
  };
  expect(validateHarnessReflectionContract(mutant)).toContain(
    "reference contains no parallel or contradictory authority",
  );
});

test("rejects removal of the skill router", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  const mutant = {
    ...sources,
    skill: sources.skill.replace(
      "[references/invariant-registry.md](references/invariant-registry.md)",
      "the registry reference",
    ),
  };
  expect(validateHarnessReflectionContract(mutant)).toContain(
    "skill contains only the closed router surface",
  );
});
