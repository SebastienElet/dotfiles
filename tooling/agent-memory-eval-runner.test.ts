import {
  assertCapabilityOwnership,
  storedEntryId,
  storedStatus,
} from "./agent-memory-eval-runner-support.ts";
import { expect, test } from "bun:test";
import { loadEvaluationScenarios } from "./agent-memory-eval-scenario.ts";
import { resolve } from "node:path";

test("each scenario owns its declared capabilities exactly once", async () => {
  const scenarios = await loadEvaluationScenarios(
    resolve(import.meta.dir, "agent-memory-eval-scenarios.json"),
  );
  expect(() => {
    assertCapabilityOwnership(scenarios);
  }).not.toThrow();
  const duplicated = {
    ...scenarios,
    scenarios: scenarios.scenarios.map((scenario, index) =>
      index === 0
        ? Object.assign(scenario, {
            capabilities: [...scenario.capabilities, "stored"],
          })
        : scenario,
    ),
  };
  expect(() => {
    assertCapabilityOwnership(duplicated);
  }).toThrow("exactly once");
});

test("accepts only an autonomous stored JSON result line", () => {
  expect(storedStatus('command output\n{"status":"stored"}\n')).toBe(true);
  expect(storedEntryId('{"status":"stored","id":"mem_candidate"}')).toBe(
    "mem_candidate",
  );
  expect(() => storedEntryId('{"status":"stored"}')).toThrow("entry identity");
  expect(storedStatus("the status is stored")).toBe(false);
  expect(storedStatus('{"status":"rejected"}')).toBe(false);
});
