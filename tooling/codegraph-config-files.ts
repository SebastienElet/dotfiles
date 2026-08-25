import {
  ConfigurationError,
  type JsonObject,
  parseJsonObject,
} from "./codegraph-config.ts";
import {
  type Stats,
  chmodSync,
  closeSync,
  lstatSync,
  mkdirSync,
  openSync,
  readFileSync,
  realpathSync,
  renameSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { basename, dirname, join } from "node:path";
import { randomUUID } from "node:crypto";

interface ConfigurationSnapshot {
  readonly path: string;
  readonly content?: readonly number[];
  readonly mode?: number;
}

const configurationFileMode = 0o600;
const jsonIndentSpaces = 2;

function withConfigurationLocks(
  paths: readonly string[],
  action: () => void,
): void {
  const lockPaths: string[] = [];
  try {
    for (const path of canonicalPaths(paths)) {
      const lockPath = `${path}.codegraph-configure.lock`;
      const descriptor = openConfigurationLock(lockPath);
      lockPaths.push(lockPath);
      closeSync(descriptor);
    }
    action();
  } finally {
    for (const lockPath of lockPaths.toReversed()) {
      rmSync(lockPath, { force: true });
    }
  }
}

function openConfigurationLock(lockPath: string): number {
  try {
    return openSync(lockPath, "wx", configurationFileMode);
  } catch (error) {
    if (isExistingFile(error)) {
      throw new ConfigurationError(
        `configuration update already in progress: ${lockPath}`,
      );
    }
    throw error;
  }
}

function inspectConfiguration(
  path: string,
  fileLabel: string,
  jsonLabel?: string,
): { snapshot: ConfigurationSnapshot; parsed?: JsonObject } {
  const status = configurationStatus(path);
  if (status === undefined) {
    return jsonLabel === undefined
      ? { snapshot: { path } }
      : { parsed: {}, snapshot: { path } };
  }
  if (status.isSymbolicLink()) {
    throw new ConfigurationError(`${fileLabel} must not be a symlink: ${path}`);
  }
  if (!status.isFile()) {
    throw new ConfigurationError(`${fileLabel} is not a regular file: ${path}`);
  }
  if (status.nlink > 1) {
    throw new ConfigurationError(
      `${fileLabel} must not have multiple hard links: ${path}`,
    );
  }
  const content = [...readFileSync(path)];
  const snapshot = { content, mode: status.mode, path };
  return jsonLabel === undefined
    ? { snapshot }
    : {
        parsed: parseJsonObject(
          Buffer.from(content).toString("utf8"),
          jsonLabel,
        ),
        snapshot,
      };
}

function configurationStatus(path: string): Stats | undefined {
  try {
    return lstatSync(path);
  } catch (error) {
    if (isMissingFile(error)) {
      return undefined;
    }
    throw error;
  }
}

function writeJsonAtomically(path: string, value: JsonObject): void {
  writeAtomically(
    path,
    [...Buffer.from(`${JSON.stringify(value, undefined, jsonIndentSpaces)}\n`)],
    configurationFileMode,
  );
}

function restoreConfigurations(
  snapshots: readonly ConfigurationSnapshot[],
): void {
  const failures: string[] = [];
  for (const snapshot of snapshots.toReversed()) {
    try {
      restoreConfiguration(snapshot);
    } catch (error) {
      failures.push(`${snapshot.path}: ${errorMessage(error)}`);
    }
  }
  if (failures.length > 0) {
    throw new Error(`configuration rollback failed:\n${failures.join("\n")}`);
  }
}

function restoreConfiguration(snapshot: ConfigurationSnapshot): void {
  if (snapshot.content === undefined) {
    rmSync(snapshot.path, { force: true });
    return;
  }
  writeAtomically(snapshot.path, snapshot.content, snapshot.mode);
}

function writeAtomically(
  path: string,
  content: readonly number[],
  mode?: number,
): void {
  mkdirSync(dirname(path), { recursive: true });
  const temporaryPath = `${path}.codegraph-configure.${randomUUID()}`;
  try {
    writeFileSync(temporaryPath, Uint8Array.from(content), {
      flag: "wx",
      mode,
    });
    if (mode !== undefined) {
      chmodSync(temporaryPath, mode);
    }
    renameSync(temporaryPath, path);
  } finally {
    rmSync(temporaryPath, { force: true });
  }
}

function isMissingFile(error: unknown): boolean {
  return error instanceof Error && "code" in error && error.code === "ENOENT";
}

function isExistingFile(error: unknown): boolean {
  return error instanceof Error && "code" in error && error.code === "EEXIST";
}

function canonicalPaths(paths: readonly string[]): string[] {
  return [
    ...new Set(
      paths.map((path) => {
        mkdirSync(dirname(path), { recursive: true });
        return join(realpathSync.native(dirname(path)), basename(path));
      }),
    ),
  ].toSorted();
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export {
  type ConfigurationSnapshot,
  inspectConfiguration,
  restoreConfigurations,
  withConfigurationLocks,
  writeJsonAtomically,
};
