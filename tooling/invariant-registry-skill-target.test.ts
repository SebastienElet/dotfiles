import {
  active,
  diagnosticCodes,
  marginalAblation,
  registry,
  validateInvariantRegistry,
  validationOptions,
  verifiedVerification,
} from "./invariant-registry-test-support.ts";
import { expect, test } from "bun:test";
import type { SkillTargetInspection } from "./invariant-registry-contract.ts";

const effectiveAblation = {
  ...marginalAblation,
  conditionalSkillActivation: {
    with: { activated: 6, total: 6 },
    without: { activated: 0, total: 6 },
  },
};
const validInspection: SkillTargetInspection = {
  deploymentManifestValid: true,
  descriptionTriggerable: true,
  frontmatterValid: true,
  installedFor: ["claude", "codex", "cursor"],
  kind: "regular-file",
  name: "enforcement-code",
  tracked: true,
};

const diagnosticsFor = (
  inspection: SkillTargetInspection,
  overrides: Readonly<Record<string, unknown>> = {},
): readonly string[] =>
  diagnosticCodes(
    validateInvariantRegistry(
      registry(
        active({
          controlKind: "probabilistic",
          marginalAblation: effectiveAblation,
          oracle: undefined,
          surface: "conditional-skill",
          verification: verifiedVerification,
          ...overrides,
        }),
      ),
      validationOptions(undefined, () => inspection),
    ),
  );

test("accepts an existing triggerable skill deployed to every declared consumer", () => {
  expect(diagnosticsFor(validInspection)).toEqual([]);
});

test("rejects a missing conditional skill target", () => {
  expect(
    diagnosticsFor({
      ...validInspection,
      kind: "missing",
      tracked: false,
    }),
  ).toContain("conditional-skill-target-missing");
});

test.each([
  [
    "non-regular target",
    { ...validInspection, kind: "non-regular" },
    "conditional-skill-target-not-regular",
  ],
  [
    "untracked target",
    { ...validInspection, tracked: false },
    "conditional-skill-target-untracked",
  ],
] as const)("rejects %s", (_name, inspection, expected) => {
  expect(diagnosticsFor(inspection)).toContain(expected);
});

test("rejects the closed harness-reflection router as its own target", () => {
  expect(
    diagnosticsFor(
      { ...validInspection, name: "harness-reflection" },
      {
        targetSkillPath: "harness/skills/harness-reflection/SKILL.md",
      },
    ),
  ).toContain("conditional-skill-self-target");
});

test.each([
  [
    "invalid frontmatter",
    { ...validInspection, frontmatterValid: false },
    "conditional-skill-frontmatter-invalid",
  ],
  [
    "wrong frontmatter name",
    { ...validInspection, name: "different-skill" },
    "conditional-skill-name-mismatch",
  ],
  [
    "non-triggerable description",
    { ...validInspection, descriptionTriggerable: false },
    "conditional-skill-not-triggerable",
  ],
] as const)("rejects %s", (_name, inspection, expected) => {
  expect(diagnosticsFor(inspection)).toContain(expected);
});

test("rejects user-skill support absent from the declared deployment", () => {
  expect(
    diagnosticsFor({
      ...validInspection,
      installedFor: ["claude", "codex"],
    }),
  ).toContain("conditional-skill-consumer-not-deployed");
});

test("rejects an unreadable deployment manifest", () => {
  expect(
    diagnosticsFor({
      ...validInspection,
      deploymentManifestValid: false,
    }),
  ).toContain("conditional-skill-deployment-invalid");
});
