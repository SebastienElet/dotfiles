import { candidate, source } from "./invariant-registry-test-support.ts";
import {
  promotionRecords,
  record,
  request,
} from "./harness-reflection-mutation-test-support.ts";
import type { InvariantRecord } from "./invariant-registry-contract.ts";

const linkApproval = {
  approvedAt: "2026-09-05T09:00:00.000Z",
  approvedBy: "Link reviewer",
};

const linkRequest = (
  lifecycle: "active" | "candidate" = "candidate",
): ReturnType<typeof request> => {
  const before =
    lifecycle === "active"
      ? promotionRecords().after
      : record(candidate({ id: "validate-boundary-input" }));
  const after = record({
    ...before,
    approval: linkApproval,
    sources: [...before.sources, source("208")],
  });
  return request({ after, approval: linkApproval, before, files: [] });
};

const withLinkAfter = (
  input: Readonly<ReturnType<typeof linkRequest>>,
  after: InvariantRecord | null,
): unknown => {
  const [registry] = input.approval.manifest.files;
  const [prepared] = input.preparedFiles;
  if (registry === undefined || prepared === undefined) {
    throw new Error("link-registry-file-missing");
  }
  const replacement = JSON.stringify({
    invariants: after === null ? [] : [after],
    version: 1,
  });
  return {
    ...input,
    approval: {
      ...input.approval,
      manifest: {
        files: [{ ...registry, replacement }],
        registryDelta: {
          ...input.approval.manifest.registryDelta,
          after,
        },
      },
    },
    preparedFiles: [{ ...prepared, contents: replacement }],
  };
};

export { linkApproval, linkRequest, withLinkAfter };
