import { existsSync, mkdirSync, symlinkSync, writeFileSync } from "node:fs";
import { expect, test } from "bun:test";
import { fixtureContainer, memoryDocker } from "./cleanup-test-support.ts";
import {
  git,
  repositoryWithLinkedWorktree,
  workspaceClosedEvent,
} from "./worktree-test-support.ts";
import type { DockerResult } from "./cleanup-containers.ts";
import { join } from "node:path";
import { removeClosedWorktree } from "./remove-closed-worktree.ts";

test("a misleading event cannot stop the main repository's containers", () => {
  const actual = repositoryWithLinkedWorktree();
  const other = repositoryWithLinkedWorktree();
  const container = fixtureContainer("a", actual.repositoryRoot);
  const docker = memoryDocker([container]);
  const outcome = removeClosedWorktree(
    workspaceClosedEvent({
      checkoutPath: actual.repositoryRoot,
      repositoryRoot: other.repositoryRoot,
      isLinkedWorktree: true,
    }),
    { dockerMode: "stop", runDocker: docker.runDocker },
  );
  expect(outcome.kind).toBe("skipped");
  expect(docker.containers.get(container.Id)?.State.Running).toBe(true);
});

test("closing a worktree never stops a separately registered nested worktree", () => {
  const { checkoutPath, repositoryRoot } = repositoryWithLinkedWorktree();
  const nestedPath = join(checkoutPath, "nested");
  git(repositoryRoot, "worktree", "add", "-qb", "nested", nestedPath);
  const owned = fixtureContainer("a", checkoutPath);
  const nested = fixtureContainer("b", nestedPath);
  const docker = memoryDocker([owned, nested]);
  removeClosedWorktree(
    workspaceClosedEvent({
      checkoutPath,
      repositoryRoot,
      isLinkedWorktree: true,
    }),
    { dockerMode: "stop", runDocker: docker.runDocker },
  );
  expect(docker.containers.get(owned.Id)?.State.Running).toBe(false);
  expect(docker.containers.get(nested.Id)?.State.Running).toBe(true);
});

test("stops attributed containers before removing the checkout", () => {
  const { checkoutPath, repositoryRoot } = repositoryWithLinkedWorktree();
  const containerId = fixtureContainer("a", checkoutPath).Id;
  let running = true;
  let checkoutExistedAtStop = false;
  const runDocker = (arguments_: readonly string[]): DockerResult => {
    let stdout = "";
    if (arguments_.includes("inspect") && arguments_.includes("context")) {
      stdout = JSON.stringify([
        { Endpoints: { docker: { Host: "unix:///fixture.sock" } } },
      ]);
    } else if (arguments_.includes("ls")) {
      stdout = `${containerId}\n`;
    } else if (arguments_.includes("inspect")) {
      stdout = JSON.stringify([
        {
          Id: containerId,
          Config: {
            Labels: {
              "com.docker.compose.project.working_dir": checkoutPath,
            },
          },
          Mounts: [],
          State: { Running: running },
        },
      ]);
    } else if (arguments_.includes("stop")) {
      running = false;
      checkoutExistedAtStop = existsSync(checkoutPath);
    }
    return { exitCode: 0, stdout, stderr: "" };
  };

  const outcome = removeClosedWorktree(
    workspaceClosedEvent({
      checkoutPath,
      isLinkedWorktree: true,
      repositoryRoot,
    }),
    { dockerMode: "stop", runDocker },
  );

  expect(running).toBe(false);
  expect(checkoutExistedAtStop).toBe(true);
  expect(outcome).toEqual({ kind: "removed", checkoutPath });
  expect(existsSync(checkoutPath)).toBe(false);
});

test.each(["absent", "stopped"])(
  "the plugin keeps Git cleanup working when Docker is %s",
  (availability) => {
    const { checkoutPath, repositoryRoot } = repositoryWithLinkedWorktree();
    const binaries = join(repositoryRoot, "binaries");
    if (availability === "absent") {
      mkdirSync(binaries);
      symlinkSync(Bun.which("git") ?? "/usr/bin/git", join(binaries, "git"));
    }
    const result = Bun.spawnSync(
      [
        process.execPath,
        "--config=/dev/null",
        "--no-env-file",
        join(import.meta.dir, "remove-closed-worktree.ts"),
      ],
      {
        env: {
          ...process.env,
          PATH: availability === "absent" ? binaries : process.env.PATH,
          DOCKER_HOST: `unix://${join(repositoryRoot, "missing.sock")}`,
          DOCKER_CONTEXT: "",
          HERDR_WORKTREE_DOCKER_CLEANUP: "stop",
          HERDR_PLUGIN_EVENT_JSON: workspaceClosedEvent({
            checkoutPath,
            repositoryRoot,
            isLinkedWorktree: true,
          }),
        },
        stdout: "pipe",
        stderr: "pipe",
      },
    );
    expect(result.exitCode).toBe(0);
    expect(result.stderr.toString()).toContain(
      availability === "absent"
        ? "Docker CLI unavailable"
        : "Docker daemon unavailable",
    );
    expect(existsSync(checkoutPath)).toBe(false);
  },
);

test.each(["dirty", "locked", "detached"])(
  "stops owned containers even if Git preserves a %s checkout",
  (state) => {
    const { checkoutPath, repositoryRoot } = repositoryWithLinkedWorktree();
    if (state === "dirty") {
      writeFileSync(join(checkoutPath, "notes.txt"), "keep");
    }
    if (state === "locked") {
      git(repositoryRoot, "worktree", "lock", checkoutPath);
    }
    if (state === "detached") {
      git(checkoutPath, "checkout", "-q", "--detach");
    }
    const container = fixtureContainer("a", checkoutPath);
    const docker = memoryDocker([container]);
    const outcome = removeClosedWorktree(
      workspaceClosedEvent({
        checkoutPath,
        repositoryRoot,
        isLinkedWorktree: true,
      }),
      { dockerMode: "stop", runDocker: docker.runDocker },
    );
    expect(outcome.kind).toBe(state === "detached" ? "skipped" : "failed");
    expect(docker.containers.get(container.Id)?.State.Running).toBe(false);
    expect(existsSync(checkoutPath)).toBe(true);
  },
);

test("preserves checkout on Docker failure, then safely replays the event", () => {
  const { checkoutPath, repositoryRoot } = repositoryWithLinkedWorktree();
  const container = fixtureContainer("a", checkoutPath);
  const event = workspaceClosedEvent({
    checkoutPath,
    repositoryRoot,
    isLinkedWorktree: true,
  });
  const failed = memoryDocker([container], { failure: "stop" });
  expect(
    removeClosedWorktree(event, {
      dockerMode: "stop",
      runDocker: failed.runDocker,
    }).kind,
  ).toBe("failed");
  expect(existsSync(checkoutPath)).toBe(true);
  const docker = memoryDocker([container]);
  expect(
    removeClosedWorktree(event, {
      dockerMode: "stop",
      runDocker: docker.runDocker,
    }).kind,
  ).toBe("removed");
  expect(
    removeClosedWorktree(event, {
      dockerMode: "remove",
      runDocker: () => {
        throw new Error("must not reach Docker");
      },
    }).kind,
  ).toBe("skipped");
});
