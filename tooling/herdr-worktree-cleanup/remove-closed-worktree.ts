import { existsSync } from "node:fs";
import { z } from "zod";

const workspaceClosedEventSchema = z.object({
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

type RemovalOutcome =
  | Readonly<{ kind: "removed"; checkoutPath: string }>
  | Readonly<{ kind: "skipped"; reason: string }>
  | Readonly<{ kind: "failed"; reason: string }>;

function runGit(
  directory: string,
  arguments_: readonly string[],
): Readonly<{ exitCode: number; stderr: string }> {
  const result = Bun.spawnSync(["git", "-C", directory, ...arguments_], {
    stderr: "pipe",
    stdout: "ignore",
  });
  return { exitCode: result.exitCode, stderr: result.stderr.toString().trim() };
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

function removeClosedWorktree(eventJson: string): RemovalOutcome {
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
  if (
    runGit(checkoutPath, ["symbolic-ref", "--quiet", "HEAD"]).exitCode !== 0
  ) {
    return skipped("checkout HEAD is detached and its commits would be lost");
  }
  const removal = runGit(worktree.repo_root, [
    "worktree",
    "remove",
    checkoutPath,
  ]);
  return removal.exitCode === 0
    ? { kind: "removed", checkoutPath }
    : { kind: "failed", reason: removal.stderr };
}

if (import.meta.main) {
  const outcome = removeClosedWorktree(
    process.env.HERDR_PLUGIN_EVENT_JSON ?? "",
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
