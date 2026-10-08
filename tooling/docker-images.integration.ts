import { afterEach, expect, test } from "bun:test";
import {
  cleanupFixtures,
  createFixture,
  run,
} from "./scrapling-mcp-test-support.ts";
import { dockerImages } from "./docker-image.ts";
import { join } from "node:path";
import { runDockerArtifactInstallation } from "./install-docker-artifact.ts";
import { z } from "zod";

const smokeTimeoutMilliseconds = 90_000;
const configurationFailureExitCode = 78;
const stopSeconds = "1";
const containerIdSchema = z.string().regex(/^[0-9a-f]{64}$/u);
const cleanupInspectionSchema = z.object({
  Id: containerIdSchema,
  Config: z.object({ Labels: z.record(z.string(), z.string()).nullable() }),
});

afterEach(cleanupFixtures);

test(
  "installs and verifies both canonical Docker images",
  () => {
    for (const target of ["scrapling", "cloakbrowser"] as const) {
      expect(
        runDockerArtifactInstallation([
          "install",
          target,
          "require-docker",
          dockerImages[target],
        ]),
      ).toBe(0);
      expect(
        runDockerArtifactInstallation([
          "verify",
          target,
          "require-docker",
          dockerImages[target],
        ]),
      ).toBe(0);
    }
  },
  smokeTimeoutMilliseconds,
);

type CloakSmoke = Readonly<{
  docker: string;
  container: string;
  owner: string;
  ownerLabel: string;
  fixture: ReturnType<typeof createFixture>;
}>;
const launcher = join(import.meta.dir, "cloakbrowser");

function createCloakSmoke(): CloakSmoke {
  const docker = Bun.which("docker");
  if (docker === null) {
    throw new Error("Docker CLI unavailable");
  }
  const container = `cloakbrowser-smoke-${crypto.randomUUID()}`;
  const owner = crypto.randomUUID();
  const ownerLabel = "dotfiles.cloakbrowser-smoke";
  const fixture = createFixture(
    {},
    {
      CLOAKBROWSER_CONTAINER: container,
      CLOAKBROWSER_DOCKER_TIMEOUT_MS: "60000",
      CLOAKBROWSER_IMAGE: dockerImages.cloakbrowser,
      SCRAPLING_REAL_DOCKER_BIN: docker,
      SCRAPLING_REAL_OWNER: owner,
      SCRAPLING_REAL_OWNER_LABEL: ownerLabel,
    },
  );
  expect(
    Bun.spawnSync([docker, "container", "inspect", container]).exitCode,
  ).not.toBe(0);
  return { docker, container, owner, ownerLabel, fixture };
}

function inspectContainerId(smoke: CloakSmoke): string {
  const inspected = Bun.spawnSync([
    smoke.docker,
    "inspect",
    "--format",
    "{{.Id}}",
    smoke.container,
  ]);
  expect(inspected.exitCode).toBe(0);
  return containerIdSchema.parse(inspected.stdout.toString().trim());
}

function exerciseCloakSmoke(smoke: CloakSmoke): void {
  expect(run(smoke.fixture, launcher)).toEqual({
    exitCode: 0,
    stderr: "",
    stdout: "",
  });
  const id = inspectContainerId(smoke);
  expect(run(smoke.fixture, launcher).exitCode).toBe(0);
  expect(inspectContainerId(smoke)).toBe(id);
  expect(
    Bun.spawnSync([
      smoke.docker,
      "stop",
      "--time",
      stopSeconds,
      smoke.container,
    ]).exitCode,
  ).toBe(0);
  expect(run(smoke.fixture, launcher).exitCode).toBe(0);
  expect(
    run(
      {
        ...smoke.fixture,
        environment: {
          ...smoke.fixture.environment,
          CLOAKBROWSER_IMAGE: dockerImages.scrapling,
        },
      },
      launcher,
    ).exitCode,
  ).toBe(configurationFailureExitCode);
}

function cleanupCloakSmoke(smoke: CloakSmoke): void {
  const inspection = Bun.spawnSync([
    smoke.docker,
    "inspect",
    "--format",
    "{{json .}}",
    smoke.container,
  ]);
  if (inspection.exitCode !== 0) {
    return;
  }
  const container = cleanupInspectionSchema.parse(
    JSON.parse(inspection.stdout.toString()),
  );
  if (container.Config.Labels?.[smoke.ownerLabel] === smoke.owner) {
    expect(
      Bun.spawnSync([smoke.docker, "rm", "--force", container.Id]).exitCode,
    ).toBe(0);
  }
}

test(
  "creates, reuses and restarts a pinned real CloakBrowser; refuses another image",
  () => {
    const smoke = createCloakSmoke();
    try {
      exerciseCloakSmoke(smoke);
    } finally {
      cleanupCloakSmoke(smoke);
    }
  },
  smokeTimeoutMilliseconds,
);
