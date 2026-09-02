import { expect, test } from "bun:test";
import {
  memoryAdapter,
  registryPath,
  retirementInput,
  retirementRegistryPair,
} from "./harness-reflection-mutation-workflow-test-support.ts";
import type { MutationWorkflowCoreInput } from "./harness-reflection-mutation-workflow-types.ts";
import { executeHarnessMutationWorkflowCore } from "./harness-reflection-mutation-workflow-core.ts";

const execute = (
  input: MutationWorkflowCoreInput,
): ReturnType<typeof executeHarnessMutationWorkflowCore> => {
  const pair = retirementRegistryPair();
  return executeHarnessMutationWorkflowCore(
    input,
    memoryAdapter({
      [registryPath]: pair.current,
      "surface.md": "old surface",
    }),
  );
};

test("refuses a manifest path different from the request", async () => {
  const pair = retirementRegistryPair();
  const input = retirementInput(pair.current, pair.retired);
  const { approval } = input;
  if (approval === undefined) {
    throw new Error("manifest-missing");
  }
  const { manifest } = approval;
  const [surface, registry] = manifest.files;
  if (surface === undefined || registry === undefined) {
    throw new Error("manifest-files-missing");
  }

  const result = await execute({
    ...input,
    approval: {
      ...approval,
      manifest: {
        ...manifest,
        files: [{ ...surface, path: "other.md" }, registry],
      },
    },
  });

  expect(result.reason).toBe("approved-manifest-request-mismatch");
});

test("refuses a manifest replacement different from the request", async () => {
  const pair = retirementRegistryPair();
  const input = retirementInput(pair.current, pair.retired);
  const { approval } = input;
  if (approval === undefined) {
    throw new Error("manifest-missing");
  }
  const { manifest } = approval;
  const [surface, registry] = manifest.files;
  if (surface === undefined || registry === undefined) {
    throw new Error("manifest-files-missing");
  }

  const result = await execute({
    ...input,
    approval: {
      ...approval,
      manifest: {
        ...manifest,
        files: [{ ...surface, replacement: "different surface" }, registry],
      },
    },
  });

  expect(result.reason).toBe("approved-manifest-request-mismatch");
});

test("refuses a manifest preimage different from the target", async () => {
  const pair = retirementRegistryPair();
  const input = retirementInput(pair.current, pair.retired);
  const { approval } = input;
  if (approval === undefined) {
    throw new Error("manifest-missing");
  }
  const { manifest } = approval;
  const [surface, registry] = manifest.files;
  if (surface === undefined || registry === undefined) {
    throw new Error("manifest-files-missing");
  }

  const result = await execute({
    ...input,
    approval: {
      ...approval,
      manifest: {
        ...manifest,
        files: [{ ...surface, preimage: "different surface" }, registry],
      },
    },
  });

  expect(result.reason).toBe("approved-manifest-preimage-mismatch");
});

test("refuses a manifest registry delta with a different before record", async () => {
  const pair = retirementRegistryPair();
  const input = retirementInput(pair.current, pair.retired);
  const { approval } = input;
  if (approval === undefined) {
    throw new Error("manifest-missing");
  }
  const { manifest } = approval;

  const result = await execute({
    ...input,
    approval: {
      ...approval,
      manifest: {
        ...manifest,
        registryDelta: { ...manifest.registryDelta, before: null },
      },
    },
  });

  expect(result.reason).toBe("approved-registry-delta-mismatch");
});

test("refuses a manifest registry delta with a different after record", async () => {
  const pair = retirementRegistryPair();
  const input = retirementInput(pair.current, pair.retired);
  const { approval } = input;
  if (approval === undefined) {
    throw new Error("manifest-missing");
  }
  const { manifest } = approval;

  const result = await execute({
    ...input,
    approval: {
      ...approval,
      manifest: {
        ...manifest,
        registryDelta: { ...manifest.registryDelta, after: null },
      },
    },
  });

  expect(result.reason).toBe("approved-registry-delta-mismatch");
});

test("refuses a manifest whose registry delta has equal records", async () => {
  const pair = retirementRegistryPair();
  const input = retirementInput(pair.current, pair.current);
  const { approval } = input;
  if (approval === undefined) {
    throw new Error("manifest-missing");
  }
  const { manifest } = approval;

  const result = await execute({
    ...input,
    approval: {
      ...approval,
      manifest: {
        ...manifest,
        registryDelta: {
          ...manifest.registryDelta,
          after: manifest.registryDelta.before,
        },
      },
    },
  });

  expect(result.reason).toBe("approved-registry-delta-mismatch");
});
