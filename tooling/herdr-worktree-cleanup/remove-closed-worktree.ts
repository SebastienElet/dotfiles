import type { DockerCommand } from "./cleanup-containers.ts";
import { canonicalPath } from "./container-selection.ts";
import { cleanupWorktreeContainers } from "./cleanup-containers.ts";
import { existsSync } from "node:fs";
import { z } from "zod";

const workspaceClosedEventSchema = z.object({
  event: z.literal("workspace_closed"),
  data: z.object({
    workspace: z
      .object({
        worktree: z
          .object({
            checkout_path: z.string().min(1),
            is_linked_worktree: z.boolean(),
            repo_root: z.string().min(1),
          })
          .optional(),
      })
      .optional(),
  }),
});

type WorkspaceClosedEvent = z.infer<typeof workspaceClosedEventSchema>;
type CleanupOptions = Readonly<{
  dockerMode?: string;
  runDocker?: DockerCommand;
  reportDocker?: (message: string) => void;
}>;

const worktreePathsSchema = z
  .string()
  .transform((output) =>
    output
      .split("\0")
      .filter((field) => field.startsWith("worktree "))
      .map((field) => field.slice("worktree ".length)),
  )
  .pipe(z.array(z.string().min(1)).min(1));

type RemovalOutcome =
  | Readonly<{ kind: "removed"; checkoutPath: string }>
  | Readonly<{ kind: "skipped"; reason: string }>
  | Readonly<{ kind: "failed"; reason: string }>;

function runGit(
  directory: string,
  arguments_: readonly string[],
): Readonly<{ exitCode: number; stderr: string; stdout: string }> {
  const result = Bun.spawnSync(["git", "-C", directory, ...arguments_], {
    stderr: "pipe",
    stdout: "pipe",
  });
  return {
    exitCode: result.exitCode,
    stderr: result.stderr.toString().trim(),
    stdout: result.stdout.toString(),
  };
}

function registeredWorktrees(repositoryRoot: string): readonly string[] {
  const result = runGit(repositoryRoot, [
    "worktree",
    "list",
    "--porcelain",
    "-z",
  ]);
  return result.exitCode === 0 ? worktreePathsSchema.parse(result.stdout) : [];
}

function removeCheckout(
  checkoutPath: string,
  repositoryRoot: string,
): RemovalOutcome {
  if (
    runGit(checkoutPath, ["symbolic-ref", "--quiet", "HEAD"]).exitCode !== 0
  ) {
    return skipped("checkout HEAD is detached and its commits would be lost");
  }
  const removal = runGit(repositoryRoot, [
    "-c",
    "status.showUntrackedFiles=all",
    "worktree",
    "remove",
    checkoutPath,
  ]);
  return removal.exitCode === 0
    ? { kind: "removed", checkoutPath }
    : { kind: "failed", reason: removal.stderr };
}

function skipped(reason: string): RemovalOutcome {
  return { kind: "skipped", reason };
}

function parseWorkspaceClosedEvent(
  eventJson: string,
):
  | Readonly<{ ok: true; event: WorkspaceClosedEvent }>
  | Readonly<{ ok: false; reason: string }> {
  try {
    const parsed = workspaceClosedEventSchema.safeParse(JSON.parse(eventJson));
    return parsed.success
      ? { ok: true, event: parsed.data }
      : { ok: false, reason: parsed.error.message };
  } catch (error) {
    return {
      ok: false,
      reason: error instanceof Error ? error.message : String(error),
    };
  }
}

function removeClosedWorktree(
  eventJson: string,
  options: CleanupOptions = {},
): RemovalOutcome {
  const parsed = parseWorkspaceClosedEvent(eventJson);
  if (!parsed.ok) {
    return { kind: "failed", reason: parsed.reason };
  }
  const worktree = parsed.event.data.workspace?.worktree;
  if (worktree === undefined) {
    return skipped("event carries no worktree snapshot");
  }
  if (!worktree.is_linked_worktree) {
    return skipped("workspace is not a linked worktree");
  }
  const checkoutPath = worktree.checkout_path;
  if (!existsSync(checkoutPath)) {
    return skipped("checkout no longer exists");
  }
  const checkouts = registeredWorktrees(worktree.repo_root);
  const canonicalCheckout = canonicalPath(checkoutPath);
  if (
    canonicalCheckout === undefined ||
    !checkouts
      .slice(1)
      .some((path) => canonicalPath(path) === canonicalCheckout)
  ) {
    return skipped(
      "checkout is not a registered linked worktree of the repository",
    );
  }
  const docker = cleanupWorktreeContainers(
    checkoutPath,
    options.dockerMode ?? "off",
    { ...options, protectedCheckouts: checkouts },
  );
  options.reportDocker?.(`Docker ${docker.kind}: ${docker.message}`);
  if (docker.kind === "failed") {
    return {
      kind: "failed",
      reason: `Docker cleanup failed; checkout preserved: ${docker.message}`,
    };
  }
  return removeCheckout(checkoutPath, worktree.repo_root);
}

if (import.meta.main) {
  const outcome = removeClosedWorktree(
    process.env.HERDR_PLUGIN_EVENT_JSON ?? "",
    {
      dockerMode: process.env.HERDR_WORKTREE_DOCKER_CLEANUP ?? "off",
      reportDocker: (message) => {
        process.stderr.write(`${message}\n`);
      },
    },
  );
  const message =
    outcome.kind === "removed"
      ? `removed ${outcome.checkoutPath}`
      : `${outcome.kind}: ${outcome.reason}`;
  process.stdout.write(`${message}\n`);
  process.exitCode = outcome.kind === "failed" ? 1 : 0;
}

export { removeClosedWorktree };
export type { RemovalOutcome };
