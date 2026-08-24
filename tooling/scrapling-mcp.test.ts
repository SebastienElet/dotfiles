import { afterEach, expect, test } from "bun:test";
import {
  calls,
  cleanupFixtures,
  createFixture,
  result,
  run,
} from "./scrapling-mcp-test-support.ts";

type Fixture = ReturnType<typeof createFixture>;
type Scenario = Parameters<typeof createFixture>[0];

afterEach(cleanupFixtures);

const concurrentProcessCount = 2;
const configurationFailureExitCode = 78;
const usageFailureExitCode = 64;
const timeoutFailureExitCode = 75;
const integrationTimeoutMilliseconds = 90_000;

type DockerSmokeFixture = Readonly<{
  container: string;
  docker: string;
  fixture: Fixture;
  volume: string;
  volumeLabel: string;
  volumeOwner: string;
}>;

function createDockerSmokeFixture(docker: string): DockerSmokeFixture {
  const container = `scrapling-mcp-smoke-${crypto.randomUUID()}`;
  const volume = `scrapling-profiles-smoke-${crypto.randomUUID()}`;
  const volumeOwner = crypto.randomUUID();
  const volumeLabel = "dotfiles.scrapling-smoke";
  const fixture = createFixture(
    {},
    {
      SCRAPLING_CONTAINER: container,
      SCRAPLING_DOCKER_TIMEOUT_MS: "60000",
      SCRAPLING_IMAGE: "alpine:3.22",
      SCRAPLING_REAL_DOCKER_BIN: docker,
      SCRAPLING_REAL_OWNER: volumeOwner,
      SCRAPLING_REAL_OWNER_LABEL: volumeLabel,
      SCRAPLING_REAL_PROFILE_VOLUME: volume,
    },
  );
  return { container, docker, fixture, volume, volumeLabel, volumeOwner };
}

function assertDockerSmokeAbsent(smoke: DockerSmokeFixture): void {
  const existingContainer = Bun.spawnSync([
    smoke.docker,
    "container",
    "inspect",
    smoke.container,
  ]);
  expect(existingContainer.exitCode).not.toBe(0);
  const existingVolume = Bun.spawnSync([
    smoke.docker,
    "volume",
    "inspect",
    smoke.volume,
  ]);
  expect(existingVolume.exitCode).not.toBe(0);
}

function createOwnedVolume(smoke: DockerSmokeFixture): void {
  const createdVolume = Bun.spawnSync([
    smoke.docker,
    "volume",
    "create",
    "--label",
    `${smoke.volumeLabel}=${smoke.volumeOwner}`,
    smoke.volume,
  ]);
  expect(createdVolume.exitCode).toBe(0);
  const owner = Bun.spawnSync([
    smoke.docker,
    "volume",
    "inspect",
    "--format",
    `{{ index .Labels "${smoke.volumeLabel}" }}`,
    smoke.volume,
  ]);
  expect(owner.stdout.toString().trim()).toBe(smoke.volumeOwner);
}

function exerciseDockerSmoke(smoke: DockerSmokeFixture): void {
  const expected = { exitCode: 0, stderr: "", stdout: "mcp smoke\n" };
  expect(run(smoke.fixture)).toEqual(expected);
  expect(run(smoke.fixture)).toEqual(expected);
  const inspection = Bun.spawnSync([smoke.docker, "inspect", smoke.container]);
  expect(inspection.exitCode).toBe(0);
}

function cleanupDockerSmoke(smoke: DockerSmokeFixture): void {
  const ownedContainer = Bun.spawnSync([
    smoke.docker,
    "container",
    "inspect",
    "--format",
    `{{.Id}} {{ index .Config.Labels "${smoke.volumeLabel}" }}`,
    smoke.container,
  ]);
  const [containerId, containerOwner] = ownedContainer.stdout
    .toString()
    .trim()
    .split(" ");
  if (
    ownedContainer.exitCode === 0 &&
    containerOwner === smoke.volumeOwner &&
    containerId !== undefined
  ) {
    Bun.spawnSync([smoke.docker, "rm", "--force", containerId]);
  }
}

function runDockerSmoke(): void {
  const docker = Bun.which("docker");
  expect(docker).not.toBeNull();
  if (docker === null) {
    throw new Error("Docker is unavailable");
  }
  const smoke = createDockerSmokeFixture(docker);
  try {
    assertDockerSmokeAbsent(smoke);
    createOwnedVolume(smoke);
    exerciseDockerSmoke(smoke);
  } finally {
    cleanupDockerSmoke(smoke);
  }
}

