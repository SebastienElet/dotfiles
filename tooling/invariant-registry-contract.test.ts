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

test("rejects a Bitbucket REST-shaped path as HTML evidence", () => {
  expect(() =>
    parseInvariantRegistry({
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
    }),
  ).toThrow();
});

test("accepts the Bitbucket Cloud comment HTML link returned by the API", () => {
  const parsed = parseInvariantRegistry({
    version: 1,
    invariants: [
      candidate({
        sources: [
          {
            provider: "bitbucket-cloud",
            pullRequestUrl:
              "https://bitbucket.org/Acme/Widgets/pull-requests/42",
            evidenceUrl:
              "https://bitbucket.org/acme/widgets/pull-requests/42/_/diff#comment-73",
          },
        ],
      }),
    ],
  });

  expect(parsed.invariants[0]?.sources[0]?.evidenceUrl).toBe(
    "https://bitbucket.org/acme/widgets/pull-requests/42/_/diff#comment-73",
  );
});

test("accepts and canonicalizes GitHub inline discussion evidence", () => {
  const parsed = parseInvariantRegistry({
    version: 1,
    invariants: [
      candidate({
        sources: [
          {
            provider: "github",
            pullRequestUrl: "https://github.com/Acme/Widgets/pull/42",
            evidenceUrl:
              "https://github.com/acme/widgets/pull/42#discussion-diff-7301",
          },
        ],
      }),
    ],
  });

  expect(parsed.invariants[0]?.sources[0]).toEqual({
    provider: "github",
    pullRequestUrl: "https://github.com/Acme/Widgets/pull/42",
    evidenceUrl: "https://github.com/acme/widgets/pull/42#discussion-diff-7301",
  });
});

test("reconstructs canonical evidence instead of retaining input syntax", () => {
  const parsed = parseInvariantRegistry({
    version: 1,
    invariants: [
      candidate({
        sources: [
          {
            provider: "bitbucket-cloud",
            pullRequestUrl:
              "https://bitbucket.org/Acme/Widgets/pull-requests/42/",
            evidenceUrl:
              "https://bitbucket.org/acme/widgets/pull-requests/42/_/diff#comment-73",
          },
        ],
      }),
    ],
  });

  expect(parsed.invariants[0]?.sources[0]).toEqual({
    provider: "bitbucket-cloud",
    pullRequestUrl: "https://bitbucket.org/Acme/Widgets/pull-requests/42",
    evidenceUrl:
      "https://bitbucket.org/acme/widgets/pull-requests/42/_/diff#comment-73",
  });
});

test.each([
  "https://user@github.com/acme/widgets/pull/42",
  "https://github.com:443/acme/widgets/pull/42",
  "https://github.com/acme/widgets/pull/42?diff=split",
  "https://github.com/../widgets/pull/42",
  "https://github.com/acme/%2e%2e/widgets/pull/42",
  "https://github.com/acme/temp/../widgets/pull/42",
  "https://github.com/acme/./widgets/pull/42",
  "https://github.com/acme/widgets/PULL/42",
] as const)(
  "rejects structurally unsafe pull request URL %s",
  (pullRequestUrl) => {
    expect(() =>
      parseInvariantRegistry({
        version: 1,
        invariants: [
          candidate({
            sources: [
              {
                provider: "github",
                pullRequestUrl,
                evidenceUrl:
                  "https://github.com/acme/widgets/pull/42#issuecomment-73",
              },
            ],
          }),
        ],
      }),
    ).toThrow();
  },
);

test.each([
  "https://github.com/acme/widgets/pull/42#ISSUECOMMENT-73",
  "https://github.com/acme/widgets/pull/42?x=1#issuecomment-73",
  "https://github.com/acme/widgets/PULL/42#issuecomment-73",
  "https://github.com/acme/temp/../widgets/pull/42#issuecomment-73",
  "https://github.com/acme/widgets/pull/42#discussion-DIFF-73",
  "https://user@github.com/acme/widgets/pull/42#discussion-diff-73",
  "https://github.com:443/acme/widgets/pull/42#discussion-diff-73",
  "https://bitbucket.org/acme/widgets/pull-requests/42/_/diff#COMMENT-73",
  "https://bitbucket.org/acme/other/pull-requests/42/_/diff#comment-73",
  "https://bitbucket.org/acme/widgets/pull-requests/41/_/diff#comment-73",
] as const)(
  "rejects structurally unsafe or incoherent evidence URL %s",
  (evidenceUrl) => {
    const bitbucket = evidenceUrl.includes("bitbucket.org");
    expect(() =>
      parseInvariantRegistry({
        version: 1,
        invariants: [
          candidate({
            sources: [
              bitbucket
                ? {
                    provider: "bitbucket-cloud",
                    pullRequestUrl:
                      "https://bitbucket.org/acme/widgets/pull-requests/42",
                    evidenceUrl,
                  }
                : {
                    provider: "github",
                    pullRequestUrl: "https://github.com/acme/widgets/pull/42",
                    evidenceUrl,
                  },
            ],
          }),
        ],
      }),
    ).toThrow();
  },
);
