import { lstat, readdir, realpath } from "node:fs/promises";
import { relative, resolve, sep } from "node:path";
import { findIgnoredPaths } from "./git-ignore-audit.ts";

type PhysicalEntry = Readonly<{
  kind: "directory" | "file" | "ignored" | "unsupported";
  path: string;
}>;

async function listPhysicalEntries(
  skillRoot: string,
  directory: string,
): Promise<readonly PhysicalEntry[]> {
  const entries = await readdir(directory, { withFileTypes: true });
  const paths = await Promise.all(
    entries.map(async (entry: Readonly<(typeof entries)[number]>) => {
      const absolutePath = resolve(directory, entry.name);
      const path = relative(skillRoot, absolutePath).split(sep).join("/");
      if (entry.isDirectory()) {
        const nestedPaths = await listPhysicalEntries(skillRoot, absolutePath);
        const directoryEntry: PhysicalEntry = {
          kind: "directory",
          path: `${path}/`,
        };
        return nestedPaths.toSpliced(0, 0, directoryEntry);
      }
      return [{ kind: entry.isFile() ? "file" : "unsupported", path } as const];
    }),
  );
  return paths
    .flat()
    .toSorted((left, right) => left.path.localeCompare(right.path));
}

async function listSkillEntries(
  requestedSkillRoot: string,
): Promise<readonly PhysicalEntry[]> {
  const requestedRootMetadata = await lstat(requestedSkillRoot).catch(() => {
    throw new Error("The skill root could not be resolved.");
  });
  if (!requestedRootMetadata.isDirectory()) {
    throw new Error("The skill root must be a regular directory.");
  }
  const skillRoot = await realpath(requestedSkillRoot).catch(() => {
    throw new Error("The skill root could not be resolved.");
  });
  const skillEntries = await listPhysicalEntries(skillRoot, skillRoot);
  if (
    !skillEntries.some(
      (entry) => entry.kind === "file" && entry.path === "SKILL.md",
    )
  ) {
    throw new Error("The skill root has no regular SKILL.md.");
  }
  const ignoredPaths = findIgnoredPaths(
    skillRoot,
    skillEntries.map((entry) => entry.path),
  );
  return skillEntries.map((entry) => {
    if (!ignoredPaths.has(entry.path)) {
      return entry;
    }
    return { kind: "ignored", path: entry.path };
  });
}

export { listSkillEntries };
export type { PhysicalEntry };
