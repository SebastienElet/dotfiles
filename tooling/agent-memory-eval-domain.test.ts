import { expect, test } from "bun:test";

import { capabilityChecks } from "./agent-memory-eval-domain.ts";

const nonce = "memory-nonce";
const unrelatedEvidence = {
  controlText: "CONTROL-NO-MEMORY",
  controlUnchanged: true,
  unrelatedText: "UNRELATED-NO-MEMORY",
  unrelatedUnchanged: true,
};

test("checks unrelated output against the fixture nonce", () => {
  expect(
    capabilityChecks(unrelatedEvidence, nonce).unrelated_not_injected,
  ).toBe(true);
  expect(
    capabilityChecks(
      { ...unrelatedEvidence, unrelatedText: `leaked ${nonce}` },
      nonce,
    ).unrelated_not_injected,
  ).toBe(false);
});
