import { afterEach, describe, expect, setDefaultTimeout, test } from "bun:test";
import {
  chmodSync,
  copyFileSync,
  mkdirSync,
  symlinkSync,
  unlinkSync,
  utimesSync,
  writeFileSync,
} from "node:fs";
import {
  cleanupDeploymentFixtures,
  createDeploymentFixture,
  expectSuccess,
  fileIdentity,
  linkTarget,
  pathExists,
  project,
  requireCommand,
  runMake,
} from "./deployment-test-support.ts";
import { dirname, join } from "node:path";

afterEach(cleanupDeploymentFixtures);
const deploymentTimeoutMilliseconds = 15_000;
setDefaultTimeout(deploymentTimeoutMilliseconds);

const prerequisites = [
  "Cargo.lock",
  "Cargo.toml",
  "src/decision.rs",
  "src/environment.rs",
  "src/error.rs",
  "src/event.rs",
  "src/lib.rs",
  "src/main.rs",
  "src/run.rs",
  "src/state.rs",
  "src/transcript.rs",
  "tests/cli.rs",
  "tests/cli/runtime_parity.rs",
  "tests/concurrency.rs",
  "tests/decision.rs",
  "tests/event.rs",
  "tests/transcript.rs",
  "tests/transcript/numeric.rs",
] as const;
const oldTime = new Date("2020-01-01T00:00:00Z");
const releaseTime = new Date("2021-01-01T00:00:00Z");
const newTime = new Date("2022-01-01T00:00:00Z");
const executableFileMode = 0o755;
type CommandResult = ReturnType<typeof runMake>;
type DeploymentFixture = ReturnType<typeof createDeploymentFixture>;

describe("agent-handoff release target", () => {
  test.each([...prerequisites])(
    "rebuilds once after %s changes",
    (relativePath) => {
      const fixture = createDeploymentFixture(
        `handoff-build-${relativePath.replaceAll("/", "-")}`,
      );
      const prepared = prepareAgentHandoff(fixture);
      utimesSync(join(prepared.crate, relativePath), newTime, newTime);

      const rebuilt = runAgentHandoffMake(fixture, prepared.release);
      expectSuccess(rebuilt);
      expect(rebuilt.stdout).toContain(`${fixture.bin}/cargo build --release`);

      const replayed = runAgentHandoffMake(fixture, prepared.release);
      expectSuccess(replayed);
      expect(replayed.stdout).not.toContain("cargo build --release");
    },
  );

  test("refuses a successful build that produces no release binary", () => {
    const fixture = createDeploymentFixture("handoff-missing-release");
    const prepared = prepareAgentHandoff(fixture);
    unlinkSync(prepared.release);

    const result = runAgentHandoffMake(fixture, prepared.release);

    expect(result.exitCode).not.toBe(0);
    expect(pathExists(prepared.release)).toBeFalse();
  });
});

describe("agent-handoff deployed link", () => {
  test("does not rebuild or revalidate an up-to-date deployed link", () => {
    const fixture = createDeploymentFixture("handoff-current");
    const prepared = prepareAgentHandoff(fixture);
    mkdirSync(dirname(prepared.destination), { recursive: true });
    symlinkSync(prepared.release, prepared.destination);

    const result = runAgentHandoffMake(fixture, prepared.destination);

    expectSuccess(result);
    expect(result.stdout).not.toContain("cargo build --release");
    expect(result.stdout).not.toContain(`readlink ${prepared.destination}`);
    expect(linkTarget(prepared.destination)).toBe(prepared.release);
  });

  test("creates an absent deployed link", () => {
    const fixture = createDeploymentFixture("handoff-absent");
    const prepared = prepareAgentHandoff(fixture);

    expectSuccess(runAgentHandoffMake(fixture, prepared.destination));

    expect(linkTarget(prepared.destination)).toBe(prepared.release);
  });
});

