import { expect, test } from "bun:test";
import {
  promotionApproval,
  retirementRequest,
} from "./harness-reflection-mutation-test-support.ts";
import type { InvariantRecord } from "./invariant-registry-contract.ts";
import { validateApprovedHarnessMutation } from "./harness-reflection-mutation-validation.ts";

const withRetiredRecord = (
  request: Readonly<ReturnType<typeof retirementRequest>>,
  after: InvariantRecord,
): ReturnType<typeof retirementRequest> => {
  const [surface, registry] = request.approval.manifest.files;
  const [surfacePrepared, registryPrepared] = request.preparedFiles;
  if (
    surface === undefined ||
    registry === undefined ||
    surfacePrepared === undefined ||
    registryPrepared === undefined
  ) {
    throw new Error("retirement-files-missing");
  }
  const replacement = JSON.stringify({ invariants: [after], version: 1 });
  return {
    ...request,
    approval: {
      ...request.approval,
      manifest: {
        files: [surface, { ...registry, replacement }],
        registryDelta: {
          ...request.approval.manifest.registryDelta,
          after,
        },
      },
    },
    preparedFiles: [
      surfacePrepared,
      { ...registryPrepared, contents: replacement },
    ],
  };
};

test("refuses retirement when a historical source is removed", () => {
  const request = retirementRequest();
  const { after } = request.approval.manifest.registryDelta;
  if (after === null || !Array.isArray(after.sources)) {
    throw new Error("retirement-record-missing");
  }

  const { sources } = after;

  expect(() =>
    validateApprovedHarnessMutation(
      withRetiredRecord(request, { ...after, sources: sources.slice(1) }),
    ),
  ).toThrow("retirement-history-changed");
});

test("refuses retirement when a scope exception is removed", () => {
  const request = retirementRequest();
  const { after, before } = request.approval.manifest.registryDelta;
  if (before === null || after === null) {
    throw new Error("retirement-record-missing");
  }
  const exception = { paths: ["legacy/**"], reason: "Legacy boundary." };
  const changedBefore = {
    ...before,
    scope: { exceptions: [exception], kind: "cross-project" as const },
  };
  const changedAfter = {
    ...after,
    scope: { exceptions: [], kind: "cross-project" as const },
  };
  const changed = withRetiredRecord(request, changedAfter);
  const [surface, registry] = changed.approval.manifest.files;
  if (surface === undefined || registry === undefined) {
    throw new Error("retirement-files-missing");
  }
  const preimage = JSON.stringify({ invariants: [changedBefore], version: 1 });

  expect(() =>
    validateApprovedHarnessMutation({
      ...changed,
      approval: {
        ...changed.approval,
        manifest: {
          ...changed.approval.manifest,
          files: [surface, { ...registry, preimage }],
          registryDelta: {
            ...changed.approval.manifest.registryDelta,
            before: changedBefore,
          },
        },
      },
      preparedFiles: [
        changed.preparedFiles[0],
        { ...changed.preparedFiles[1], preimage },
      ],
    }),
  ).toThrow("retirement-history-changed");
});

test("derives retirement kind and rejects a caller-supplied kind", () => {
  expect(validateApprovedHarnessMutation(retirementRequest()).kind).toBe(
    "retirement",
  );
  expect(() =>
    validateApprovedHarnessMutation({
      ...retirementRequest(),
      kind: "record-update",
    }),
  ).toThrow("mutation-request-invalid");
});

test("accepts retirement with its newly recorded approval attestation", () => {
  const request = retirementRequest();
  const result = validateApprovedHarnessMutation(request);

  expect(result.target.approval).toEqual({
    approvedAt: request.approval.approvedAt,
    approvedBy: request.approval.approvedBy,
  });
});

test("refuses reactivation of a retired invariant", () => {
  const retirement = retirementRequest();
  const { after: retired, before: active } =
    retirement.approval.manifest.registryDelta;
  if (retired === null || active === null) {
    throw new Error("retirement-record-missing");
  }
  const registryBefore = JSON.stringify({ invariants: [retired], version: 1 });
  const registryAfter = JSON.stringify({ invariants: [active], version: 1 });
  const files = [
    {
      path: "harness/AGENTS.md",
      preimage: "Replacement guidance.\n",
      replacement: "Always validate external input before domain use.",
    },
    {
      path: "harness/invariants/registry.json",
      preimage: registryBefore,
      replacement: registryAfter,
    },
  ] as const;

  expect(() =>
    validateApprovedHarnessMutation({
      approval: {
        ...promotionApproval,
        manifest: {
          files,
          registryDelta: {
            after: active,
            before: retired,
            targetInvariantId: active.id,
          },
        },
      },
      preparedFiles: files.map(
        ({
          path,
          preimage,
          replacement,
        }: Readonly<(typeof files)[number]>) => ({
          contents: replacement,
          path,
          preimage,
        }),
      ),
      targetInvariantId: active.id,
    }),
  ).toThrow("lifecycle-transition-invalid");
});

test("refuses a persisted approval different from the accepted attestation", () => {
  const request = retirementRequest();
  const { after } = request.approval.manifest.registryDelta;
  if (after === null) {
    throw new Error("retirement-record-missing");
  }

  expect(() =>
    validateApprovedHarnessMutation(
      withRetiredRecord(request, {
        ...after,
        approval: {
          approvedAt: request.approval.approvedAt,
          approvedBy: "Mallory",
        },
      }),
    ),
  ).toThrow("prepared-registry-approval-mismatch");
});

test("accepts an attestation without machine provenance", () => {
  const request = retirementRequest();
  expect("source" in request.approval).toBeFalse();
  expect(validateApprovedHarnessMutation(request).kind).toBe("retirement");
});

test("refuses caller-supplied approval provenance", () => {
  const request = retirementRequest();

  expect(() =>
    validateApprovedHarnessMutation({
      ...request,
      approval: { ...request.approval, claimedOrigin: "human" },
    }),
  ).toThrow("mutation-request-invalid");
});
