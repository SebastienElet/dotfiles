import { expect, test } from "bun:test";
import {
  linkApproval,
  linkRequest,
  withLinkAfter,
} from "./harness-reflection-mutation-link-test-support.ts";
import type { InvariantRecord } from "./invariant-registry-contract.ts";
import { validateApprovedHarnessMutation } from "./harness-reflection-mutation-validation.ts";

test.each(["candidate", "active"] as const)(
  "derives a link for a canonical source addition to a %s record",
  (lifecycle) => {
    const transition = validateApprovedHarnessMutation(linkRequest(lifecycle));

    expect(transition.kind).toBe("link");
    expect(transition.target.approval).toEqual(linkApproval);
  },
);

test.each([
  [
    "statement",
    (after: InvariantRecord): InvariantRecord => ({
      ...after,
      statement: "Changed statement.",
    }),
  ],
  [
    "scope",
    (after: InvariantRecord): InvariantRecord => ({
      ...after,
      scope: { exceptions: [], kind: "project-local" as const },
    }),
  ],
  [
    "severity",
    (after: InvariantRecord): InvariantRecord => ({
      ...after,
      severity: "high" as const,
    }),
  ],
] as const)("rejects a link that changes %s", (_field, change) => {
  const input = linkRequest();
  const { after } = input.approval.manifest.registryDelta;
  if (after === null) {
    throw new Error("link-after-record-missing");
  }

  expect(() =>
    validateApprovedHarnessMutation(withLinkAfter(input, change(after))),
  ).toThrow("invalid-link-transition");
});

test("rejects a link that removes an existing source", () => {
  const input = linkRequest();
  const { after } = input.approval.manifest.registryDelta;
  if (after === null) {
    throw new Error("link-after-record-missing");
  }

  expect(() =>
    validateApprovedHarnessMutation(
      withLinkAfter(input, { ...after, sources: after.sources.slice(1) }),
    ),
  ).toThrow("invalid-link-transition");
});

test("rejects a link that duplicates an existing source", () => {
  const input = linkRequest();
  const { after, before } = input.approval.manifest.registryDelta;
  if (after === null || before === null) {
    throw new Error("link-record-missing");
  }
  const [existingSource] = before.sources;
  if (existingSource === undefined) {
    throw new Error("link-source-missing");
  }

  expect(() =>
    validateApprovedHarnessMutation(
      withLinkAfter(input, {
        ...after,
        sources: [...before.sources, existingSource],
      }),
    ),
  ).toThrow("invalid-link-transition");
});
