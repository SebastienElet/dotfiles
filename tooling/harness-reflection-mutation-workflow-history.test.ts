import {
  approvedAt,
  memoryAdapter,
  registryPath,
  retirementInput,
  retirementRegistryPair,
} from "./harness-reflection-mutation-workflow-test-support.ts";
import { expect, test } from "bun:test";
import {
  firstPullRequest,
  secondPullRequest,
  source,
} from "./invariant-registry-test-support.ts";
import { executeHarnessMutationWorkflowCore } from "./harness-reflection-mutation-workflow-core.ts";

test("refuses retirement when any historical source is removed", async () => {
  const pair = retirementRegistryPair({
    sources: [source(firstPullRequest), source(secondPullRequest)],
  });
  const proposed = retirementRegistryPair({
    sources: [source(firstPullRequest)],
  }).retired;
  const adapter = memoryAdapter({
    [registryPath]: pair.current,
    "surface.md": "old surface",
  });

  const result = await executeHarnessMutationWorkflowCore(
    retirementInput(pair.current, proposed),
    adapter,
  );

  expect(result.status).toBe("rejected");
  expect(result.reason).toBe("retirement-history-changed");
  expect(adapter.contents.get(registryPath)).toBe(pair.current);
});

test("refuses retirement when a scope exception is removed", async () => {
  const pair = retirementRegistryPair({
    scope: {
      exceptions: [{ paths: ["legacy/**"], reason: "Legacy boundary." }],
      kind: "cross-project",
    },
  });
  const proposed = pair.retired.replace(
    '"exceptions":[{"paths":["legacy/**"],"reason":"Legacy boundary."}]',
    '"exceptions":[]',
  );
  const adapter = memoryAdapter({
    [registryPath]: pair.current,
    "surface.md": "old surface",
  });

  const result = await executeHarnessMutationWorkflowCore(
    retirementInput(pair.current, proposed),
    adapter,
  );

  expect(result.status).toBe("rejected");
  expect(result.reason).toBe("retirement-history-changed");
  expect(adapter.contents.get(registryPath)).toBe(pair.current);
});

test("refuses a caller-supplied mutation kind", async () => {
  const currentPair = retirementRegistryPair({
    sources: [source(firstPullRequest), source(secondPullRequest)],
  });
  const proposed = retirementRegistryPair({
    sources: [source(firstPullRequest)],
  }).retired;
  const adapter = memoryAdapter({
    [registryPath]: currentPair.current,
    "surface.md": "old surface",
  });

  const result = await executeHarnessMutationWorkflowCore(
    Object.assign(retirementInput(currentPair.current, proposed), {
      kind: "approved-mutation",
    }),
    adapter,
  );

  expect(result.status).toBe("rejected");
  expect(result.reason).toBe("mutation-request-invalid");
  expect(adapter.contents.get(registryPath)).toBe(currentPair.current);
});

test("accepts retirement with a newly recorded approval attestation", async () => {
  const pair = retirementRegistryPair();
  const secondApprovalAt = "2026-09-03T00:00:00.000Z";
  const retired = pair.retired
    .replace('"approvedBy":"Reviewer"', '"approvedBy":"Second reviewer"')
    .replaceAll("2026-09-02T00:00:00.000Z", secondApprovalAt);
  const adapter = memoryAdapter({
    [registryPath]: pair.current,
    "surface.md": "old surface",
  });

  const result = await executeHarnessMutationWorkflowCore(
    retirementInput(pair.current, retired, {
      approval: {
        approvedAt: secondApprovalAt,
        approvedBy: "Second reviewer",
      },
    }),
    adapter,
  );

  expect(result.status).toBe("succeeded");
  expect(adapter.contents.get(registryPath)).toBe(retired);
});

test("refuses reactivation of a retired invariant", async () => {
  const pair = retirementRegistryPair();
  const adapter = memoryAdapter({
    [registryPath]: pair.retired,
    "surface.md": "old surface",
  });

  const result = await executeHarnessMutationWorkflowCore(
    retirementInput(pair.retired, pair.current),
    adapter,
  );

  expect(result.status).toBe("rejected");
  expect(result.reason).toBe("lifecycle-transition-invalid");
  expect(adapter.contents.get(registryPath)).toBe(pair.retired);
});

test("refuses a persisted approval different from the accepted context", async () => {
  const pair = retirementRegistryPair();
  const persisted = pair.retired.replaceAll("Reviewer", "Mallory");
  const adapter = memoryAdapter({
    [registryPath]: pair.current,
    "surface.md": "old surface",
  });

  const result = await executeHarnessMutationWorkflowCore(
    retirementInput(pair.current, persisted, {
      approval: {
        approvedAt,
        approvedBy: "Alice",
      },
    }),
    adapter,
  );

  expect(result.status).toBe("rejected");
  expect(result.reason).toBe("prepared-registry-approval-mismatch");
  expect(adapter.contents.get(registryPath)).toBe(pair.current);
});

test("accepts an approval attestation without machine provenance", async () => {
  const pair = retirementRegistryPair();
  const adapter = memoryAdapter({
    [registryPath]: pair.current,
    "surface.md": "old surface",
  });
  const input = retirementInput(pair.current, pair.retired);
  const { approval } = input;
  if (approval === undefined) {
    throw new Error("approval-attestation-missing");
  }
  expect("source" in approval).toBe(false);

  const result = await executeHarnessMutationWorkflowCore(
    { ...input, approval },
    adapter,
  );

  expect(result.status).toBe("succeeded");
  expect(adapter.contents.get(registryPath)).toBe(pair.retired);
});

test("refuses a caller-supplied approval provenance field", async () => {
  const pair = retirementRegistryPair();
  const adapter = memoryAdapter({
    [registryPath]: pair.current,
    "surface.md": "old surface",
  });
  const input = retirementInput(pair.current, pair.retired);
  const { approval } = input;
  if (approval === undefined) {
    throw new Error("approval-attestation-missing");
  }
  const approvalWithProvenance = {
    ...approval,
    claimedOrigin: "human",
  };
  const result = await executeHarnessMutationWorkflowCore(
    {
      ...input,
      approval: approvalWithProvenance,
    },
    adapter,
  );

  expect(result.status).toBe("rejected");
  expect(result.reason).toBe("mutation-request-invalid");
  expect(adapter.contents.get(registryPath)).toBe(pair.current);
});
