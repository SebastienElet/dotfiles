import { afterEach, expect, test } from "bun:test";
import { chmod, mkdir, mkdtemp, readlink, rm, symlink } from "node:fs/promises";
import { join } from "node:path";

const projectRoot = join(import.meta.dir, "..");
const temporaryDirectories: string[] = [];
const executableFileMode = 0o755;

afterEach(async () => {
  await Promise.all(
    temporaryDirectories.splice(0).map((path) => rm(path, { recursive: true })),
  );
});

test("installs the repository pre-push hook idempotently", async () => {
  const fixture = await mkdtemp("/tmp/git-hooks-deployment-");
  temporaryDirectories.push(fixture);
  const source = join(fixture, "tooling", "pre-push");
  const destinationDirectory = join(fixture, ".git", "hooks");
  await mkdir(destinationDirectory, { recursive: true });
  await mkdir(join(fixture, "tooling"));
  await Bun.write(source, Bun.file(join(projectRoot, "tooling", "pre-push")));
  await chmod(source, executableFileMode);
  await symlink(
    join(projectRoot, "tooling", "pre-push.ts"),
    join(fixture, "tooling", "pre-push.ts"),
  );

  for (const attempt of ["install", "replay"] as const) {
    const result = Bun.spawnSync(
      [
        "make",
        "-f",
        join(projectRoot, "Makefile"),
        `DOTFILES_PATH=${fixture}`,
        "git-hooks",
      ],
      { cwd: projectRoot, stderr: "pipe", stdout: "pipe" },
    );
    expect(result.exitCode, attempt).toBe(0);
  }

  expect(await readlink(join(destinationDirectory, "pre-push"))).toBe(source);
  const process = Bun.spawn(
    [join(destinationDirectory, "pre-push"), "origin", "url"],
    {
      cwd: fixture,
      stderr: "pipe",
      stdin: "pipe",
      stdout: "pipe",
    },
  );
  await process.stdin.write("invalid\n");
  await process.stdin.end();
  expect(await process.exited).toBe(1);
});
