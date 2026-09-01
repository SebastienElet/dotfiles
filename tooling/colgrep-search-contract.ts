import { isAbsolute, relative } from "node:path";
import { lstatSync, readFileSync, realpathSync } from "node:fs";
import { z } from "zod";

const projectMetadataSchema = z
  .object({
    model: z.string().min(1),
    project_name: z.string().min(1),
    project_path: z.string().min(1),
  })
  .strict();

const fileStateSchema = z
  .object({
    content_hash: z.number().nonnegative(),
    mtime: z.number().nonnegative(),
    size: z.number().nonnegative(),
  })
  .strict();

const indexStateSchema = z
  .object({
    cli_version: z.string().min(1),
    dirty: z.literal(false),
    files: z.record(z.string(), fileStateSchema),
    ignored_files: z.array(z.string()),
    index_format_version: z.number().int().positive(),
    search_count: z.number().int().nonnegative(),
  })
  .strict();

const searchResultSchema = z.looseObject({
  unit: z.looseObject({ file: z.string().min(1) }),
});
const searchResultsSchema = z.array(searchResultSchema);

interface ColgrepStatus {
  readonly indexDirectory: string;
  readonly projectRoot: string;
}

function parseColgrepStatus(stdout: string): ColgrepStatus {
  const lines = stdout.split("\n");
  const projects = prefixedValues(lines, "Project: ");
  const indexes = prefixedValues(lines, "Index:   ");
  if (
    projects.length !== 1 ||
    indexes.length !== 1 ||
    lines.some((line) => line.startsWith("  Subdirectory: "))
  ) {
    throw new Error("ColGrep status is ambiguous");
  }
  const [projectRoot] = projects;
  const [indexDirectory] = indexes;
  if (projectRoot === undefined || indexDirectory === undefined) {
    throw new Error("ColGrep status is ambiguous");
  }
  return {
    indexDirectory,
    projectRoot,
  };
}

function validateColgrepIndex(
  expectedRoot: string,
  status: ColgrepStatus,
): string {
  const statusRoot = canonicalDirectory(status.projectRoot, "ColGrep project");
  if (statusRoot !== expectedRoot) {
    throw new Error("ColGrep status belongs to another root");
  }
  const indexDirectory = canonicalDirectory(
    status.indexDirectory,
    "ColGrep index",
  );
  const metadata = projectMetadataSchema.parse(
    parseJsonFile(`${indexDirectory}/project.json`),
  );
  const metadataRoot = canonicalDirectory(
    metadata.project_path,
    "ColGrep metadata project",
  );
  if (metadataRoot !== expectedRoot) {
    throw new Error("ColGrep metadata belongs to another root");
  }
  const state = indexStateSchema.parse(
    parseJsonFile(`${indexDirectory}/state.json`),
  );
  if (Object.keys(state.files).length === 0) {
    throw new Error("ColGrep index state is empty");
  }
  return indexDirectory;
}

function parseAndConfineResults(stdout: string, root: string): unknown[] {
  const results = searchResultsSchema.parse(JSON.parse(stdout));
  for (const result of results) {
    const { file } = result.unit;
    if (!isAbsolute(file)) {
      throw new Error("ColGrep returned a relative result path");
    }
    const canonicalFile = realpathSync.native(file);
    const relativeFile = relative(root, canonicalFile);
    if (
      relativeFile === "" ||
      relativeFile === ".." ||
      relativeFile.startsWith(
        `..${process.platform === "win32" ? "\\" : "/"}`,
      ) ||
      isAbsolute(relativeFile)
    ) {
      throw new Error("ColGrep returned a result outside the checkout");
    }
  }
  return results;
}

function canonicalDirectory(path: string, label: string): string {
  const metadata = lstatSync(path);
  if (metadata.isSymbolicLink() || !metadata.isDirectory()) {
    throw new Error(`${label} is not a real directory`);
  }
  return realpathSync.native(path);
}

function parseJsonFile(path: string): unknown {
  const bytes = readFileSync(path);
  const content = new TextDecoder("utf-8", { fatal: true }).decode(bytes);
  return JSON.parse(content);
}

function prefixedValues(lines: readonly string[], prefix: string): string[] {
  return lines
    .filter((line) => line.startsWith(prefix))
    .map((line) => line.slice(prefix.length))
    .filter((value) => value !== "");
}

export {
  parseAndConfineResults,
  parseColgrepStatus,
  validateColgrepIndex,
  type ColgrepStatus,
};
