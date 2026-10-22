import { afterEach, expect, setDefaultTimeout, test } from "bun:test";
import {
  clean,
  fixture,
  managedLink,
  workerClean,
  workerFixture,
  workerPlist,
  workerState,
} from "./clean-deployment-worker-test-support.ts";
import {
  cleanupDeploymentFixtures,
  expectSuccess,
  pathExists,
} from "./deployment-test-support.ts";
import { dirname, join } from "node:path";
import {
  mkdirSync,
  readFileSync,
  symlinkSync,
  unlinkSync,
  writeFileSync,
} from "node:fs";

afterEach(cleanupDeploymentFixtures);
const workerTestTimeoutMilliseconds = 30_000;
setDefaultTimeout(workerTestTimeoutMilliseconds);

test("removes retired remem skill links after their manifest entries disappear", () => {
  const context = fixture();
  const codex = managedLink(
    context,
    ".agents/skills/remem-memory",
    "harness/skills/remem-memory",
  );
  const claude = join(context.home, ".claude/skills/remem-memory");
  mkdirSync(dirname(claude), { recursive: true });
  symlinkSync(codex.source, claude);
  const foreign = join(context.home, ".cursor/skills/remem-memory");
  mkdirSync(foreign, { recursive: true });
  expect(clean(context).exitCode).toBe(0);
  expect(pathExists(codex.destination)).toBeFalse();
  expect(pathExists(claude)).toBeFalse();
  expect(pathExists(foreign)).toBeTrue();
  expect(clean(context).exitCode).toBe(0);
});

test("stops the recognized loaded worker before removing its plist and config, then replays", () => {
  const context = workerFixture();
  const result = workerClean(context);
  expectSuccess(result);
  expect(pathExists(workerPlist(context))).toBeFalse();
  expect(pathExists(join(context.home, ".remem/config.toml"))).toBeFalse();
  expect(workerState(context).definition).toBeNull();
  expect(
    workerState(context).trace.find((entry) => entry.command === "launchctl")
      ?.configPresent,
  ).toBeTrue();
  expectSuccess(workerClean(context));
  expect(
    workerState(context).trace.filter((entry) => entry.command === "launchctl"),
  ).toHaveLength(1);
});

test("inspects the owned loaded worker without stopping or deleting", () => {
  const context = workerFixture();
  expectSuccess(workerClean(context, false));
  expect(pathExists(workerPlist(context))).toBeTrue();
  expect(pathExists(join(context.home, ".remem/config.toml"))).toBeTrue();
  expect(
    workerState(context).trace.some((entry) => entry.command === "launchctl"),
  ).toBeFalse();
});

test("removes an owned plist for a worker proven absent without bootout", () => {
  const context = workerFixture({ loaded: false });
  expectSuccess(workerClean(context));
  expect(pathExists(workerPlist(context))).toBeFalse();
  expect(
    workerState(context).trace.some((entry) => entry.command === "launchctl"),
  ).toBeFalse();
});

test.each([
  { foreignDisk: true },
  { foreignService: true },
  { conflictingProgram: true },
  { failure: "inspect" },
  { failure: "timeout" },
  { failure: "convert" },
  { failure: "invalid" },
  { failure: "bootout" },
  { failure: "retained" },
])(
  "refuses unsafe worker cleanup before removing configuration: %j",
  (
    options: Readonly<{
      foreignDisk?: boolean;
      foreignService?: boolean;
      conflictingProgram?: boolean;
      failure?: string;
    }>,
  ) => {
    const context = workerFixture(options);
    const before = readFileSync(workerPlist(context), "utf8");
    const unrelated = join(context.home, ".codex/AGENTS.md");
    mkdirSync(dirname(unrelated), { recursive: true });
    writeFileSync(unrelated, "preserve until safe\n");
    const result = workerClean(context);
    expect(result.exitCode).not.toBe(0);
    expect(readFileSync(workerPlist(context), "utf8")).toBe(before);
    expect(pathExists(join(context.home, ".remem/config.toml"))).toBeTrue();
    expect(readFileSync(unrelated, "utf8")).toBe("preserve until safe\n");
  },
);

test("validates shared configuration before stopping the loaded worker", () => {
  const context = workerFixture();
  mkdirSync(join(context.home, ".codex"));
  writeFileSync(join(context.home, ".codex/config.toml"), "[invalid\n");
  expect(workerClean(context).exitCode).not.toBe(0);
  expect(
    workerState(context).trace.some((entry) => entry.command === "launchctl"),
  ).toBeFalse();
  expect(pathExists(workerPlist(context))).toBeTrue();
});

test("stops an owned loaded worker whose plist is already absent before removing its config", () => {
  const context = workerFixture();
  unlinkSync(workerPlist(context));
  expectSuccess(workerClean(context));
  expect(workerState(context).definition).toBeNull();
  expect(pathExists(join(context.home, ".remem/config.toml"))).toBeFalse();
});
