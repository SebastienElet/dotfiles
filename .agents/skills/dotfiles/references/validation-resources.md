# Validation resources

## Before execution

Inspect the selected check's real commands, cleanup executable and version, including worktree-close
handlers. A temporary HOME or checkout does not isolate a container engine, image tag or database.
Distinguish required operational validation from speculative preparation: the former may create
private fixtures within the task's mandate; the latter grants no access to real or shared data.

Classify every affected resource before running the check:

| Resource                           | Permitted use                                                                                                                                                         |
| ---------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Private, created by this task      | Use and clean only the recorded task-owned paths or IDs. Keep retained data outside the cleanup root.                                                                 |
| Shared, with an identifiable owner | Use only as its existing reservation contract permits. Validation must not rebuild, replace, migrate, stop or delete shared containers, images, databases or volumes. |
| Forbidden or ownership unknown     | Do not use or clean it. Record the missing prerequisite.                                                                                                              |

Record the owner, exact resource identity and permitted effects. A Compose project name alone does
not isolate image tags, external volumes, host ports or a shared database. Use private identities
for all mutable dependencies, or leave that proof unavailable. Never use engine-wide prune or
cleanup by a broad name pattern.

For an occupied resource, establish its owner, release condition, maximum wait and explicitly
authorized fallback before waiting. Observe availability within that bound while continuing
independent work. A timeout never grants concurrent access. On expiry, use only the authorized
private fallback; otherwise report the unavailable check and its prerequisite. The investigation
budget is not a resource reservation or permission to keep waiting indefinitely.

## Before cleanup

Inventory tracked, untracked and ignored data, plus any paths explicitly marked for retention.
Confirm which data are disposable and which must survive; unknown retention blocks deletion.
Resolve the actual cleanup root and any links before establishing ownership. A recorded temporary
root is not permission to follow links into another task's data.

For a Git worktree holding retained data, lock it before validation or workspace closure with
`git worktree lock --reason 'retained validation data' /absolute/worktree/path`, and verify the lock
with `git worktree list --porcelain`. Keep retained data outside a disposable checkout whenever
possible. Git and the Herdr cleanup handler can delete ignored files in an otherwise clean
checkout; a backup branch does not preserve untracked or ignored files. The lock protects normal
Git worktree removal, not arbitrary scripts or forced deletion.

Before permitting deletion of the only local copy, copy retained data outside every cleanup root
to an owned durable destination, compare file contents and metadata needed for restoration, and
demonstrate restoration to a separate disposable destination. A partial backup is insufficient.
Keep the lock until retained data and any deployed links have verified durable destinations.
Unlock only a lock created by this task, then invoke only the inspected cleanup on owned disposable
resources. A refusal or partial failure means inspect what remains; never retry with force or
broaden the deletion scope.

## When changing a cleanup procedure

Exercise the actual executable on synthetic tracked, untracked and ignored files, retained data
and a locked worktree. Compare data before and after, exercise a cleanup failure, restore retained
data from the verified external copy, and confirm an owned disposable checkout remains removable.
If the reported command or version is unknown, label a reproduced mechanism as a compatible
hypothesis, not the historical cause. Fix a repository-owned defect at its cleanup boundary;
external defects belong to their owning source, not a local imitation.

Exercise occupied-then-released and timeout cases with two private jobs. Verify the neighbor's
resource identities and data remain unchanged, only the permitted fallback runs, and cleanup
removes only the caller's resources. Name substitutions and any unexercised live container or
database boundary in the evidence; a file-based exercise cannot establish their isolation.
