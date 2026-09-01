import { chmod, lstat, readFile, readdir, readlink } from "node:fs/promises";
import { join, relative, resolve } from "node:path";
import type { Dirent } from "node:fs";
import { createHash } from "node:crypto";

const fullPermissionMode = 0o777;

type DigestChildRequest = Readonly<{
  child: string;
  entry: Readonly<Dirent>;
}>;

async function treeDigest(root: string): Promise<string> {
  const hash = createHash("sha256");
  async function digestDirectory(path: string): Promise<void> {
    const entries = await directoryEntries(path);
    for (const entry of entries.toSorted(
      (left: Readonly<Dirent>, right: Readonly<Dirent>) =>
        left.name.localeCompare(right.name),
    )) {
      const child = join(path, entry.name);
      const metadata = await lstat(child);
      hash.update(relative(root, child));
      hash.update(
        `:${metadata.mode & fullPermissionMode}:${entryType(entry)}:`,
      );
      await digestChild({ child, entry });
    }
  }
  async function digestChild(request: DigestChildRequest): Promise<void> {
    if (request.entry.isDirectory()) {
      await digestDirectory(request.child);
    } else if (request.entry.isFile()) {
      hash.update(await readFile(request.child));
    } else if (request.entry.isSymbolicLink()) {
      hash.update(await readlink(request.child));
    }
  }
  await digestDirectory(root);
  return hash.digest("hex");
}

async function directoryEntries(path: string): Promise<readonly Dirent[]> {
  try {
    return await readdir(path, { withFileTypes: true });
  } catch (error) {
    if (error instanceof Error && "code" in error && error.code === "ENOENT") {
      return [];
    }
    throw error;
  }
}

function entryType(entry: Readonly<Dirent>): string {
  if (entry.isDirectory()) {
    return "d";
  }
  return entry.isFile() ? "f" : "l";
}

async function runtimeSha(): Promise<string> {
  const runtime = resolve(
    import.meta.dir,
    "agent-memory/target/release/agent-memory",
  );
  const bytes = await readFile(runtime);
  return createHash("sha256").update(bytes).digest("hex");
}

async function makeSourceUnavailable(source: string): Promise<void> {
  await chmod(source, 0o000);
}

export { makeSourceUnavailable, runtimeSha, treeDigest };
