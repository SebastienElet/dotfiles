import {
  type Scenario,
  calls,
  cleanupFixtures,
  createFixture,
  run,
} from "./scrapling-mcp-test-support.ts";
import { afterEach, expect, test } from "bun:test";
import { join } from "node:path";

const launcher = join(import.meta.dir, "cloakbrowser");
const configurationFailureExitCode = 78;
const usageFailureExitCode = 64;
const timeoutFailureExitCode = 75;
const environment = { CLOAKBROWSER_TEST: "1" };

afterEach(cleanupFixtures);

test.each([false, true])(
  "reuses a compatible CloakBrowser (running=%s)",
  (running) => {
    const fixture = createFixture({ present: true, running }, environment);
    expect(run(fixture, launcher).exitCode).toBe(0);
    expect(calls(fixture).some((call) => call[0] === "run")).toBe(false);
    expect(calls(fixture).some((call) => call[0] === "start")).toBe(!running);
  },
);

test("creates and reuses one named CloakBrowser", () => {
  const fixture = createFixture({}, environment);
  expect(run(fixture, launcher).exitCode).toBe(0);
  expect(run(fixture, launcher).exitCode).toBe(0);
  expect(calls(fixture).filter((call) => call[0] === "run")).toHaveLength(1);
});

test.each([
  { compatible: false },
  { imageMismatch: true },
  { differentDigest: true },
  { invalidImageInspect: true },
])("refuses incompatible CloakBrowser evidence %j", (scenario: Scenario) => {
  const fixture = createFixture({ ...scenario, present: true }, environment);
  expect(run(fixture, launcher).exitCode).toBe(configurationFailureExitCode);
  expect(
    calls(fixture).some((call) => call[0] === "start" || call[0] === "run"),
  ).toBe(false);
});

test.each([
  { infoFailure: true },
  { inspectFailure: true, present: true },
  { imageInspectFailure: true, present: true },
  { runFailure: true },
])("reports Docker failure %j", (scenario: Scenario) => {
  const fixture = createFixture(scenario, environment);
  const outcome = run(fixture, launcher);
  expect(outcome.exitCode).not.toBe(0);
  expect(outcome.stderr).not.toBe("");
});

test.each(["", "image:tag", "image:tag@sha256:abc", "--help"])(
  "refuses invalid image %s before Docker",
  (image) => {
    const fixture = createFixture(
      {},
      { ...environment, CLOAKBROWSER_IMAGE: image },
    );
    expect(run(fixture, launcher).exitCode).toBe(usageFailureExitCode);
    expect(calls(fixture)).toEqual([]);
  },
);

test("times out inspection without starting CloakBrowser", () => {
  const fixture = createFixture(
    { present: true, hang: "image inspect" },
    { ...environment, CLOAKBROWSER_DOCKER_TIMEOUT_MS: "300" },
  );
  const outcome = run(fixture, launcher);
  expect(outcome.exitCode).toBe(timeoutFailureExitCode);
  expect(outcome.stderr).toContain("timed out");
  expect(calls(fixture).some((call) => call[0] === "start")).toBe(false);
});

test("starts the inspected CloakBrowser when its name is replaced", () => {
  const fixture = createFixture(
    { present: true, replaceAfterInspection: true },
    environment,
  );
  expect(run(fixture, launcher).exitCode).toBe(0);
});
