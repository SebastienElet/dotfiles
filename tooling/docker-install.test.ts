import {
  type DockerInstallTarget,
  cleanupDockerInstallFixtures,
  runDockerInstallTarget,
} from "./docker-install-test-support.ts";
import { afterAll, describe, expect, test } from "bun:test";

const targets: DockerInstallTarget[] = [
  "firecrawl",
  "scrapling",
  "cloakbrowser",
];

afterAll(cleanupDockerInstallFixtures);

describe.each(targets)("%s Docker installation target", (target) => {
  test("reports an allowed unavailable daemon as skipped", () => {
    const result = runDockerInstallTarget(target, "daemon-unavailable");

    expect(result.exitCode).toBe(0);
    expect(result.stdout).toContain(resultMarker(target, "skipped"));
    expect(result.stderr).toContain("daemon unavailable");
    expect(result.trace).toBe("info\n");
  });

  test("fails when the installation command fails", () => {
    const result = runDockerInstallTarget(target, "command-failure");

    expect(result.exitCode).not.toBe(0);
    expect(result.stdout).not.toContain(resultMarker(target, "verified"));
  });

  test("fails when the promised Docker artifact is absent", () => {
    const result = runDockerInstallTarget(target, "artifact-absent");

    expect(result.exitCode).not.toBe(0);
    expect(result.stdout).not.toContain(resultMarker(target, "verified"));
    expect(result.trace).toContain(oracleCommand(target));
  });

  test("reports verified only when the promised artifact is present", () => {
    const result = runDockerInstallTarget(target, "artifact-present");

    expect(result.exitCode).toBe(0);
    expect(result.stdout).toContain(resultMarker(target, "verified"));
    expect(result.trace).toContain(oracleCommand(target));
  });
});

test("Firecrawl rejects malformed Docker service evidence", () => {
  const result = runDockerInstallTarget("firecrawl", "invalid-evidence");

  expect(result.exitCode).not.toBe(0);
  expect(result.stdout).not.toContain(resultMarker("firecrawl", "verified"));
});

test.each(["scrapling", "cloakbrowser"] as DockerInstallTarget[])(
  "%s rejects an image value that Docker would interpret as an option",
  (target) => {
    const result = runDockerInstallTarget(target, "artifact-present", {
      imageOverride: "--help",
    });

    expect(result.exitCode).not.toBe(0);
    expect(result.stdout).not.toContain(resultMarker(target, "verified"));
  },
);

test.each(targets)(
  "%s fails closed when the Docker CLI is absent",
  (target) => {
    const result = runDockerInstallTarget(target, "artifact-present", {
      dockerProviderAvailable: false,
    });

    expect(result.exitCode).not.toBe(0);
    expect(result.stderr).toContain("Docker CLI unavailable");
    expect(result.stdout).not.toContain(resultMarker(target, "skipped"));
  },
);

test.each(targets)("%s refuses an unknown skip policy", (target) => {
  const result = runDockerInstallTarget(target, "daemon-unavailable", {
    policy: "unknown-policy",
  });

  expect(result.exitCode).not.toBe(0);
  expect(result.stdout).not.toContain(resultMarker(target, "skipped"));
});

test.each(targets)(
  "%s fails closed when daemon skips are forbidden",
  (target) => {
    const result = runDockerInstallTarget(target, "daemon-unavailable", {
      policy: "require-docker",
    });

    expect(result.exitCode).not.toBe(0);
    expect(result.stdout).not.toContain(resultMarker(target, "skipped"));
  },
);

function resultMarker(
  target: DockerInstallTarget,
  result: "skipped" | "verified",
): string {
  return `docker-install target=${target} result=${result}`;
}

function oracleCommand(target: DockerInstallTarget): string {
  return target === "firecrawl"
    ? "ps --services --status running"
    : "image inspect";
}
