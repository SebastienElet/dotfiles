import { isAbsolute, relative, sep } from "node:path";
import { realpathSync } from "node:fs";
import { z } from "zod";

const containerSchema = z
  .object({
    Id: z.string().regex(/^[0-9a-f]{64}$/u),
    Config: z
      .object({
        Labels: z.record(z.string(), z.string()).readonly().nullable(),
      })
      .readonly(),
    Mounts: z
      .array(
        z
          .object({
            Type: z.string(),
            Source: z.string().optional(),
            Name: z.string().optional(),
          })
          .readonly(),
      )
      .readonly(),
    State: z.object({ Running: z.boolean() }).readonly(),
  })
  .readonly();

type Container = z.infer<typeof containerSchema>;
type Selection = Readonly<{
  containers: readonly Container[];
  skipped: readonly string[];
}>;

function canonicalPath(path: string): string | undefined {
  if (!isAbsolute(path)) {
    return undefined;
  }
  try {
    return realpathSync(path);
  } catch {
    return undefined;
  }
}

function isInside(root: string, path: string): boolean {
  const canonicalRoot = canonicalPath(root);
  const canonical = canonicalPath(path);
  if (canonical === undefined || canonicalRoot === undefined) {
    return false;
  }
  const difference = relative(canonicalRoot, canonical);
  return (
    difference !== ".." &&
    !difference.startsWith(`..${sep}`) &&
    !isAbsolute(difference)
  );
}

function attributionFailure(
  container: Container,
  checkout: string,
  protectedCheckouts: readonly string[],
): string | undefined {
  const labels = container.Config.Labels ?? {};
  const composePath = labels["com.docker.compose.project.working_dir"];
  const explicitPath = labels["dotfiles.worktree.path"];
  if (hasUncertainMounts(container.Mounts)) {
    return "mount metadata is uncertain";
  }
  if (belongsToProtectedCheckout(container, protectedCheckouts)) {
    return "path belongs to another registered worktree";
  }
  if (
    labels["dotfiles.worktree.shared"] !== undefined &&
    labels["dotfiles.worktree.shared"] !== "false"
  ) {
    return "container is marked shared";
  }
  if (composePath !== undefined && !isInside(checkout, composePath)) {
    return "Compose working directory is outside the worktree or unresolved";
  }
  if (explicitPath !== undefined && canonicalPath(explicitPath) !== checkout) {
    return "explicit worktree attribution conflicts or is unresolved";
  }
  const binds = container.Mounts.filter((mount) => mount.Type === "bind");
  if (
    binds.some(
      (mount) =>
        mount.Source === undefined || !isInside(checkout, mount.Source),
    )
  ) {
    return "bind mount is outside the worktree or unresolved";
  }
  if (
    composePath === undefined &&
    (explicitPath === undefined || binds.length === 0)
  ) {
    return "no exclusive worktree attribution";
  }
  return undefined;
}

function hasUncertainMounts(mounts: Container["Mounts"]): boolean {
  return mounts.some(
    (mount) =>
      !["bind", "volume", "tmpfs"].includes(mount.Type) ||
      (mount.Type === "volume" &&
        (mount.Name === undefined || mount.Name === "")),
  );
}

function belongsToProtectedCheckout(
  container: Container,
  checkouts: readonly string[],
): boolean {
  const labels = container.Config.Labels ?? {};
  const paths = [
    labels["com.docker.compose.project.working_dir"],
    labels["dotfiles.worktree.path"],
    ...container.Mounts.filter((mount) => mount.Type === "bind").map(
      (mount) => mount.Source,
    ),
  ];
  return paths.some(
    (path) =>
      path !== undefined &&
      checkouts.some((checkout) => isInside(checkout, path)),
  );
}

function mountsOverlap(
  mount: Container["Mounts"][number],
  external: Container["Mounts"][number],
): boolean {
  if (mount.Type === "volume") {
    return external.Type === "volume" && mount.Name === external.Name;
  }
  if (
    mount.Type !== "bind" ||
    external.Type !== "bind" ||
    mount.Source === undefined ||
    external.Source === undefined
  ) {
    return false;
  }
  return (
    isInside(mount.Source, external.Source) ||
    isInside(external.Source, mount.Source)
  );
}

function selectWorktreeContainers(
  containers: readonly Container[],
  checkoutPath: string,
  protectedCheckouts: readonly string[] = [],
): Selection {
  const checkout = realpathSync(checkoutPath);
  const nestedCheckouts = protectedCheckouts.filter(
    (path) => canonicalPath(path) !== checkout && isInside(checkout, path),
  );
  const failures = new Map(
    containers.map((container) => [
      container.Id,
      attributionFailure(container, checkout, nestedCheckouts),
    ]),
  );
  const externalMounts = containers
    .filter((container) => failures.get(container.Id) !== undefined)
    .flatMap((container) => container.Mounts);
  const selected: Container[] = [];
  const skipped: string[] = [];
  for (const container of containers) {
    const failure =
      failures.get(container.Id) ??
      (container.Mounts.some((mount) =>
        externalMounts.some((external) => mountsOverlap(mount, external)),
      )
        ? "mount is shared with a container outside the worktree"
        : undefined);
    if (failure === undefined) {
      selected.push(container);
    } else {
      skipped.push(`${container.Id}: ${failure}`);
    }
  }
  return { containers: selected, skipped };
}

export { canonicalPath, containerSchema, selectWorktreeContainers };
export type { Container, Selection };
