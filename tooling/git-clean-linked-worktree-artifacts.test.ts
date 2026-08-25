import { afterEach, expect, test } from "bun:test";
import { dirname, join } from "node:path";
import { mkdtemp, rm } from "node:fs/promises";

const entrypoint = join(import.meta.dir, "git-clean-linked-worktree-artifacts");
const temporaryDirectories: string[] = [];

afterEach(async () => {
  await Promise.all(
    temporaryDirectories.splice(0).map((path) => rm(path, { recursive: true })),
  );
});

async function createRepository(): Promise<string> {
  const root = await mkdtemp("/tmp/linked-worktree-");
  temporaryDirectories.push(root);
  const repository = join(root, "repository");
  await Bun.$`git init --quiet --initial-branch=main ${repository}`;
  await Bun.$`git -C ${repository} config user.email test@example.com`;
  await Bun.$`git -C ${repository} config user.name Test`;
  await Bun.write(join(repository, ".gitignore"), "ignored\n");
  await Bun.write(join(repository, "file"), "base\n");
  await Bun.$`git -C ${repository} add .gitignore file`;
  await Bun.$`git -C ${repository} commit --quiet --message base`;
  return repository;
}

async function createWorktree(
  repository: string,
  path: string,
  branch: string,
): Promise<void> {
  await Bun.$`git -C ${repository} worktree add --quiet -b ${branch} ${path}`;
  await Bun.write(join(path, "ignored"), "artifact\n");
}

function run(
  path: string,
  invocationWorktree: string,
): ReturnType<typeof Bun.spawnSync> {
  return Bun.spawnSync([entrypoint, path, invocationWorktree], {
    stderr: "pipe",
    stdout: "pipe",
  });
}

test("removes ignored artifacts from a linked worktree under the repository", async () => {
  const repository = await createRepository();
  const worktree = join(repository, ".worktrees", "feature");
  await createWorktree(repository, worktree, "feature");

  const result = run(worktree, repository);

  expect(result.exitCode).toBe(0);
  expect(await Bun.file(join(worktree, "ignored")).exists()).toBe(false);
});

test("removes ignored artifacts from a linked worktree outside the repository", async () => {
  const repository = await createRepository();
  const worktree = join(dirname(repository), "external-feature");
  await createWorktree(repository, worktree, "feature");

  const result = run(worktree, repository);

  expect(result.exitCode).toBe(0);
  expect(await Bun.file(join(worktree, "ignored")).exists()).toBe(false);
});

test("refuses the primary worktree", async () => {
  const repository = await createRepository();
  const invocationWorktree = join(dirname(repository), "invocation-feature");
  await createWorktree(repository, invocationWorktree, "feature");
  await Bun.write(join(repository, "ignored"), "artifact\n");

  const result = run(repository, invocationWorktree);

  expect(result.exitCode).toBe(1);
  expect(result.stderr?.toString()).toContain("refusing to clean the primary");
  expect(await Bun.file(join(repository, "ignored")).exists()).toBe(true);
});

test("preserves ignored artifacts in a locked linked worktree", async () => {
  const repository = await createRepository();
  const worktree = join(dirname(repository), "locked-feature");
  await createWorktree(repository, worktree, "feature");
  await Bun.$`git -C ${repository} worktree lock ${worktree}`;

  const result = run(worktree, repository);

  expect(result.exitCode).toBe(1);
  expect(result.stderr?.toString()).toContain("locked");
  expect(await Bun.file(join(worktree, "ignored")).exists()).toBe(true);
});

test("preserves the linked worktree that invoked cleanup", async () => {
  const repository = await createRepository();
  const worktree = join(dirname(repository), "active-feature");
  await createWorktree(repository, worktree, "feature");

  const result = run(worktree, worktree);

  expect(result.exitCode).toBe(1);
  expect(result.stderr?.toString()).toContain("invoking worktree");
  expect(await Bun.file(join(worktree, "ignored")).exists()).toBe(true);
});
