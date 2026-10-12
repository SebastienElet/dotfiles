import {
  existsSync,
  readFileSync,
  readlinkSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { expect, test } from "bun:test";
import {
  git,
  repositoryWithLinkedWorktree,
  workspaceClosedEvent,
} from "./worktree-test-support.ts";
import { join } from "node:path";
import { removeClosedWorktree } from "./remove-closed-worktree.ts";

test.each(["normal", "no"])(
  "removes a clean checkout and keeps its branch with status.showUntrackedFiles=%s",
  (visibility) => {
    const { checkoutPath, repositoryRoot } = repositoryWithLinkedWorktree();
    git(repositoryRoot, "config", "status.showUntrackedFiles", visibility);

    const outcome = removeClosedWorktree(
      workspaceClosedEvent({
        checkoutPath,
        isLinkedWorktree: true,
        repositoryRoot,
      }),
    );

    expect(outcome).toEqual({ kind: "removed", checkoutPath });
    expect(existsSync(checkoutPath)).toBe(false);
    expect(git(repositoryRoot, "branch", "--list", "feature")).toContain(
      "feature",
    );
  },
);

test.each(["normal", "no"])(
  "fails and keeps untracked files with status.showUntrackedFiles=%s",
  (visibility) => {
    const { checkoutPath, repositoryRoot } = repositoryWithLinkedWorktree();
    git(repositoryRoot, "config", "status.showUntrackedFiles", visibility);
    writeFileSync(join(checkoutPath, "notes.txt"), "work in progress\n");

    const outcome = removeClosedWorktree(
      workspaceClosedEvent({
        checkoutPath,
        isLinkedWorktree: true,
        repositoryRoot,
      }),
    );

    expect(outcome.kind).toBe("failed");
    expect(outcome.kind === "failed" && outcome.reason).toBeTruthy();
    expect(readFileSync(join(checkoutPath, "notes.txt"), "utf8")).toBe(
      "work in progress\n",
    );
  },
);

test("fails and preserves a locked checkout used by a deployed link", () => {
  const { checkoutPath, repositoryRoot } = repositoryWithLinkedWorktree();
  const source = join(checkoutPath, "skill.md");
  const deployedLink = join(repositoryRoot, "deployed-skill.md");
  writeFileSync(source, "fixture skill\n");
  git(checkoutPath, "add", "skill.md");
  git(checkoutPath, "commit", "-q", "-m", "fixture skill");
  symlinkSync(source, deployedLink);
  git(
    repositoryRoot,
    "worktree",
    "lock",
    "--reason",
    "deployment source",
    checkoutPath,
  );

  const outcome = removeClosedWorktree(
    workspaceClosedEvent({
      checkoutPath,
      isLinkedWorktree: true,
      repositoryRoot,
    }),
  );

  expect(outcome.kind).toBe("failed");
  expect(outcome.kind === "failed" && outcome.reason).toBeTruthy();
  expect(readlinkSync(deployedLink)).toBe(source);
  expect(readFileSync(deployedLink, "utf8")).toBe("fixture skill\n");
  expect(git(repositoryRoot, "branch", "--list", "feature")).toContain(
    "feature",
  );
});

test("removes ignored files along with an otherwise clean checkout", () => {
  const { checkoutPath, repositoryRoot } = repositoryWithLinkedWorktree();
  git(repositoryRoot, "config", "status.showUntrackedFiles", "no");
  writeFileSync(
    join(repositoryRoot, ".git", "info", "exclude"),
    "ignored-notes.txt\n",
  );
  writeFileSync(
    join(checkoutPath, "ignored-notes.txt"),
    "disposable ignored notes\n",
  );

  const outcome = removeClosedWorktree(
    workspaceClosedEvent({
      checkoutPath,
      isLinkedWorktree: true,
      repositoryRoot,
    }),
  );

  expect(outcome).toEqual({ kind: "removed", checkoutPath });
  expect(existsSync(checkoutPath)).toBe(false);
});

test("the plugin entry point reports hidden untracked files with a failing exit", () => {
  const { checkoutPath, repositoryRoot } = repositoryWithLinkedWorktree();
  git(repositoryRoot, "config", "status.showUntrackedFiles", "no");
  const notes = join(checkoutPath, "notes.txt");
  writeFileSync(notes, "work in progress\n");
  const pluginPath = join(import.meta.dir, "remove-closed-worktree.ts");

  const result = Bun.spawnSync(
    [process.execPath, "--config=/dev/null", "--no-env-file", pluginPath],
    {
      env: {
        ...process.env,
        HERDR_PLUGIN_EVENT_JSON: workspaceClosedEvent({
          checkoutPath,
          isLinkedWorktree: true,
          repositoryRoot,
        }),
      },
      stdout: "pipe",
      stderr: "pipe",
    },
  );

  expect(result.exitCode).toBe(1);
  expect(result.stdout.toString()).toMatch(/^failed: .+/u);
  expect(readFileSync(notes, "utf8")).toBe("work in progress\n");
});

test("skips a workspace that is not a linked worktree", () => {
  const { checkoutPath, repositoryRoot } = repositoryWithLinkedWorktree();

  const outcome = removeClosedWorktree(
    workspaceClosedEvent({
      checkoutPath: repositoryRoot,
      isLinkedWorktree: false,
      repositoryRoot,
    }),
  );

  expect(outcome).toEqual({
    kind: "skipped",
    reason: "workspace is not a linked worktree",
  });
  expect(existsSync(join(repositoryRoot, ".git"))).toBe(true);
  expect(existsSync(checkoutPath)).toBe(true);
});

test("skips a checkout whose HEAD is detached", () => {
  const { checkoutPath, repositoryRoot } = repositoryWithLinkedWorktree();
  git(checkoutPath, "checkout", "-q", "--detach");

  const outcome = removeClosedWorktree(
    workspaceClosedEvent({
      checkoutPath,
      isLinkedWorktree: true,
      repositoryRoot,
    }),
  );

  expect(outcome).toEqual({
    kind: "skipped",
    reason: "checkout HEAD is detached and its commits would be lost",
  });
  expect(existsSync(checkoutPath)).toBe(true);
});

test("skips a checkout that is already gone", () => {
  const { checkoutPath, repositoryRoot } = repositoryWithLinkedWorktree();
  rmSync(checkoutPath, { force: true, recursive: true });

  const outcome = removeClosedWorktree(
    workspaceClosedEvent({
      checkoutPath,
      isLinkedWorktree: true,
      repositoryRoot,
    }),
  );

  expect(outcome).toEqual({
    kind: "skipped",
    reason: "checkout no longer exists",
  });
});

test("skips an event whose workspace has no worktree snapshot", () => {
  const outcome = removeClosedWorktree(
    JSON.stringify({
      event: "workspace_closed",
      data: { type: "workspace_closed", workspace_id: "w1" },
    }),
  );

  expect(outcome).toEqual({
    kind: "skipped",
    reason: "event carries no worktree snapshot",
  });
});

test("fails on an event that is not valid JSON", () => {
  const outcome = removeClosedWorktree("not json");

  expect(outcome.kind).toBe("failed");
});

test("fails on an event whose worktree snapshot is malformed", () => {
  const outcome = removeClosedWorktree(
    JSON.stringify({
      data: { workspace: { worktree: { checkout_path: 3 } } },
    }),
  );

  expect(outcome.kind).toBe("failed");
});
