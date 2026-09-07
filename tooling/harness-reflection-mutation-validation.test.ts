import { expect, test } from "bun:test";
import {
  marginalAblation,
  promotionRequest,
  retirementRequest,
} from "./harness-reflection-mutation-test-support.ts";
import { validateApprovedHarnessMutation } from "./harness-reflection-mutation-validation.ts";

const replaceSurface = (
  request: Readonly<ReturnType<typeof promotionRequest>>,
  changes: Readonly<{ preimage?: string; replacement?: string }>,
): ReturnType<typeof promotionRequest> => {
  const [surface, registry] = request.approval.manifest.files;
  const [, registryPrepared] = request.preparedFiles;
  if (
    surface === undefined ||
    registry === undefined ||
    registryPrepared === undefined
  ) {
    throw new Error("promotion-files-missing");
  }
  const changedSurface = { ...surface, ...changes };
  return {
    ...request,
    approval: {
      ...request.approval,
      manifest: {
        ...request.approval.manifest,
        files: [changedSurface, registry],
      },
    },
    preparedFiles: [
      {
        contents: changedSurface.replacement,
        path: changedSurface.path,
        preimage: changedSurface.preimage,
      },
      registryPrepared,
    ],
  };
};

test("accepts an exact promotion whose candidate text is newly present", () => {
  expect(validateApprovedHarnessMutation(promotionRequest()).kind).toBe(
    "promotion",
  );
});

test("rejects a no-op surface replacement", () => {
  const request = promotionRequest();
  const [surface] = request.approval.manifest.files;
  if (surface === undefined || surface.preimage === null) {
    throw new Error("promotion-surface-missing");
  }
  const { preimage } = surface;

  expect(() =>
    validateApprovedHarnessMutation(
      replaceSurface(request, { replacement: preimage }),
    ),
  ).toThrow("approved-file-no-op");
});

test("rejects a promotion replacement without the exact candidate text", () => {
  expect(() =>
    validateApprovedHarnessMutation(
      replaceSurface(promotionRequest(), {
        replacement: "Different guidance.\n",
      }),
    ),
  ).toThrow("promotion-candidate-text-not-added");
});

test("rejects a promotion when the exact candidate text already existed", () => {
  expect(() =>
    validateApprovedHarnessMutation(
      replaceSurface(promotionRequest(), {
        preimage: marginalAblation.candidateTextExact,
      }),
    ),
  ).toThrow("promotion-candidate-text-not-added");
});

test("accepts retirement only when the exact candidate text is removed", () => {
  expect(validateApprovedHarnessMutation(retirementRequest()).kind).toBe(
    "retirement",
  );
});

test("rejects retirement when the exact candidate text remains", () => {
  expect(() =>
    validateApprovedHarnessMutation(
      retirementRequest({
        path: "harness/AGENTS.md",
        preimage: marginalAblation.candidateTextExact,
        replacement: marginalAblation.candidateTextExact,
      }),
    ),
  ).toThrow("approved-file-no-op");
});
