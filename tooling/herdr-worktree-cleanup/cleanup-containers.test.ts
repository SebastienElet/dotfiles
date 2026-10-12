import { afterEach, expect, test } from "bun:test";
import { fixtureContainer, memoryDocker } from "./cleanup-test-support.ts";
import { mkdtempSync, rmSync } from "node:fs";
import type { DockerCommand } from "./cleanup-containers.ts";
import { cleanupWorktreeContainers } from "./cleanup-containers.ts";
import { tmpdir } from "node:os";

const roots: string[] = [];
afterEach(() => {
  for (const root of roots.splice(0)) {
    rmSync(root, { recursive: true, force: true });
  }
});
function checkout(): string {
  const root = mkdtempSync(`${tmpdir()}/docker-cleanup-`);
  roots.push(root);
  return root;
}

test.each(["stop", "remove"])(
  "%s is limited to owned IDs and replay is harmless",
  (mode) => {
    const path = checkout();
    const owned = fixtureContainer("a", path);
    const other = fixtureContainer("b", tmpdir());
    const docker = memoryDocker([owned, other]);
    expect(
      cleanupWorktreeContainers(path, mode, { runDocker: docker.runDocker })
        .kind,
    ).toBe("completed");
    expect(docker.containers.get(owned.Id)?.State.Running ?? false).toBe(false);
    expect(docker.containers.has(owned.Id)).toBe(mode === "stop");
    expect(docker.containers.get(other.Id)?.State.Running).toBe(true);
    expect(
      cleanupWorktreeContainers(path, mode, { runDocker: docker.runDocker })
        .kind,
    ).toBe("completed");
  },
);

test.each(["off", "invalid"])("%s never reaches Docker", (mode) => {
  const outcome = cleanupWorktreeContainers(checkout(), mode, {
    runDocker: () => {
      throw new Error("Docker must not be reached");
    },
  });
  expect(outcome.kind).toBe(mode === "off" ? "skipped" : "failed");
  expect(outcome.message).not.toContain("Docker must not be reached");
});

test.each(["tcp://localhost:2375", "ssh://server", "unix://relative"])(
  "skips remote or ambiguous endpoint %s",
  (host) => {
    const path = checkout();
    const container = fixtureContainer("a", path);
    const docker = memoryDocker([container], { host });
    expect(
      cleanupWorktreeContainers(path, "remove", { runDocker: docker.runDocker })
        .kind,
    ).toBe("skipped");
    expect(docker.containers.get(container.Id)?.State.Running).toBe(true);
  },
);

test.each(["context", "info", "ls", "inspect", "stop", "rm"])(
  "handles failure of %s without widening cleanup",
  (failure) => {
    const path = checkout();
    const container = fixtureContainer("a", path);
    const docker = memoryDocker([container], { failure });
    expect(
      cleanupWorktreeContainers(path, "remove", { runDocker: docker.runDocker })
        .kind,
    ).toBe(failure === "info" ? "skipped" : "failed");
    expect(docker.containers.has(container.Id)).toBe(true);
    expect(docker.containers.get(container.Id)?.State.Running).toBe(
      failure !== "rm",
    );
  },
);

test.each(["invalid-json", "wrong-id", "timeout"])(
  "refuses %s before mutation",
  (failure) => {
    const path = checkout();
    const container = fixtureContainer("a", path);
    const docker = memoryDocker([container]);
    const command: DockerCommand = (arguments_) => {
      if (arguments_.includes("inspect") && !arguments_.includes("context")) {
        if (failure === "timeout") {
          throw new Error("Docker command timed out");
        }
        return {
          exitCode: 0,
          stderr: "",
          stdout:
            failure === "invalid-json"
              ? "{}"
              : JSON.stringify([fixtureContainer("b", path)]),
        };
      }
      return docker.runDocker(arguments_);
    };
    expect(
      cleanupWorktreeContainers(path, "remove", { runDocker: command }).kind,
    ).toBe("failed");
    expect(docker.containers.get(container.Id)?.State.Running).toBe(true);
  },
);

test("retains partial failure and can replay after the error is resolved", () => {
  const path = checkout();
  const first = fixtureContainer("a", path);
  const second = fixtureContainer("b", path);
  const docker = memoryDocker([first, second]);
  const failing: DockerCommand = (arguments_) =>
    arguments_.includes("stop") && arguments_.includes(second.Id)
      ? { exitCode: 1, stdout: "", stderr: "fixture failure" }
      : docker.runDocker(arguments_);
  expect(
    cleanupWorktreeContainers(path, "stop", { runDocker: failing }).kind,
  ).toBe("failed");
  expect(docker.containers.get(first.Id)?.State.Running).toBe(false);
  expect(docker.containers.get(second.Id)?.State.Running).toBe(true);
  expect(
    cleanupWorktreeContainers(path, "stop", { runDocker: docker.runDocker })
      .kind,
  ).toBe("completed");
  expect(docker.containers.get(second.Id)?.State.Running).toBe(false);
});

test.each(["stop", "rm"])(
  "tolerates a container disappearing during %s",
  (disappear) => {
    const path = checkout();
    const docker = memoryDocker([fixtureContainer("a", path)], { disappear });
    expect(
      cleanupWorktreeContainers(path, "remove", { runDocker: docker.runDocker })
        .kind,
    ).toBe("completed");
    expect(docker.containers.size).toBe(0);
  },
);
