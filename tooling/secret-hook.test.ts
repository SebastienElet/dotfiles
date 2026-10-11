import {
  commit,
  git,
  gitEnvironment,
  repository,
  sourceRoot,
  token,
} from "./secret-scan-test-support.ts";
import { copyFile, readFile, stat, symlink, writeFile } from "node:fs/promises";
import { expect, test } from "bun:test";
import { join } from "node:path";

function prepare(root: string): Bun.SyncSubprocess<"pipe", "pipe"> {
  return Bun.spawnSync(
    [
      process.execPath,
      "--no-env-file",
      join(sourceRoot, "tooling/secret-hook.ts"),
    ],
    { cwd: root, env: gitEnvironment },
  );
}

test("preparation is idempotent and real commits refuse fictitious tokens", async () => {
  const root = await repository();
  expect(prepare(root).exitCode).toBe(0);
  const hook = join(root, ".git/hooks/pre-commit");
  const before = await stat(hook);
  expect(prepare(root).exitCode).toBe(0);
  const after = await stat(hook);
  expect(after.mtimeMs).toBe(before.mtimeMs);
  await writeFile(join(root, "file.txt"), "safe\n");
  git(root, "add", "file.txt");
  expect(commit(root).exitCode).toBe(0);
  await writeFile(join(root, "file.txt"), `TOKEN=${token}\n`);
  git(root, "add", "file.txt");
  const refusal = commit(root);
  expect(refusal.exitCode).not.toBe(0);
  expect(refusal.stderr.toString()).not.toContain(token);
});

test("preserves a third-party hook, symlink and custom hooksPath", async () => {
  const root = await repository();
  const hook = join(root, ".git/hooks/pre-commit");
  await writeFile(hook, "third-party");
  expect(prepare(root).exitCode).not.toBe(0);
  expect(await readFile(hook, "utf8")).toBe("third-party");
  git(root, "config", "core.hooksPath", "custom-hooks");
  expect(prepare(root).exitCode).not.toBe(0);
  expect(git(root, "config", "--get", "core.hooksPath")).toBe("custom-hooks");
  const second = await repository();
  await symlink(hook, join(second, ".git/hooks/pre-commit"));
  expect(prepare(second).exitCode).not.toBe(0);
  expect(await readFile(hook, "utf8")).toBe("third-party");
});

test("a shared hook resolves the active worktree instead of the installer checkout", async () => {
  const root = await repository();
  const worktree = join(root, "worktree");
  git(root, "worktree", "add", "-qb", "fixture-worktree", worktree);
  await symlink(
    join(sourceRoot, "node_modules"),
    join(worktree, "node_modules"),
    "dir",
  );
  expect(prepare(worktree).exitCode).toBe(0);
  expect(prepare(root).exitCode).toBe(0);
  await writeFile(join(worktree, "file.txt"), `TOKEN=${token}\n`);
  git(worktree, "add", "file.txt");
  expect(commit(worktree).exitCode).not.toBe(0);
  await writeFile(join(root, "safe.txt"), "safe\n");
  git(root, "add", "safe.txt");
  expect(commit(root).exitCode).toBe(0);
});

test("a clone can prepare its own hook", async () => {
  const root = await repository();
  const clone = join(root, "clone");
  git(root, "clone", "-q", root, clone);
  await copyFile(join(root, ".gitleaks.toml"), join(clone, ".gitleaks.toml"));
  expect(prepare(clone).exitCode).toBe(0);
  const hook = await stat(join(clone, ".git/hooks/pre-commit"));
  expect(hook.isFile()).toBe(true);
});
