import {
  chmodSync,
  closeSync,
  lstatSync,
  mkdirSync,
  openSync,
  readFileSync,
  renameSync,
  realpathSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { randomUUID } from "node:crypto";
import { basename, dirname, join } from "node:path";
import {
  ConfigurationError,
  parseJsonObject,
  type JsonObject,
} from "./codegraph-config.ts";

export type ConfigurationSnapshot = {
  path: string;
  content?: Buffer;
  mode?: number;
};

export function withConfigurationLocks(
  paths: string[],
  action: () => void,
): void {
  const lockPaths: string[] = [];
  try {
    for (const path of canonicalPaths(paths)) {
      const lockPath = `${path}.codegraph-configure.lock`;
      let descriptor: number;
      try {
        descriptor = openSync(lockPath, "wx", 0o600);
      } catch (error) {
        if (isExistingFile(error)) {
          throw new ConfigurationError(
            `configuration update already in progress: ${lockPath}`,
          );
        }
        throw error;
      }
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

export function inspectConfiguration(
  path: string,
  fileLabel: string,
  jsonLabel?: string,
): { snapshot: ConfigurationSnapshot; parsed?: JsonObject } {
  let status: ReturnType<typeof lstatSync>;
  try {
    status = lstatSync(path);
  } catch (error) {
    if (isMissingFile(error)) {
      return jsonLabel === undefined
        ? { snapshot: { path } }
        : { snapshot: { path }, parsed: {} };
    }
    throw error;
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
  const content = readFileSync(path);
  const snapshot = { path, content, mode: status.mode };
  return jsonLabel === undefined
    ? { snapshot }
    : {
        snapshot,
        parsed: parseJsonObject(content.toString("utf8"), jsonLabel),
      };
}

export function writeJsonAtomically(path: string, value: JsonObject): void {
  writeAtomically(
    path,
    Buffer.from(`${JSON.stringify(value, null, 2)}\n`),
    0o600,
  );
}

export function restoreConfigurations(
  snapshots: ConfigurationSnapshot[],
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

function writeAtomically(path: string, content: Buffer, mode?: number): void {
  mkdirSync(dirname(path), { recursive: true });
  const temporaryPath = `${path}.codegraph-configure.${randomUUID()}`;
  try {
    writeFileSync(temporaryPath, content, { flag: "wx", mode });
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

function canonicalPaths(paths: string[]): string[] {
  return [
    ...new Set(
      paths.map((path) => {
        mkdirSync(dirname(path), { recursive: true });
        return join(realpathSync.native(dirname(path)), basename(path));
      }),
    ),
  ].sort();
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
