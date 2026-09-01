import { join, relative } from "node:path";
import { lstat, readFile, readdir } from "node:fs/promises";

import { isRecord } from "./agent-memory-eval-evidence-types.ts";

const MODE_MASK = 0o777;

type StoreArtifact = Readonly<{
  directory: boolean;
  mode: number;
  path: string;
}>;

async function storeArtifacts(root: string): Promise<readonly StoreArtifact[]> {
  const artifacts: StoreArtifact[] = [];
  async function visit(path: string): Promise<void> {
    const metadata = await lstat(path);
    artifacts.push({
      directory: metadata.isDirectory(),
      mode: metadata.mode & MODE_MASK,
      path: relative(root, path),
    });
    if (!metadata.isDirectory()) {
      return;
    }
    const entries = await readdir(path);
    for (const entry of entries) {
      await visit(join(path, entry));
    }
  }
  await visit(root);
  return artifacts;
}

async function readJson(
  path: string,
): Promise<Readonly<Record<string, unknown>> | undefined> {
  try {
    const parsed: unknown = JSON.parse(await readFile(path, "utf8"));
    return isRecord(parsed) ? parsed : undefined;
  } catch {
    return undefined;
  }
}

function records(
  value: unknown,
): readonly Readonly<Record<string, unknown>>[] | undefined {
  return Array.isArray(value) && value.every((item) => isRecord(item))
    ? value
    : undefined;
}

export { readJson, records, storeArtifacts, type StoreArtifact };
