import type { UpgradeRunner } from "./upgrade-runner.ts";
import { join } from "node:path";
import { stat } from "node:fs/promises";
import { z } from "zod";

const branchSchema = z.string().trim().regex(/^\S+$/u);
const lockfile = "home/.config/nvim/lazy-lock.json";

async function onMainBranch(
  runner: UpgradeRunner,
  repository: string,
  action: string,
): Promise<boolean> {
  const options = { capture: true, required: true };
  const current = await runner.command(
    "Dotfiles current branch",
    ["git", "-C", repository, "branch", "--show-current"],
    options,
  );
  if (current.state !== "succeeded") {
    runner.skip(action, "unable to determine the dotfiles branch");
    return false;
  }
  if (current.value.trim() === "") {
    runner.skip(action, "detached HEAD");
    return false;
  }
  const main = await runner.command(
    "Dotfiles main branch",
    [join(repository, "tooling/git-main-branch"), "--strict", "-C", repository],
    options,
  );
  if (main.state !== "succeeded") {
    runner.skip(action, "unable to determine the main dotfiles branch");
    return false;
  }
  const branches = await runner.attempt("Dotfiles branch validation", () => ({
    current: branchSchema.parse(current.value),
    main: branchSchema.parse(main.value),
  }));
  if (branches.state !== "succeeded") {
    runner.skip(action, "invalid branch evidence");
    return false;
  }
  if (branches.value.current !== branches.value.main) {
    runner.skip(action, `on branch ${branches.value.current}`);
    return false;
  }
  return true;
}

async function upgradeRepository(
  runner: UpgradeRunner,
  repository: string,
  moon: string,
): Promise<void> {
  const action = "Dotfiles repository update";
  const repositoryState = await runner.attempt(
    "Dotfiles repository inspection",
    () => repositoryExists(repository),
  );
  if (repositoryState.state !== "succeeded" || !repositoryState.value) {
    runner.skip(action, "~/.dotfiles not a git repo");
    runner.skip("Dotfiles redeployment", "repository unavailable");
    return;
  }
  if (!(await onMainBranch(runner, repository, action))) {
    runner.skip("Dotfiles redeployment", "repository update not attempted");
    return;
  }
  const pull = await runner.command(action, ["git", "-C", repository, "pull"], {
    required: true,
  });
  if (pull.state !== "succeeded") {
    runner.skip("Dotfiles redeployment", "repository update failed");
    return;
  }
  await runner.command(
    "Dotfiles redeployment",
    [moon, "exec", "--quiet", "repository:install"],
    { cwd: repository, required: true },
  );
}

async function repositoryExists(repository: string): Promise<boolean> {
  try {
    const metadata = await stat(join(repository, ".git"));
    return metadata.isDirectory();
  } catch (error) {
    if (error instanceof Error && "code" in error && error.code === "ENOENT") {
      return false;
    }
    throw error;
  }
}

async function commitNeovimLockfile(
  runner: UpgradeRunner,
  repository: string,
): Promise<void> {
  const action = "NeoVim lockfile commit";
  if (!(await onMainBranch(runner, repository, action))) {
    return;
  }
  const status = await runner.command(
    "NeoVim lockfile inspection",
    ["git", "-C", repository, "status", "--porcelain", "--", lockfile],
    { capture: true, required: true },
  );
  if (status.state !== "succeeded") {
    runner.skip(action, "unable to inspect the NeoVim lockfile");
    return;
  }
  if (status.value.trim() === "") {
    runner.skip(action, "lockfile unchanged");
    return;
  }
  await runner.command(
    action,
    [
      "git",
      "-C",
      repository,
      "commit",
      "-m",
      "chore(nvim): update plugins",
      "--",
      lockfile,
    ],
    { required: true },
  );
}

export { commitNeovimLockfile, upgradeRepository };
