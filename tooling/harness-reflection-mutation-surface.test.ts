import { expect, test } from "bun:test";
import {
  promotionRecords,
  promotionRequest,
  registryPath,
} from "./harness-reflection-mutation-test-support.ts";
import { validateApprovedHarnessMutation } from "./harness-reflection-mutation-validation.ts";

const forbiddenSurfacePaths = [
  "README.md",
  "package.json",
  "tooling/harness-reflection-mutation-validation.ts",
  "harness/skills/harness-reflection/references/invariant-registry.md",
] as const;

const mismatchedConsumerRequest = (): unknown => {
  const { after, before } = promotionRecords();
  const mismatchedAfter = {
    ...after,
    consumers: {
      ...after.consumers,
      claude: {
        mechanism: "claude-user-skill",
        state: "supported",
      },
    },
  };
  const request = promotionRequest();
  const [surface] = request.approval.manifest.files;
  if (surface === undefined) {
    throw new Error("surface-test-file-missing");
  }
  const registryReplacement = JSON.stringify({
    invariants: [mismatchedAfter],
    version: 1,
  });
  const registryFile = {
    path: registryPath,
    preimage: JSON.stringify({ invariants: [before], version: 1 }),
    replacement: registryReplacement,
  };
  return {
    ...request,
    approval: {
      ...request.approval,
      manifest: {
        files: [surface, registryFile],
        registryDelta: {
          after: mismatchedAfter,
          before,
          targetInvariantId: after.id,
        },
      },
    },
    preparedFiles: [
      request.preparedFiles[0],
      {
        contents: registryReplacement,
        path: registryPath,
        preimage: registryFile.preimage,
      },
    ],
  };
};

test.each([...forbiddenSurfacePaths])(
  "refuses unsupported mutation destination %s",
  (path) => {
    const request = promotionRequest();
    const [surface, registry] = request.approval.manifest.files;
    if (surface === undefined || registry === undefined) {
      throw new Error("surface-test-files-missing");
    }
    const changedSurface = { ...surface, path };

    expect(() =>
      validateApprovedHarnessMutation({
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
            path,
            preimage: changedSurface.preimage,
          },
          request.preparedFiles[1],
        ],
      }),
    ).toThrow("unsupported-control-surface");
  },
);

test("refuses consumers that do not match the selected target surface", () => {
  expect(() =>
    validateApprovedHarnessMutation(mismatchedConsumerRequest()),
  ).toThrow("mutation-consumer-surface-mismatch");
});
