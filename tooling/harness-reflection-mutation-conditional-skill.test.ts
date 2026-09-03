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

const conditionalPromotionRecords = () => {
  const records = promotionRecords();
  const before = record({
    ...records.before,
    consumers,
    statement: marginalAblation.candidateTextExact,
    surface: "conditional-skill",
  });
  const after = record({
    ...records.after,
    consumers,
    statement: marginalAblation.candidateTextExact,
    surface: "conditional-skill",
  });
  return { after, before };
};

const conditionalPromotionRequest = () => {
  const records = conditionalPromotionRecords();
  return request({
    after: records.after,
    approval: promotionApproval,
    before: records.before,
    files: [],
  });
};

const conditionalRetirementRequest = () => {
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
    files: [],
  });
};

test("accepts conditional-skill promotion as an exact registry-only change", () => {
  expect(validateApprovedHarnessMutation(conditionalPromotionRequest()).kind).toBe(
    "promotion",
  );
});

test("accepts conditional-skill retirement as an exact registry-only change", () => {
  expect(validateApprovedHarnessMutation(conditionalRetirementRequest()).kind).toBe(
    "retirement",
  );
});

test("rejects a conditional-skill request that mutates the closed router", () => {
  const input = conditionalPromotionRequest();
  const routerFile = {
    path: "harness/skills/harness-reflection/SKILL.md",
    preimage: "closed router",
    replacement: "changed router",
  };

  expect(() =>
    validateApprovedHarnessMutation({
      ...input,
      approval: {
        ...input.approval,
        manifest: {
          ...input.approval.manifest,
          files: [...input.approval.manifest.files, routerFile],
        },
      },
      preparedFiles: [
        ...input.preparedFiles,
        {
          contents: routerFile.replacement,
          path: routerFile.path,
          preimage: routerFile.preimage,
        },
      ],
    }),
  ).toThrow("conditional-skill-registry-only");
});

test("rejects conditional candidate text that differs from its registry statement", () => {
  const input = conditionalPromotionRequest();
  const after = record({
    ...input.approval.manifest.registryDelta.after,
    statement: "Different conditional guidance.",
  });

  expect(() =>
    validateApprovedHarnessMutation(
      request({
        after,
        approval: promotionApproval,
        before: input.approval.manifest.registryDelta.before,
        files: [],
      }),
    ),
  ).toThrow("conditional-skill-statement-mismatch");
});