describe("agent-handoff historical migration", () => {
  test("migrates the exact historical link when the release is newer", () => {
    const fixture = createDeploymentFixture("handoff-historical-stale");
    const prepared = prepareAgentHandoff(fixture);
    installHistoricalLink(prepared);
    utimesSync(prepared.crate, oldTime, oldTime);

    expectSuccess(runAgentHandoffMake(fixture, prepared.destination));

    expect(linkTarget(prepared.destination)).toBe(prepared.release);
  });

  test("leaves an up-to-date historical link untouched", () => {
    const fixture = createDeploymentFixture("handoff-historical-current");
    const prepared = prepareAgentHandoff(fixture);
    installHistoricalLink(prepared);
    utimesSync(prepared.crate, newTime, newTime);

    const result = runAgentHandoffMake(fixture, prepared.destination);

    expectSuccess(result);
    expect(result.stdout).not.toContain(`readlink ${prepared.destination}`);
    expect(linkTarget(prepared.destination)).toBe(prepared.crate);
  });
});

describe("agent-handoff up-to-date unexpected destination", () => {
  test("leaves an up-to-date unexpected file untouched", () => {
    const fixture = createDeploymentFixture("handoff-current-file");
    const prepared = prepareAgentHandoff(fixture);
    mkdirSync(dirname(prepared.destination), { recursive: true });
    writeFileSync(prepared.destination, "keep\n");
    utimesSync(prepared.destination, newTime, newTime);
    const before = fileIdentity(prepared.destination);

    const result = runAgentHandoffMake(fixture, prepared.destination);

    expectSuccess(result);
    expect(result.stdout).not.toContain(`readlink ${prepared.destination}`);
    expect(fileIdentity(prepared.destination)).toEqual(before);
  });
});

describe("agent-handoff stale unexpected destination", () => {
  test.each([
    ["file", false, false],
    ["link", true, false],
    ["dangling link", true, true],
  ] as const)(
    "refuses a stale unexpected %s without changing it",
    (name, linked, dangling) => {
      const fixture = createDeploymentFixture(
        `handoff-stale-${name.replaceAll(" ", "-")}`,
      );
      const prepared = prepareAgentHandoff(fixture);
      const unexpected = join(fixture.root, "unexpected");
      mkdirSync(dirname(prepared.destination), { recursive: true });
      if (!dangling) {
        writeFileSync(unexpected, "keep\n");
        utimesSync(unexpected, oldTime, oldTime);
      }
      if (linked) {
        symlinkSync(unexpected, prepared.destination);
      } else {
        writeFileSync(prepared.destination, "keep\n");
        utimesSync(prepared.destination, oldTime, oldTime);
      }
      const before = linked
        ? linkTarget(prepared.destination)
        : fileIdentity(prepared.destination);

      const result = runAgentHandoffMake(fixture, prepared.destination);

      expect(result.exitCode).not.toBe(0);
      expect(result.stderr).not.toBe("");
      expect(
        linked
          ? linkTarget(prepared.destination)
          : fileIdentity(prepared.destination),
      ).toEqual(before);
    },
  );
});

type PreparedAgentHandoff = Readonly<{
  crate: string;
  destination: string;
  release: string;
}>;

function prepareAgentHandoff(fixture: DeploymentFixture): PreparedAgentHandoff {
  const crate = join(fixture.repository, "tooling", "agent-handoff");
  for (const relativePath of prerequisites) {
    const destination = join(crate, relativePath);
    mkdirSync(dirname(destination), { recursive: true });
    copyFileSync(
      join(project, "tooling", "agent-handoff", relativePath),
      destination,
    );
    utimesSync(destination, oldTime, oldTime);
  }
  const release = join(crate, "target", "release", "agent-handoff");
  mkdirSync(dirname(release), { recursive: true });
  writeFileSync(release, "binary");
  chmodSync(release, executableFileMode);
  utimesSync(release, releaseTime, releaseTime);
  const noOperation = requireCommand("true");
  symlinkSync(noOperation, join(fixture.bin, "brew"));
  symlinkSync(noOperation, join(fixture.bin, "cargo"));
  return {
    crate,
    destination: join(fixture.home, ".local", "bin", "agent-handoff"),
    release,
  };
}

function installHistoricalLink(prepared: PreparedAgentHandoff): void {
  mkdirSync(dirname(prepared.destination), { recursive: true });
  symlinkSync(prepared.crate, prepared.destination);
}

function runAgentHandoffMake(
  fixture: DeploymentFixture,
  target: string,
): CommandResult {
  return runMake(fixture, [target], {
    variables: { BREW_BIN: fixture.bin },
  });
}
