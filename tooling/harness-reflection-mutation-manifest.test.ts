import { expect, test } from "bun:test";
import { promotionRequest } from "./harness-reflection-mutation-test-support.ts";
import { validateApprovedHarnessMutation } from "./harness-reflection-mutation-validation.ts";

type ManifestFile = ReturnType<
  typeof promotionRequest
>["approval"]["manifest"]["files"][number];

const manifestFiles = (
  request: Readonly<ReturnType<typeof promotionRequest>>,
): Readonly<{ registry: ManifestFile; surface: ManifestFile }> => {
  const [surface, registry] = request.approval.manifest.files;
  if (surface === undefined || registry === undefined) {
    throw new Error("manifest-files-missing");
  }
  return { registry, surface };
};

test("refuses a manifest path different from the request", () => {
  const request = promotionRequest();
  const { registry, surface } = manifestFiles(request);

  expect(() =>
    validateApprovedHarnessMutation({
      ...request,
      approval: {
        ...request.approval,
        manifest: {
          ...request.approval.manifest,
          files: [{ ...surface, path: "other.md" }, registry],
        },
      },
    }),
  ).toThrow("approved-manifest-request-mismatch");
});

test("refuses a manifest replacement different from the request", () => {
  const request = promotionRequest();
  const { registry, surface } = manifestFiles(request);

  expect(() =>
    validateApprovedHarnessMutation({
      ...request,
      approval: {
        ...request.approval,
        manifest: {
          ...request.approval.manifest,
          files: [
            { ...surface, replacement: "Different guidance.\n" },
            registry,
          ],
        },
      },
    }),
  ).toThrow("approved-manifest-request-mismatch");
});

test("refuses a manifest preimage different from the request", () => {
  const request = promotionRequest();
  const { registry, surface } = manifestFiles(request);

  expect(() =>
    validateApprovedHarnessMutation({
      ...request,
      approval: {
        ...request.approval,
        manifest: {
          ...request.approval.manifest,
          files: [{ ...surface, preimage: "Different preimage.\n" }, registry],
        },
      },
    }),
  ).toThrow("approved-manifest-request-mismatch");
});

test("refuses a manifest registry delta with a different before record", () => {
  const request = promotionRequest();

  expect(() =>
    validateApprovedHarnessMutation({
      ...request,
      approval: {
        ...request.approval,
        manifest: {
          ...request.approval.manifest,
          registryDelta: {
            ...request.approval.manifest.registryDelta,
            before: null,
          },
        },
      },
    }),
  ).toThrow("approved-registry-delta-mismatch");
});

test("refuses a manifest registry delta with a different after record", () => {
  const request = promotionRequest();

  expect(() =>
    validateApprovedHarnessMutation({
      ...request,
      approval: {
        ...request.approval,
        manifest: {
          ...request.approval.manifest,
          registryDelta: {
            ...request.approval.manifest.registryDelta,
            after: null,
          },
        },
      },
    }),
  ).toThrow("approved-registry-delta-mismatch");
});

test("refuses a manifest whose registry delta has equal records", () => {
  const request = promotionRequest();
  const { before } = request.approval.manifest.registryDelta;

  expect(() =>
    validateApprovedHarnessMutation({
      ...request,
      approval: {
        ...request.approval,
        manifest: {
          ...request.approval.manifest,
          registryDelta: {
            ...request.approval.manifest.registryDelta,
            after: before,
          },
        },
      },
    }),
  ).toThrow("approved-registry-delta-mismatch");
});
