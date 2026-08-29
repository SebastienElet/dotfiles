import {
  type Fixture,
  createParityFixture as createFixture,
  runLegacy,
  runRust,
} from "./agent-handoff-parity-support.ts";
import { afterEach, expect, test } from "bun:test";
import {
  concurrentParityCase,
  homeFallbackCase,
  parityCases,
  validClaudeWindowCase,
} from "./agent-handoff-parity-cases.ts";
import { rmSync } from "node:fs";
import { runtimeParityCases } from "./agent-handoff-parity-runtime-cases.ts";

const concurrentProcessCount = 3;
const fixtures: Fixture[] = [];
const noOutput = {
  exitCode: 0,
  stderr: new Uint8Array(),
  stdout: new Uint8Array(),
};
const allParityCases = [...parityCases, ...runtimeParityCases];

type ConcurrentSetup = Readonly<{
  environment: Readonly<Record<string, string>>;
  fixture: Fixture;
  input: Uint8Array;
}>;

afterEach(() => {
  for (const fixture of fixtures.splice(0)) {
    rmSync(fixture.root, { force: true, recursive: true });
  }
});

function createParityFixture(): Fixture {
  const fixture = createFixture();
  fixtures.push(fixture);
  return fixture;
}

function prepareConcurrentFixture(): ConcurrentSetup {
  const fixture = createParityFixture();
  concurrentParityCase.prepare?.(fixture);
  return {
    environment: concurrentParityCase.environment(fixture),
    fixture,
    input: concurrentParityCase.input(fixture),
  };
}

test.each(allParityCases)(
  "matches Bun for $name",
  async ({ input, environment, prepare }) => {
    const legacyFixture = createParityFixture();
    const rustFixture = createParityFixture();
    prepare?.(legacyFixture);
    prepare?.(rustFixture);
    const legacy = await runLegacy(
      input(legacyFixture),
      environment(legacyFixture),
      legacyFixture,
    );
    const rust = await runRust(
      input(rustFixture),
      environment(rustFixture),
      rustFixture,
    );
    expect(rust).toEqual(legacy);
  },
);

test("legacy valid Claude context window changes the default decision", async () => {
  const fixture = createParityFixture();
  validClaudeWindowCase.prepare?.(fixture);
  const result = await runLegacy(
    validClaudeWindowCase.input(fixture),
    validClaudeWindowCase.environment(fixture),
    fixture,
  );
  expect(result).toEqual(noOutput);
});

test("legacy empty XDG_STATE_HOME falls back to HOME", async () => {
  const fixture = createParityFixture();
  homeFallbackCase.prepare?.(fixture);
  const result = await runLegacy(
    homeFallbackCase.input(fixture),
    homeFallbackCase.environment(fixture),
    fixture,
  );
  expect(result).toEqual(noOutput);
});

test(`matches Bun for ${concurrentParityCase.name}`, async () => {
  const legacySetup = prepareConcurrentFixture();
  const rustSetup = prepareConcurrentFixture();
  const legacy = await Promise.all(
    Array.from({ length: concurrentProcessCount }, () =>
      runLegacy(
        legacySetup.input,
        legacySetup.environment,
        legacySetup.fixture,
      ),
    ),
  );
  const legacyKeys: string[] = [];
  let legacyBlockCount = 0;
  for (const result of legacy) {
    legacyBlockCount += Number(result.stdout.length > 0);
    legacyKeys.push(
      `${result.exitCode}:${Buffer.from(result.stdout).toString("hex")}:${Buffer.from(result.stderr).toString("hex")}`,
    );
  }
  expect(legacyBlockCount).toBe(1);

  const rust = await Promise.all(
    Array.from({ length: concurrentProcessCount }, () =>
      runRust(rustSetup.input, rustSetup.environment, rustSetup.fixture),
    ),
  );
  const rustKeys: string[] = [];
  let rustBlockCount = 0;
  for (const result of rust) {
    rustBlockCount += Number(result.stdout.length > 0);
    rustKeys.push(
      `${result.exitCode}:${Buffer.from(result.stdout).toString("hex")}:${Buffer.from(result.stderr).toString("hex")}`,
    );
  }
  expect(rustBlockCount).toBe(1);
  expect(rustKeys.toSorted()).toEqual(legacyKeys.toSorted());
});
