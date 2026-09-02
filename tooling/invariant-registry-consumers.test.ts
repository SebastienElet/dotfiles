import {
  candidate,
  diagnosticCodes,
  registry,
  validateInvariantRegistry,
  validationOptions,
} from "./invariant-registry-test-support.ts";
import { expect, test } from "bun:test";

const defaultConsumers = {
  claude: { state: "supported", mechanism: "claude-global-instruction" },
  codex: { state: "supported", mechanism: "codex-global-instruction" },
  cursor: {
    state: "unsupported",
    reason: "No managed instruction surface.",
  },
};

test("rejects a nonexistent consumer adapter", () => {
  expect(() =>
    registry(
      candidate({
        consumers: {
          ...defaultConsumers,
          claude: { state: "supported", mechanism: "nonexistent-adapter" },
        },
      }),
    ),
  ).toThrow();
});

test("rejects a consumer mechanism owned by another agent", () => {
  expect(() =>
    registry(
      candidate({
        consumers: {
          ...defaultConsumers,
          claude: {
            state: "supported",
            mechanism: "codex-global-instruction",
          },
        },
      }),
    ),
  ).toThrow();
});

test("rejects Cursor user-skill support for an always-loaded instruction", () => {
  const diagnostics = validateInvariantRegistry(
    registry(
      candidate({
        consumers: {
          ...defaultConsumers,
          cursor: {
            state: "supported",
            mechanism: "cursor-user-skill",
          },
        },
      }),
    ),
    validationOptions(),
  );

  expect(diagnosticCodes(diagnostics)).toContain("consumer-surface-mismatch");
});
