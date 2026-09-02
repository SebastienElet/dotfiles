import {
  type InvariantRecord,
  type InvariantRegistry,
  parseInvariantRegistry,
} from "./invariant-registry-contract.ts";
import {
  type TestInvariant,
  candidate,
  firstPullRequest,
  source,
} from "./invariant-registry-test-support.ts";
import { expect, expectTypeOf, test } from "bun:test";

expectTypeOf<InvariantRegistry["invariants"]>().not.toExtend<
  InvariantRecord[]
>();
expectTypeOf<InvariantRecord["sources"]>().not.toExtend<
  InvariantRecord["sources"][number][]
>();
expectTypeOf<InvariantRecord["scope"]["exceptions"]>().not.toExtend<
  InvariantRecord["scope"]["exceptions"][number][]
>();
expectTypeOf<
  InvariantRecord["scope"]["exceptions"][number]["paths"]
>().not.toExtend<string[]>();
expectTypeOf<InvariantRecord["consumers"]>().toEqualTypeOf<
  Readonly<InvariantRecord["consumers"]>
>();

test("rejects unknown registry versions", () => {
  expect(() =>
    parseInvariantRegistry({ version: 2, invariants: [] }),
  ).toThrow();
});

test("rejects unknown lifecycle values", () => {
  expect(() =>
    parseInvariantRegistry({
      version: 1,
      invariants: [{ ...candidate(), lifecycle: "enforced" }],
    }),
  ).toThrow();
});

test("rejects unknown record fields", () => {
  expect(() =>
    parseInvariantRegistry({
      version: 1,
      invariants: [{ ...candidate(), extra: true }],
    }),
  ).toThrow();
});

test("requires separate Claude, Codex and Cursor declarations", () => {
  const consumers: TestInvariant = {
    claude: { state: "supported", mechanism: "always-loaded-instruction" },
    codex: { state: "supported", mechanism: "always-loaded-instruction" },
  };

  expect(() =>
    parseInvariantRegistry({
      version: 1,
      invariants: [{ ...candidate(), consumers }],
    }),
  ).toThrow();
});

test.each([
  "http://github.com/SebastienElet/dotfiles/pull/206",
  "https://example.com/SebastienElet/dotfiles/pull/206",
  "https://github.com/SebastienElet/dotfiles/pull/206/files",
  "https://github.com/SebastienElet/dotfiles/issues/206",
  "https://github.com/SebastienElet/dotfiles/pull/0206",
  "https://github.com/SebastienElet/dotfiles/pull/206?diff=split",
] as const)(
  "rejects non-canonical pull request URL %s",
  (pullRequestUrl): void => {
    expect(() =>
      parseInvariantRegistry({
        version: 1,
        invariants: [
          candidate({
            sources: [{ ...source(firstPullRequest), pullRequestUrl }],
          }),
        ],
      }),
    ).toThrow();
  },
);

test("normalizes a trailing slash on a canonical pull request URL", () => {
  const canonicalUrl = source(firstPullRequest).pullRequestUrl;
  const parsed = parseInvariantRegistry({
    version: 1,
    invariants: [
      candidate({
        sources: [
          { ...source(firstPullRequest), pullRequestUrl: `${canonicalUrl}/` },
        ],
      }),
    ],
  });

  expect(parsed.invariants[0]?.sources[0]?.pullRequestUrl).toBe(canonicalUrl);
});

test("accepts canonical Bitbucket Cloud pull request comment evidence", () => {
  const parsed = parseInvariantRegistry({
    version: 1,
    invariants: [
      candidate({
        sources: [
          {
            provider: "bitbucket-cloud",
            pullRequestUrl:
              "https://bitbucket.org/acme/widgets/pull-requests/42/",
            evidenceUrl:
              "https://bitbucket.org/acme/widgets/pull-requests/42/comments/73",
          },
        ],
      }),
    ],
  });

  expect(parsed.invariants[0]?.sources[0]).toEqual({
    provider: "bitbucket-cloud",
    pullRequestUrl: "https://bitbucket.org/acme/widgets/pull-requests/42",
    evidenceUrl:
      "https://bitbucket.org/acme/widgets/pull-requests/42/comments/73",
  });
});

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
