import { lstatSync, readdirSync, realpathSync, type Dirent } from "node:fs";
import { relative, resolve, sep } from "node:path";
import { MeasurementError } from "./codegraph-repository-measurement.ts";

const excludedDirectories = new Set([
  ".git",
  ".codegraph",
  ".worktrees",
  "node_modules",
  "vendor",
  "dist",
  "build",
  "out",
  "target",
  "coverage",
  "generated",
  "docs",
  "fixtures",
]);

export function gitOpenTofuFiles(
  repository: string,
  nulSeparatedFiles: string,
): string[] {
  if (nulSeparatedFiles !== "" && !nulSeparatedFiles.endsWith("\0")) {
    throw new MeasurementError("Git paths are not NUL-terminated");
  }
  const files = nulSeparatedFiles === "" ? [] : nulSeparatedFiles.split("\0");
  if (files.at(-1) === "") {
    files.pop();
  }
  if (files.includes("")) {
    throw new MeasurementError("invalid empty path from Git");
  }
  return files.flatMap((file) => eligibleFile(repository, file));
}

export function nonGitOpenTofuFiles(repository: string): string[] {
  const files: string[] = [];
  visit(repository, "", files);
  return files;
}

function visit(repository: string, directory: string, files: string[]): void {
  let entries: Dirent[];
  try {
    entries = readdirSync(resolve(repository, directory), {
      withFileTypes: true,
    });
  } catch (error) {
    throw new MeasurementError(
      `repository traversal failed: ${message(error)}`,
    );
  }
  for (const entry of entries) {
    const path = directory === "" ? entry.name : `${directory}/${entry.name}`;
    if (entry.isDirectory()) {
      if (!excludedDirectories.has(entry.name)) {
        visit(repository, path, files);
      }
      continue;
    }
    if (entry.isFile() && /\.tofu$/i.test(entry.name)) {
      files.push(realpathSync.native(resolve(repository, path)));
    }
  }
}

function eligibleFile(repository: string, file: string): string[] {
  if (!/\.tofu$/i.test(file) || hasExcludedSegment(file)) {
    return [];
  }
  const path = resolve(repository, file);
  const confined = relative(repository, path);
  if (confined === ".." || confined.startsWith(`..${sep}`)) {
    throw new MeasurementError(
      `Git returned a path outside repository: ${file}`,
    );
  }
  try {
    const status = lstatSync(path);
    return status.isFile() && !status.isSymbolicLink() ? [path] : [];
  } catch (error) {
    if (error instanceof Error && "code" in error && error.code === "ENOENT") {
      return [];
    }
    throw new MeasurementError(
      `cannot inspect Git path ${file}: ${message(error)}`,
    );
  }
}

function hasExcludedSegment(path: string): boolean {
  return path.split("/").some((segment) => excludedDirectories.has(segment));
}

function message(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
