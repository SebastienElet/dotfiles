import {
  type Scenario,
  calls,
  cleanupFixtures,
  createFixture,
  run,
} from "./scrapling-mcp-test-support.ts";
import { afterEach, expect, test } from "bun:test";

const configurationFailureExitCode = 78;
const timeoutFailureExitCode = 75;
afterEach(cleanupFixtures);
test("refuses the wrong image ID even when the requested reference matches", () => {
  const fixture = createFixture({
    present: true,
    running: true,
    imageMismatch: true,
  });
  expect(run(fixture).exitCode).toBe(configurationFailureExitCode);
  expect(
    calls(fixture).some((call) => call[0] === "exec" || call[0] === "start"),
  ).toBe(false);
});

test.each([
  { differentDigest: true },
  { invalidImageInspect: true },
  { imageInspectFailure: true },
])("refuses invalid required image evidence %j", (scenario: Scenario) => {
  const fixture = createFixture({ ...scenario, present: true });
  expect(run(fixture).exitCode).not.toBe(0);
  expect(
    calls(fixture).some((call) => call[0] === "start" || call[0] === "exec"),
  ).toBe(false);
});

test("times out required image inspection without starting Scrapling", () => {
  const fixture = createFixture(
    { present: true, hang: "image inspect" },
    { SCRAPLING_DOCKER_TIMEOUT_MS: "300" },
  );
  expect(run(fixture).exitCode).toBe(timeoutFailureExitCode);
  expect(calls(fixture).some((call) => call[0] === "start")).toBe(false);
});

test("keeps the inspected container identity when its name is replaced", () => {
  const fixture = createFixture({
    present: true,
    replaceAfterInspection: true,
  });
  expect(run(fixture).exitCode).toBe(0);
});

test("refuses malformed RepoDigests evidence", () => {
  const fixture = createFixture({ present: true, malformedDigest: true });
  expect(run(fixture).exitCode).toBe(configurationFailureExitCode);
});

test("refuses a timed-out daemon probe even if Docker exits successfully", () => {
  const fixture = createFixture(
    { delayedOutput: true },
    { SCRAPLING_DOCKER_TIMEOUT_MS: "300" },
  );
  const outcome = run(fixture);
  expect(outcome.exitCode).toBe(timeoutFailureExitCode);
  expect(outcome.stderr).toContain("timed out");
  expect(calls(fixture)).toHaveLength(1);
});
