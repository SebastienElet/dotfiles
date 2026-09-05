import {
  candidate,
  diagnosticCodes,
  firstPullRequest,
  registry,
  secondPullRequest,
  source,
  validateInvariantRegistry,
  validationOptions,
} from "./invariant-registry-test-support.ts";
import { expect, test } from "bun:test";

test("rejects duplicate identifiers", () => {
  const diagnostics = validateInvariantRegistry(
    registry(candidate(), candidate({ sources: [source(secondPullRequest)] })),
    validationOptions(),
  );

  expect(diagnosticCodes(diagnostics)).toContain("duplicate-id");
});

test("rejects evidence occurrences shared by multiple invariants", () => {
  const diagnostics = validateInvariantRegistry(
    registry(candidate(), candidate({ id: "different-invariant" })),
    validationOptions(),
  );

  expect(diagnosticCodes(diagnostics)).toContain("duplicate-evidence");
});

test("allows two evidence occurrences from one pull request", () => {
  const canonicalSource = source(firstPullRequest);
  const diagnostics = validateInvariantRegistry(
    registry(
      candidate({
        sources: [
          canonicalSource,
          {
            ...canonicalSource,
            evidenceUrl: `${canonicalSource.pullRequestUrl}#pullrequestreview-207`,
          },
        ],
      }),
    ),
    validationOptions(),
  );

  expect(diagnostics).toEqual([]);
});

test("normalizes evidence identity before global deduplication", () => {
  const canonicalSource = source(firstPullRequest);
  const diagnostics = validateInvariantRegistry(
    registry(
      candidate({ sources: [canonicalSource] }),
      candidate({
        id: "different-invariant",
        sources: [
          {
            ...canonicalSource,
            evidenceUrl:
              "https://github.com/sebastienelet/DOTFILES/pull/206#issuecomment-206",
            pullRequestUrl:
              "https://github.com/sebastienelet/DOTFILES/pull/206/",
          },
        ],
      }),
    ),
    validationOptions(),
  );

  expect(diagnostics).toContainEqual({
    code: "duplicate-evidence",
    path: "invariants.1.sources.0.evidenceUrl",
    message: "Review evidence is already assigned to an invariant.",
  });
});
