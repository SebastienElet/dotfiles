import { dirname, join, relative, resolve, sep } from "node:path";
import {
  lstat,
  mkdir,
  mkdtemp,
  readFile,
  readdir,
  realpath,
  rm,
  symlink,
  writeFile,
} from "node:fs/promises";
import type { Stats } from "node:fs";
import { z } from "zod";

async function metadata(path: string): Promise<Stats | null> {
  try {
    return await lstat(path);
  } catch (error) {
    if (z.object({ code: z.literal("ENOENT") }).safeParse(error).success) {
      return null;
    }
    throw error;
  }
}

async function regularText(root: string, path: string): Promise<string> {
  const absolute = resolve(root, path);
  const entry = await lstat(absolute);
  if (
    !entry.isFile() ||
    entry.isSymbolicLink() ||
    entry.nlink !== 1 ||
    (await realpath(absolute)) !== absolute
  ) {
    throw new Error(`Not a confined regular resource: ${path}`);
  }
  return new TextDecoder("utf-8", { fatal: true }).decode(
    await readFile(absolute),
  );
}

const directoryMode = 0o700;

async function ensureDirectories(
  home: string,
  destination: string,
): Promise<void> {
  let current = home;
  for (const part of relative(home, destination).split(sep)) {
    current = join(current, part);
    await mkdir(current, { mode: directoryMode }).catch((error: unknown) => {
      if (!z.object({ code: z.literal("EEXIST") }).safeParse(error).success) {
        throw error;
      }
    });
    const entry = await lstat(current);
    if (!entry.isDirectory() || entry.isSymbolicLink()) {
      throw new Error(`Divergent directory: ${current}`);
    }
  }
}

async function installedFiles(root: string, path = ""): Promise<string[]> {
  const files: string[] = [];
  for (const entry of await readdir(join(root, path), {
    withFileTypes: true,
  })) {
    const child = join(path, entry.name);
    if (entry.isSymbolicLink()) {
      throw new Error(`divergent resource: ${child}`);
    }
    if (entry.isDirectory()) {
      files.push(...(await installedFiles(root, child)));
    } else if (entry.isFile()) {
      files.push(child);
    } else {
      throw new Error(`divergent resource: ${child}`);
    }
  }
  return files.toSorted();
}

async function preserveExisting(
  destination: string,
  files: readonly (readonly [string, string])[],
): Promise<void> {
  const root = await realpath(destination);
  if (
    JSON.stringify(await installedFiles(root)) !==
    JSON.stringify(files.map(([path]) => path).toSorted())
  ) {
    throw new Error(`divergent marketplace: ${destination}`);
  }
  for (const [path, content] of files) {
    if ((await regularText(root, path)) !== content) {
      throw new Error(`divergent marketplace resource: ${path}`);
    }
  }
}

async function publishMarketplace(
  directory: string,
  destination: string,
  files: readonly (readonly [string, string])[],
): Promise<void> {
  const staging = await mkdtemp(join(directory, ".marketplace-"));
  let published = false;
  try {
    for (const [path, content] of files) {
      await mkdir(dirname(join(staging, path)), { recursive: true });
      await writeFile(join(staging, path), content);
    }
    await symlink(staging, destination);
    published = true;
  } finally {
    if (!published) {
      await rm(staging, { recursive: true, force: true });
    }
  }
}

export {
  ensureDirectories,
  metadata,
  preserveExisting,
  publishMarketplace,
  regularText,
};
