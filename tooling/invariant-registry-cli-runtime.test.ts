import {
  active,
  candidate,
  diagnosticCodes,
  oracle,
  registry,
  validateInvariantRegistry,
  validationOptions,
} from "./invariant-registry-test-support.ts";
import { afterEach, expect, test } from "bun:test";
import {
  cleanup,
  createRegistry,
  runRegistryCli,
} from "./invariant-registry-cli.test-support.ts";
import { validateInvariantRegistryText } from "./invariant-registry-repository-validator.ts";

const failingOraclePath =
  "./tooling/invariant-registry-fixtures/.runtime/failing.test.ts";

afterEach(cleanup);

test("fails when a verified record's declared executable oracle fails", async () => {
  const declaredOracle = {
    ...oracle,
    invocation: ["bun", "test", failingOraclePath],
    testPath: failingOraclePath,
  };
  const verifiedRecord = active({
    oracle: declaredOracle,
    verification: {
      state: "verified",
      lastRun: {
        environment: "runtime CLI fixture",
        oracle: {
          invocation: declaredOracle.invocation,
          name: declaredOracle.name,
          testPath: declaredOracle.testPath,
        },
        outcome: "passed",
        ranAt: "2026-09-03T09:00:00.000Z",
      },
    },
  });
  const registryText = JSON.stringify(registry(verifiedRecord));
  expect(() =>
    validateInvariantRegistryText(registryText, `${import.meta.dir}/..`),
  ).not.toThrow();
  const path = await createRegistry(registryText);

  const outcome = await runRegistryCli(path);

  expect(outcome.exitCode).not.toBe(0);
  expect(outcome.stdout).toBe("");
  expect(outcome.stderr).toContain("declared oracle failed");
});

test("rejects a verified runtime invocation without its matching record oracle", () => {
  const diagnostics = validateInvariantRegistry(
    registry(
      candidate({
        verification: {
          state: "verified",
          lastRun: {
            environment: "runtime CLI fixture",
            oracle: {
              invocation: oracle.invocation,
              name: oracle.name,
              testPath: oracle.testPath,
            },
            outcome: "passed",
            ranAt: "2026-09-03T09:00:00.000Z",
          },
        },
      }),
    ),
    validationOptions(),
  );

  expect(diagnosticCodes(diagnostics)).toContain("missing-oracle");
});
