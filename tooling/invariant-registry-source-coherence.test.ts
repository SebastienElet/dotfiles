import { expect, test } from "bun:test";
import { candidate } from "./invariant-registry-test-support.ts";
import { parseInvariantRegistry } from "./invariant-registry-contract.ts";

test.each([
  {
    name: "provider domain",
    source: {
      provider: "github",
      pullRequestUrl: "https://example.com/acme/widgets/pull/42",
      evidenceUrl: "https://github.com/acme/widgets/pull/42#issuecomment-73",
    },
  },
  {
    name: "evidence repository",
    source: {
      provider: "github",
      pullRequestUrl: "https://github.com/acme/widgets/pull/42",
      evidenceUrl: "https://github.com/acme/other/pull/42#issuecomment-73",
    },
  },
  {
    name: "evidence pull request",
    source: {
      provider: "bitbucket-cloud",
      pullRequestUrl: "https://bitbucket.org/acme/widgets/pull-requests/42",
      evidenceUrl:
        "https://bitbucket.org/acme/widgets/pull-requests/41/comments/73",
    },
  },
  {
    name: "unstable evidence location",
    source: {
      provider: "github",
      pullRequestUrl: "https://github.com/acme/widgets/pull/42",
      evidenceUrl: "https://github.com/acme/widgets/pull/42/files",
    },
  },
] as const)("rejects a source with incoherent $name", (testCase) => {
  expect(() =>
    parseInvariantRegistry({
      version: 1,
      invariants: [candidate({ sources: [testCase.source] })],
    }),
  ).toThrow();
});
