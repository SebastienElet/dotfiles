import { type Dirent, lstatSync, readdirSync, realpathSync } from "node:fs";
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

function gitOpenTofuFiles(
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

function nonGitOpenTofuFiles(repository: string): string[] {
  return filesBelow(repository, "");
}

function filesBelow(repository: string, directory: string): string[] {
  const entries = readDirectory(repository, directory);
  return entries.flatMap((entry: Readonly<Dirent>) => {
    const path = directory === "" ? entry.name : `${directory}/${entry.name}`;
    if (entry.isDirectory()) {
      return excludedDirectories.has(entry.name)
        ? []
        : filesBelow(repository, path);
    }
    if (entry.isFile() && /\.tofu$/iu.test(entry.name)) {
      return [realpathSync.native(resolve(repository, path))];
    }
    return [];
  });
}

function readDirectory(repository: string, directory: string): Dirent[] {
  try {
    return readdirSync(resolve(repository, directory), {
      withFileTypes: true,
    });
  } catch (error) {
    throw new MeasurementError(
      `repository traversal failed: ${message(error)}`,
    );
  }
}

function eligibleFile(repository: string, file: string): string[] {
  if (!/\.tofu$/iu.test(file) || hasExcludedSegment(file)) {
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

export { gitOpenTofuFiles, nonGitOpenTofuFiles };