test("reuses a compatible running container and preserves MCP output", () => {
  const fixture = createFixture({
    execStdout: "mcp response\n",
    present: true,
    running: true,
  });

  expect(run(fixture)).toEqual({
    exitCode: 0,
    stderr: "",
    stdout: "mcp response\n",
  });
  expect(calls(fixture).map((call) => call[0])).toEqual([
    "info",
    "container",
    "container",
    "exec",
  ]);
});

test("starts a compatible stopped container", () => {
  const fixture = createFixture({ present: true });

  expect(run(fixture).exitCode).toBe(0);
  expect(calls(fixture).map((call) => call[0])).toEqual([
    "info",
    "container",
    "container",
    "start",
    "exec",
  ]);
});

test("creates the established container when absent", () => {
  const fixture = createFixture();

  expect(run(fixture).exitCode).toBe(0);
  expect(calls(fixture)).toContainEqual([
    "run",
    "--detach",
    "--name",
    "scrapling-mcp",
    "--add-host=host.docker.internal:host-gateway",
    "--volume",
    "scrapling-profiles:/profiles",
    "--entrypoint",
    "sleep",
    "pyd4vinci/scrapling",
    "infinity",
  ]);
});

test("concurrent entry points converge on one named container", async () => {
  const fixture = createFixture({ concurrent: true });
  const outcomes = await Promise.all([result(fixture), result(fixture)]);

  expect(outcomes.map(({ exitCode }) => exitCode)).toEqual([0, 0]);
  expect(calls(fixture).filter((call) => call[0] === "exec")).toHaveLength(
    concurrentProcessCount,
  );
});

test("refuses an incompatible existing container", () => {
  const fixture = createFixture({ compatible: false, present: true });
  const outcome = run(fixture);

  expect(outcome.exitCode).toBe(configurationFailureExitCode);
  expect(outcome.stderr).toContain("incompatible");
  expect(calls(fixture).some((call) => call[0] === "exec")).toBeFalse();
});

test.each([
  ["Docker daemon unavailable", { infoFailure: true }],
  ["cannot list", { listFailure: true }],
  ["cannot inspect", { inspectFailure: true, present: true }],
  ["cannot start", { present: true, startFailure: true }],
  ["cannot create", { runFailure: true }],
])("reports %s", (...[_label, scenario]: readonly [string, Scenario]) => {
  const outcome = run(createFixture(scenario));

  expect(outcome.exitCode).not.toBe(0);
  expect(outcome.stderr).not.toBe("");
});

test("propagates the MCP process status and stderr", () => {
  const fixture = createFixture({
    execExit: 42,
    execStderr: "mcp failed\n",
    present: true,
    running: true,
  });

  expect(run(fixture)).toEqual({
    exitCode: 42,
    stderr: "mcp failed\n",
    stdout: "",
  });
});

test("rejects malformed and non-UTF-8 Docker evidence", () => {
  for (const scenario of [
    { invalidInspect: true, present: true },
    { invalidUtf8: "container ls" },
  ]) {
    const outcome = run(createFixture(scenario));
    expect(outcome.exitCode).not.toBe(0);
    expect(outcome.stdout).toBe("");
    expect(outcome.stderr).toMatch(/invalid (?<reason>inspection data|UTF-8)/u);
  }
});

test("rejects unsafe environment overrides before Docker runs", () => {
  for (const environment of [
    { SCRAPLING_CONTAINER: "--privileged" },
    { SCRAPLING_IMAGE: "-v /:/host" },
    { SCRAPLING_DOCKER_TIMEOUT_MS: "0" },
  ]) {
    const fixture = createFixture({}, environment);
    expect(run(fixture).exitCode).toBe(usageFailureExitCode);
    expect(calls(fixture)).toEqual([]);
  }
});

test("times out a blocked lifecycle command", () => {
  const fixture = createFixture(
    { hang: "container ls" },
    { SCRAPLING_DOCKER_TIMEOUT_MS: "100" },
  );
  const outcome = run(fixture);

  expect(outcome.exitCode).toBe(timeoutFailureExitCode);
  expect(outcome.stderr).toContain("timed out");
});

test.skipIf(process.env.SCRAPLING_DOCKER_SMOKE !== "1")(
  "uses an isolated real Docker lifecycle when explicitly enabled",
  runDockerSmoke,
  integrationTimeoutMilliseconds,
);
