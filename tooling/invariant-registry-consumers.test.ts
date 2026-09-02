import { candidate, registry } from "./invariant-registry-test-support.ts";
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
