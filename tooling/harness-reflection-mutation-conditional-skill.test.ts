import { expect, test } from "bun:test";
import {
  marginalAblation,
  promotionApproval,
  promotionRecords,
  record,
  request,
  retirementApproval,
} from "./harness-reflection-mutation-test-support.ts";
import { validateApprovedHarnessMutation } from "./harness-reflection-mutation-validation.ts";

const targetSkillPath = "harness/skills/enforcement-code/SKILL.md";
const routerPath = "harness/skills/harness-reflection/SKILL.md";

const consumers = {
  claude: {
    mechanism: "claude-user-skill" as const,
    state: "supported" as const,
  },
  codex: {
    mechanism: "codex-user-skill" as const,
    state: "supported" as const,
  },
  cursor: {
    mechanism: "cursor-user-skill" as const,
    state: "supported" as const,
  },
};

const conditionalPromotionRecords = (
  target: string = targetSkillPath,
): ReturnType<typeof promotionRecords> => {
  const records = promotionRecords();
  const before = record({
    ...records.before,
    consumers,
    surface: "conditional-skill",
    targetSkillPath: target,
  });
  const after = record({
    ...records.after,
    consumers,
    surface: "conditional-skill",
    targetSkillPath: target,
  });
  return { after, before };
};

const promotionSurface = (
  path: string,
): Readonly<{ path: string; preimage: string; replacement: string }> => ({
  path,
  preimage: "Existing skill guidance.\n",
  replacement: `Existing skill guidance.\n${marginalAblation.candidateTextExact}\n`,
});

const conditionalPromotionRequest = (
  target: string = targetSkillPath,
): ReturnType<typeof request> => {
  const records = conditionalPromotionRecords(target);
  return request({
    after: records.after,
    approval: promotionApproval,
    before: records.before,
    files: [promotionSurface(target)],
  });
};

const conditionalRetirementRequest = (): ReturnType<typeof request> => {
  const { after: active } = conditionalPromotionRecords();
  const retired = record({
    ...active,
    approval: retirementApproval,
    lifecycle: "retired",
    retirement: {
      reason: "The conditional guidance is no longer required.",
      retiredAt: retirementApproval.approvedAt,
    },
  });
  return request({
    after: retired,
    approval: retirementApproval,
    before: active,
    files: [
      {
        path: targetSkillPath,
        preimage: `Existing skill guidance.\n${marginalAblation.candidateTextExact}\n`,
        replacement: "Existing skill guidance.\n",
      },
    ],
  });
};

test("accepts conditional-skill promotion against its exact target skill", () => {
  expect(
    validateApprovedHarnessMutation(conditionalPromotionRequest()).kind,
  ).toBe("promotion");
});

test("accepts conditional-skill retirement against its exact target skill", () => {
  expect(
    validateApprovedHarnessMutation(conditionalRetirementRequest()).kind,
  ).toBe("retirement");
});

test("rejects a conditional skill without its exact target surface", () => {
  const records = conditionalPromotionRecords();
  expect(() =>
    validateApprovedHarnessMutation(
      request({
        after: records.after,
        approval: promotionApproval,
        before: records.before,
        files: [],
      }),
    ),
  ).toThrow("unsupported-control-surface");
});

test("rejects harness-reflection as a conditional skill target", () => {
  expect(() =>
    validateApprovedHarnessMutation(conditionalPromotionRequest(routerPath)),
  ).toThrow("conditional-skill-self-target");
});

test("rejects a surface path different from targetSkillPath", () => {
  const input = conditionalPromotionRequest();
  const records = conditionalPromotionRecords();
  expect(() =>
    validateApprovedHarnessMutation(
      request({
        after: records.after,
        approval: promotionApproval,
        before: records.before,
        files: [promotionSurface(routerPath)],
      }),
    ),
  ).toThrow("unsupported-control-surface");
  expect(input.approval.manifest.registryDelta.after.statement).not.toBe(
    marginalAblation.candidateTextExact,
  );
});

test("rejects a conditional target without an existing file preimage", () => {
  const records = conditionalPromotionRecords();
  expect(() =>
    validateApprovedHarnessMutation(
      request({
        after: records.after,
        approval: promotionApproval,
        before: records.before,
        files: [
          {
            ...promotionSurface(targetSkillPath),
            preimage: null,
          },
        ],
      }),
    ),
  ).toThrow("conditional-skill-target-must-exist");
});
