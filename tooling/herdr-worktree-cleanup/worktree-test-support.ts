import { afterEach, expect } from "bun:test";
import { mkdtempSync, rmSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";

const temporaryRoots: string[] = [];

afterEach(() => {
  for (const root of temporaryRoots.splice(0)) {
    rmSync(root, { force: true, recursive: true });
  }
});

function git(directory: string, ...arguments_: readonly string[]): string {
  const result = Bun.spawnSync(
    [
      "git",
      "-c",
      "user.email=test@example.com",
      "-c",
      "user.name=Test",
      "-C",
      directory,
      ...arguments_,
    ],
    { stderr: "pipe", stdout: "pipe" },
  );
  expect(result.exitCode).toBe(0);
  return result.stdout.toString().trim();
}

function repositoryWithLinkedWorktree(): Readonly<{
  checkoutPath: string;
  repositoryRoot: string;
}> {
  const root = mkdtempSync(join(tmpdir(), "herdr-worktree-cleanup-"));
  temporaryRoots.push(root);
  const repositoryRoot = join(root, "repository");
  const checkoutPath = join(root, "checkout");
  expect(
    Bun.spawnSync(["git", "init", "-q", "-b", "main", repositoryRoot]).exitCode,
  ).toBe(0);
  git(repositoryRoot, "commit", "-q", "--allow-empty", "-m", "initial");
  git(repositoryRoot, "worktree", "add", "-q", "-b", "feature", checkoutPath);
  return { checkoutPath, repositoryRoot };
}

function workspaceClosedEvent(
  worktree: Readonly<{
    checkoutPath: string;
    isLinkedWorktree: boolean;
    repositoryRoot: string;
  }>,
): string {
  return JSON.stringify({
    event: "workspace_closed",
    data: {
      type: "workspace_closed",
      workspace_id: "w1",
      workspace: {
        workspace_id: "w1",
        label: "feature",
        worktree: {
          checkout_path: worktree.checkoutPath,
          is_linked_worktree: worktree.isLinkedWorktree,
          repo_key: `${worktree.repositoryRoot}/.git`,
          repo_name: "repository",
          repo_root: worktree.repositoryRoot,
        },
      },
    },
  });
}

export { git, repositoryWithLinkedWorktree, workspaceClosedEvent };
