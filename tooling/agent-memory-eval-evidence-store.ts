import { isAbsolute, join, relative, resolve } from "node:path";

import {
  readJson,
  records,
  storeArtifacts,
} from "./agent-memory-eval-evidence-store-artifacts.ts";

const CACHE_ENTRY_MAXIMUM_AGE_MILLISECONDS = Number("172800000");
const DIRECTORY_MODE = 0o700;
const FILE_MODE = 0o600;
const SCHEMA_VERSION_INDEX = 2;

async function validateStoreArtifacts(store: string): Promise<boolean> {
  const artifacts = await storeArtifacts(store);
  if (
    artifacts.some(
      (artifact) =>
        artifact.mode !== (artifact.directory ? DIRECTORY_MODE : FILE_MODE),
    )
  ) {
    return false;
  }
  const index = await readJson(join(store, "index.json"));
  const cache = await readJson(join(store, "oracle-cache.json"));
  if (
    index?.schema_version !== SCHEMA_VERSION_INDEX ||
    cache?.schema_version !== 1
  ) {
    return false;
  }
  const rows = records(index.entries);
  const cached = records(cache.entries);
  if (rows === undefined || cached === undefined) {
    return false;
  }
  const indexedIds = new Set<string>();
  const yamlPaths = new Set(
    artifacts
      .filter(
        (artifact) => !artifact.directory && artifact.path.endsWith(".yaml"),
      )
      .map((artifact) => artifact.path),
  );
  for (const row of rows) {
    if (typeof row.id !== "string" || typeof row.path !== "string") {
      return false;
    }
    if (!confinedExistingYaml(store, row.path, yamlPaths)) {
      return false;
    }
    indexedIds.add(row.id);
  }
  return cached.every((entry) => freshSpecificCacheEntry(entry, indexedIds));
}

function confinedExistingYaml(
  store: string,
  path: string,
  yamlPaths: Readonly<Set<string>>,
): boolean {
  const relativePath = relative(resolve(store), resolve(store, path));
  if (
    relativePath.startsWith("..") ||
    isAbsolute(relativePath) ||
    !path.endsWith(".yaml")
  ) {
    return false;
  }
  return yamlPaths.has(path);
}

function freshSpecificCacheEntry(
  entry: Readonly<Record<string, unknown>>,
  indexedIds: Readonly<Set<string>>,
): boolean {
  if (
    typeof entry.entry_id !== "string" ||
    !indexedIds.has(entry.entry_id) ||
    entry.verdict !== "valid" ||
    typeof entry.validated_at !== "string"
  ) {
    return false;
  }
  const age = Date.now() - Date.parse(entry.validated_at);
  return (
    Number.isFinite(age) &&
    age >= 0 &&
    age < CACHE_ENTRY_MAXIMUM_AGE_MILLISECONDS
  );
}

export { validateStoreArtifacts };
