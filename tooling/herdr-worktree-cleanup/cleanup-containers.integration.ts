import { existsSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { expect, test } from "bun:test";
import type { Container } from "./container-selection.ts";
import { containerSchema } from "./container-selection.ts";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { z } from "zod";

const integrationTimeoutMilliseconds = 60_000;
const commandTimeoutMilliseconds = 20_000;
const idSchema = z.string().regex(/^[0-9a-f]{64}$/u);
type Fixture = Readonly<{
  root: string;
  repository: string;
  checkoutA: string;
  checkoutB: string;
  projectA: string;
  projectB: string;
}>;

function command(
  arguments_: readonly string[],
  env: Readonly<NodeJS.ProcessEnv> = process.env,
): string {
  const result = Bun.spawnSync([...arguments_], {
    env,
    stdout: "pipe",
    stderr: "pipe",
    timeout: commandTimeoutMilliseconds,
  });
  if (result.exitCode !== 0) {
    throw new Error(result.stderr.toString());
  }
  return result.stdout.toString().trim();
}

function git(root: string, ...arguments_: readonly string[]): string {
  return command([
    "git",
    "-c",
    "user.name=Test",
    "-c",
    "user.email=test@example.com",
    "-C",
    root,
    ...arguments_,
  ]);
}

function startCompose(checkout: string, project: string): string {
  writeFileSync(
    join(checkout, "compose.yml"),
    "services:\n  app:\n    image: ubuntu:24.04\n    command: [sleep, infinity]\n    volumes:\n      - data:/data\n      - /anonymous\nvolumes:\n  data:\n",
  );
  git(checkout, "add", "compose.yml");
  git(checkout, "commit", "-qm", "fixture Compose");
  command([
    "docker",
    "compose",
    "--project-directory",
    checkout,
    "-p",
    project,
    "up",
    "-d",
  ]);
  return idSchema.parse(
    command([
      "docker",
      "compose",
      "--project-directory",
      checkout,
      "-p",
      project,
      "ps",
      "--all",
      "--quiet",
    ]),
  );
}

function inspect(id: string): Container {
  return z
    .tuple([containerSchema])
    .parse(JSON.parse(command(["docker", "container", "inspect", id])))[0];
}

function containerVolumes(id: string): readonly string[] {
  return inspect(id)
    .Mounts.filter((mount) => mount.Type === "volume")
    .map((mount) => z.string().min(1).parse(mount.Name));
}

function fixture(): Fixture {
  const root = mkdtempSync(join(tmpdir(), "herdr-docker-integration-"));
  const prefix = `herdr-492-${crypto.randomUUID()}`;
  return {
    root,
    repository: join(root, "repository"),
    checkoutA: join(root, "work tree A"),
    checkoutB: join(root, "work tree B"),
    projectA: `${prefix}-a`,
    projectB: `${prefix}-b`,
  };
}

function startFixtures(
  testFixture: Fixture,
  record: (id: string) => void,
): Readonly<{ owned: string; other: string; witness: string }> {
  const { repository, checkoutA, checkoutB, projectA, projectB } = testFixture;
  command(["git", "init", "-q", "-b", "main", repository]);
  git(repository, "commit", "-q", "--allow-empty", "-m", "initial");
  git(repository, "worktree", "add", "-qb", "feature-a", checkoutA);
  git(repository, "worktree", "add", "-qb", "feature-b", checkoutB);
  const owned = startCompose(checkoutA, projectA);
  record(owned);
  const other = startCompose(checkoutB, projectB);
  record(other);
  const witness = idSchema.parse(
    command([
      "docker",
      "run",
      "-d",
      "--name",
      `${projectA}-witness`,
      "ubuntu:24.04",
      "sleep",
      "infinity",
    ]),
  );
  record(witness);
  return { owned, other, witness };
}

function closeWorktree(testFixture: Fixture, mode: string): string {
  const event = JSON.stringify({
    event: "workspace_closed",
    data: {
      type: "workspace_closed",
      workspace: {
        worktree: {
          checkout_path: testFixture.checkoutA,
          is_linked_worktree: true,
          repo_root: testFixture.repository,
        },
      },
    },
  });
  return command(
    [
      process.execPath,
      "--config=/dev/null",
      "--no-env-file",
      join(import.meta.dir, "remove-closed-worktree.ts"),
    ],
    {
      ...process.env,
      HERDR_WORKTREE_DOCKER_CLEANUP: mode,
      HERDR_PLUGIN_EVENT_JSON: event,
    },
  );
}

function expectResourcesPreserved(
  volumes: readonly string[],
  project: string,
): void {
  for (const volume of volumes) {
    expect(
      command(["docker", "volume", "inspect", "--format", "{{.Name}}", volume]),
    ).toBe(volume);
  }
  expect(
    command([
      "docker",
      "network",
      "inspect",
      "--format",
      "{{.Name}}",
      `${project}_default`,
    ]),
  ).toBe(`${project}_default`);
}

function removeFixtures(
  testFixture: Fixture,
  resources: Readonly<{
    containers: readonly string[];
    volumes: readonly string[];
  }>,
): void {
  for (const id of resources.containers) {
    Bun.spawnSync(["docker", "container", "rm", "--force", id]);
  }
  for (const volume of resources.volumes) {
    Bun.spawnSync(["docker", "volume", "rm", volume]);
  }
  for (const project of [testFixture.projectA, testFixture.projectB]) {
    Bun.spawnSync(["docker", "network", "rm", `${project}_default`]);
  }
  rmSync(testFixture.root, { recursive: true, force: true });
}

test.each(["stop", "remove"])(
  "real Docker %s preserves worktree B, a witness, volumes and networks",
  (mode) => {
    const testFixture = fixture();
    const containers: string[] = [];
    const fixtureVolumes: string[] = [];
    try {
      const { owned, other, witness } = startFixtures(testFixture, (id) => {
        containers.push(id);
        fixtureVolumes.push(...containerVolumes(id));
      });
      const volumes = containerVolumes(owned);
      expect(closeWorktree(testFixture, mode)).toContain(
        `removed ${testFixture.checkoutA}`,
      );
      expect(existsSync(testFixture.checkoutA)).toBe(false);
      expect(existsSync(testFixture.checkoutB)).toBe(true);
      expect(inspect(other).State.Running).toBe(true);
      expect(inspect(witness).State.Running).toBe(true);
      if (mode === "stop") {
        expect(inspect(owned).State.Running).toBe(false);
      } else {
        expect(
          command([
            "docker",
            "container",
            "ls",
            "--all",
            "--quiet",
            "--no-trunc",
          ]).split("\n"),
        ).not.toContain(owned);
      }
      expectResourcesPreserved(volumes, testFixture.projectA);
    } finally {
      removeFixtures(testFixture, { containers, volumes: fixtureVolumes });
    }
  },
  integrationTimeoutMilliseconds,
);
