import { expect, test } from "bun:test";
import {
  loadHarnessReflectionSources,
  parseHarnessReflectionContract,
} from "./harness-reflection-contract.ts";
import { resolve } from "node:path";

const repositoryRoot = resolve(import.meta.dir, "..");

test("records approval attestations without claiming origin authentication", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  const contract = parseHarnessReflectionContract(sources.reference);

  expect(contract.approval).toEqual({
    agentSelfAssertion: "procedurally-forbidden-not-machine-detectable",
    authentication: "not-performed",
    codeAcceptance: "exact-approval-attestation-without-origin-authentication",
    manifestContents: [
      "exact-paths",
      "exact-preimages",
      "exact-replacements",
      "target-invariant-id",
      "exact-target-before-and-after",
    ],
    manifestRequired: true,
    manifestTiming: "present-exact-manifest-before-approval",
    preApprovalState: "session-local",
    proceduralPrecondition: "contextual-human-approval-before-attestation",
    registryRecordMeaning: "recorded-attestation-not-independent-proof",
    requiredBeforeMutation: true,
    timePressureBypass: false,
    transitionKind: "derived-from-exact-target-before-and-after",
  });
});

test("routes lifecycle validation without exposing a surface writer", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  const contract = parseHarnessReflectionContract(sources.reference);

  expect(contract.workflowRoutes).toEqual({
    manifestValidation: {
      export: "validateAppliedHarnessMutation",
      module: "tooling/harness-reflection-mutation-validation.ts",
    },
    registryValidation: {
      command: "bun tooling/invariant-registry-cli.ts",
    },
  });
  expect(contract.lifecycle).toEqual({
    allowedTransitions: [
      "new-to-candidate",
      "new-to-active",
      "candidate-to-candidate",
      "candidate-to-active",
      "active-to-active",
      "active-to-retired",
    ],
    independentWithOnlySessions: "never-sufficient-for-probabilistic-control",
    promotion: "control-kind-specific-green-oracle-required",
    retiredTerminal: true,
    rollback: ["two-failed-trials", "one-safety-regression", "user-veto"],
  });
});

test("binds mutable surfaces to exact paths and consumer adapters", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  const contract = parseHarnessReflectionContract(sources.reference);

  expect(contract.consumers).toEqual({
    declaration: "independent-supported-mechanism-or-unsupported-reason",
    mutationTargets: {
      alwaysLoadedInstruction: {
        consumers: {
          claude: "claude-global-instruction",
          codex: "codex-global-instruction",
          cursor: "unsupported",
        },
        path: "harness/AGENTS.md",
        surface: "always-loaded-instruction",
      },
      conditionalSkill: {
        consumers: {
          claude: "claude-user-skill",
          codex: "codex-user-skill",
          cursor: "cursor-user-skill",
        },
        path: "harness/invariants/registry.json",
        surface: "conditional-skill",
      },
      projectLocalContract: {
        consumers: {
          claude: "unsupported",
          codex: "unsupported",
          cursor: "unsupported",
        },
        path: "AGENTS.md",
        surface: "project-local-contract",
      },
    },
    required: ["claude", "codex", "cursor"],
    supportedMechanisms: {
      claude: ["claude-global-instruction", "claude-user-skill"],
      codex: ["codex-global-instruction", "codex-user-skill"],
      cursor: ["cursor-user-skill"],
    },
  });
});
