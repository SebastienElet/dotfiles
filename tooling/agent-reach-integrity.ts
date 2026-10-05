import { lstat, readFile, readdir } from "node:fs/promises";
import { createHash } from "node:crypto";
import { join } from "node:path";
import { z } from "zod";

const treeEntrySchema = z.tuple([
  z.enum(["100644", "100755"]),
  z.literal("blob"),
  z.string().regex(/^[a-f0-9]{40}$/u),
]);
type GitTree = Readonly<{ entries: readonly (readonly [string, string])[] }>;

async function sourceFiles(root: string, path = ""): Promise<string[]> {
  const files: string[] = [];
  const entries = await readdir(join(root, path), { withFileTypes: true });
  for (const entry of entries) {
    if (path !== "" || entry.name !== ".git") {
      const child = join(path, entry.name);
      if (entry.isDirectory()) {
        files.push(...(await sourceFiles(root, child)));
      } else if (entry.isFile()) {
        files.push(child);
      } else {
        throw new Error(`divergent source resource: ${child}`);
      }
    }
  }
  return files.toSorted();
}

function gitTree(tree: string): GitTree {
  const expected = new Map<string, string>();
  for (const line of tree.split("\0").filter(Boolean)) {
    const separator = line.indexOf("\t");
    if (separator === -1) {
      throw new Error("Invalid Git tree entry");
    }
    const entry = treeEntrySchema.parse(line.slice(0, separator).split(" "));
    const path = z
      .string()
      .min(1)
      .parse(line.slice(separator + 1));
    if (path.startsWith("/") || path.split("/").includes("..")) {
      throw new Error("Invalid Git tree path");
    }
    expected.set(path, entry[2]);
  }
  return { entries: [...expected] };
}

async function verifyGitSourceBytes(root: string, tree: string): Promise<void> {
  const expected = gitTree(tree).entries;
  const actual = await sourceFiles(root);
  if (
    expected.length === 0 ||
    JSON.stringify(expected.map(([path]) => path).toSorted()) !==
      JSON.stringify(actual)
  ) {
    throw new Error("divergent source file inventory");
  }
  for (const [path, hash] of expected) {
    const file = join(root, path);
    const entry = await lstat(file);
    if (!entry.isFile() || entry.isSymbolicLink() || entry.nlink !== 1) {
      throw new Error(`divergent source resource: ${path}`);
    }
    const bytes = await readFile(file);
    const observed = createHash("sha1")
      .update(`blob ${bytes.length}\0`)
      .update(bytes)
      .digest("hex");
    if (observed !== hash) {
      throw new Error(`divergent source bytes: ${path}`);
    }
  }
}

export { verifyGitSourceBytes };
