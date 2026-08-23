import { afterEach, describe, expect, test } from "bun:test";
import {
  calls,
  cleanupFixtures,
  createFixture,
  result,
  run,
  start,
} from "./scrapling-mcp-test-support.ts";

afterEach(cleanupFixtures);

describe("scrapling-mcp entry point", () => {
  test("reuses a compatible running container and preserves MCP output", () => {
    const fixture = createFixture({
      present: true,
      running: true,
      execStdout: "mcp response\n",
    });

    expect(run(fixture)).toEqual({
      exitCode: 0,
      stdout: "mcp response\n",
      stderr: "",
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

    const outcomes = await Promise.all([
      result(start(fixture)),
      result(start(fixture)),
    ]);

    expect(outcomes.map(({ exitCode }) => exitCode)).toEqual([0, 0]);
    expect(calls(fixture).filter((call) => call[0] === "exec")).toHaveLength(2);
  });

  test("refuses an incompatible existing container", () => {
    const fixture = createFixture({ present: true, compatible: false });

    const outcome = run(fixture);

    expect(outcome.exitCode).toBe(78);
    expect(outcome.stderr).toContain("incompatible");
    expect(calls(fixture).some((call) => call[0] === "exec")).toBeFalse();
  });

  test.each([
    ["Docker daemon unavailable", { infoFailure: true }],
    ["cannot list", { listFailure: true }],
    ["cannot inspect", { present: true, inspectFailure: true }],
    ["cannot start", { present: true, startFailure: true }],
    ["cannot create", { runFailure: true }],
  ])("reports %s", (_label, scenario) => {
    const outcome = run(createFixture(scenario));

    expect(outcome.exitCode).not.toBe(0);
    expect(outcome.stderr).not.toBe("");
  });

  test("propagates the MCP process status and stderr", () => {
    const fixture = createFixture({
      present: true,
      running: true,
      execExit: 42,
      execStderr: "mcp failed\n",
    });

    expect(run(fixture)).toEqual({
      exitCode: 42,
      stdout: "",
      stderr: "mcp failed\n",
    });
  });

  test("rejects malformed and non-UTF-8 Docker evidence", () => {
    for (const scenario of [
      { present: true, invalidInspect: true },
      { invalidUtf8: "container ls" },
    ]) {
      const outcome = run(createFixture(scenario));
      expect(outcome.exitCode).not.toBe(0);
      expect(outcome.stdout).toBe("");
      expect(outcome.stderr).toMatch(/invalid (inspection data|UTF-8)/);
    }
  });

  test("rejects unsafe environment overrides before Docker runs", () => {
    for (const environment of [
      { SCRAPLING_CONTAINER: "--privileged" },
      { SCRAPLING_IMAGE: "-v /:/host" },
      { SCRAPLING_DOCKER_TIMEOUT_MS: "0" },
    ]) {
      const fixture = createFixture({}, environment);
      expect(run(fixture).exitCode).toBe(64);
      expect(calls(fixture)).toEqual([]);
    }
  });

  test("times out a blocked lifecycle command", () => {
    const fixture = createFixture(
      { hang: "container ls" },
      { SCRAPLING_DOCKER_TIMEOUT_MS: "100" },
    );

    const outcome = run(fixture);

    expect(outcome.exitCode).toBe(75);
    expect(outcome.stderr).toContain("timed out");
  });

  test.skipIf(process.env.SCRAPLING_DOCKER_SMOKE !== "1")(
    "uses an isolated real Docker lifecycle when explicitly enabled",
    () => {
      const docker = Bun.which("docker");
      expect(docker).not.toBeNull();
      const container = `scrapling-mcp-smoke-${crypto.randomUUID()}`;
      const volume = `scrapling-profiles-smoke-${crypto.randomUUID()}`;
      const volumeOwner = crypto.randomUUID();
      const volumeLabel = "dotfiles.scrapling-smoke";
      const fixture = createFixture(
        {},
        {
          SCRAPLING_CONTAINER: container,
          SCRAPLING_IMAGE: "alpine:3.22",
          SCRAPLING_REAL_DOCKER_BIN: docker!,
          SCRAPLING_REAL_PROFILE_VOLUME: volume,
          SCRAPLING_REAL_OWNER: volumeOwner,
          SCRAPLING_REAL_OWNER_LABEL: volumeLabel,
          SCRAPLING_DOCKER_TIMEOUT_MS: "60000",
        },
      );
      try {
        const existingContainer = Bun.spawnSync([
          docker!,
          "container",
          "inspect",
          container,
        ]);
        expect(existingContainer.exitCode).not.toBe(0);
        const existingVolume = Bun.spawnSync([
          docker!,
          "volume",
          "inspect",
          volume,
        ]);
        expect(existingVolume.exitCode).not.toBe(0);
        const createdVolume = Bun.spawnSync([
          docker!,
          "volume",
          "create",
          "--label",
          `${volumeLabel}=${volumeOwner}`,
          volume,
        ]);
        expect(createdVolume.exitCode).toBe(0);
        const owner = Bun.spawnSync([
          docker!,
          "volume",
          "inspect",
          "--format",
          `{{ index .Labels "${volumeLabel}" }}`,
          volume,
        ]);
        expect(owner.stdout.toString().trim()).toBe(volumeOwner);
        expect(run(fixture)).toEqual({
          exitCode: 0,
          stdout: "mcp smoke\n",
          stderr: "",
        });
        expect(run(fixture)).toEqual({
          exitCode: 0,
          stdout: "mcp smoke\n",
          stderr: "",
        });
        const inspection = Bun.spawnSync([docker!, "inspect", container]);
        expect(inspection.exitCode).toBe(0);
      } finally {
        const ownedContainer = Bun.spawnSync([
          docker!,
          "container",
          "inspect",
          "--format",
          `{{.Id}} {{ index .Config.Labels "${volumeLabel}" }}`,
          container,
        ]);
        const [containerId, containerOwner] = ownedContainer.stdout
          .toString()
          .trim()
          .split(" ");
        if (ownedContainer.exitCode === 0 && containerOwner === volumeOwner) {
          Bun.spawnSync([docker!, "rm", "--force", containerId!]);
        }
      }
    },
    90_000,
  );
});
