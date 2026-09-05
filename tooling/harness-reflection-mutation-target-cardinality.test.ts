import { expect, test } from "bun:test";
import {
  linkRequest,
  withLinkAfter,
} from "./harness-reflection-mutation-link-test-support.ts";
import { validateApprovedHarnessMutation } from "./harness-reflection-mutation-validation.ts";

const withRegistryPreimage = (
  input: Readonly<ReturnType<typeof linkRequest>>,
  preimage: string,
  before: unknown,
): unknown => {
  const [registry] = input.approval.manifest.files;
  const [prepared] = input.preparedFiles;
  if (registry === undefined || prepared === undefined) {
    throw new Error("link-registry-file-missing");
  }
  return {
    ...input,
    approval: {
      ...input.approval,
      manifest: {
        files: [{ ...registry, preimage }],
        registryDelta: {
          ...input.approval.manifest.registryDelta,
          before,
        },
      },
    },
    preparedFiles: [{ ...prepared, preimage }],
  };
};

test("rejects two target copies in the registry preimage", () => {
  const input = linkRequest();
  const { before } = input.approval.manifest.registryDelta;
  const preimage = JSON.stringify({ invariants: [before, before], version: 1 });

  expect(() =>
    validateApprovedHarnessMutation(
      withRegistryPreimage(input, preimage, before),
    ),
  ).toThrow("registry-target-count-invalid");
});

test("rejects two target copies in the registry replacement", () => {
  const input = linkRequest();
  const { after } = input.approval.manifest.registryDelta;
  if (after === null) {
    throw new Error("link-after-record-missing");
  }
  const [registry] = input.approval.manifest.files;
  const [prepared] = input.preparedFiles;
  if (registry === undefined || prepared === undefined) {
    throw new Error("link-registry-file-missing");
  }
  const replacement = JSON.stringify({
    invariants: [after, after],
    version: 1,
  });

  expect(() =>
    validateApprovedHarnessMutation({
      ...input,
      approval: {
        ...input.approval,
        manifest: {
          ...input.approval.manifest,
          files: [{ ...registry, replacement }],
        },
      },
      preparedFiles: [{ ...prepared, contents: replacement }],
    }),
  ).toThrow("registry-target-count-invalid");
});

test("rejects an absent target in the registry preimage", () => {
  const input = linkRequest();
  const preimage = JSON.stringify({ invariants: [], version: 1 });

  expect(() =>
    validateApprovedHarnessMutation(
      withRegistryPreimage(input, preimage, null),
    ),
  ).toThrow("registry-target-count-invalid");
});

test("rejects an absent target in the registry replacement", () => {
  expect(() =>
    validateApprovedHarnessMutation(withLinkAfter(linkRequest(), null)),
  ).toThrow("registry-target-count-invalid");
});
