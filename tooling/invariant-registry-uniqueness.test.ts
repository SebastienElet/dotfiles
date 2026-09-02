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

test("rejects review sources shared by multiple invariants", () => {
  const diagnostics = validateInvariantRegistry(
    registry(candidate(), candidate({ id: "different-invariant" })),
    validationOptions(),
  );

  expect(diagnosticCodes(diagnostics)).toContain("duplicate-source");
});

test("normalizes GitHub pull request URLs before source deduplication", () => {
  const canonicalSource = source(firstPullRequest);
  const diagnostics = validateInvariantRegistry(
    registry(
      candidate(),
      candidate({
        id: "different-invariant",
        sources: [
          {
            ...canonicalSource,
            pullRequestUrl:
              "https://github.com/sebastienelet/DOTFILES/pull/206/",
          },
        ],
      }),
    ),
    validationOptions(),
  );

  expect(diagnostics).toContainEqual({
    code: "duplicate-source",
    path: "invariants.1.sources.0.pullRequestUrl",
    message: "Review source is already assigned to an invariant.",
  });
});
