import {
  type EditedFile,
  exists,
  ignoreStatus,
  isConfinedRegularFile,
  locateInRepository,
} from "./format-edited-file-location.ts";
import {
  type Environment,
  type Formatter,
  type Transformation,
  formatterFor,
} from "./format-edited-file-formatters.ts";
import { readFile, writeFile } from "node:fs/promises";

type FormatEditedFilesOptions = Readonly<{
  environment: Environment;
  repositoryCommonDirectory: string;
  timeoutMilliseconds: number;
}>;

type Deadline = Readonly<{
  cancel: () => void;
  expired: Promise<Transformation>;
}>;

function singleLine(text: string): string {
  return text.replaceAll(/\s+/gu, " ").trim().replace(/\.$/u, "");
}

function deadline(formatter: Formatter, milliseconds: number): Deadline {
  let timer: ReturnType<typeof setTimeout> | undefined = undefined;
  const expired = new Promise<Transformation>((resolve) => {
    timer = setTimeout(() => {
      resolve({
        kind: "failed",
        reason: `${formatter.name} timed out after ${milliseconds} ms`,
      });
    }, milliseconds);
  });
  return {
    cancel: () => {
      clearTimeout(timer);
    },
    expired,
  };
}

async function transformWithinDeadline(
  file: EditedFile,
  formatter: Formatter,
  options: FormatEditedFilesOptions,
): Promise<Readonly<{ source: string; transformation: Transformation }>> {
  const source = await readFile(file.absolutePath, "utf8");
  const limit = deadline(formatter, options.timeoutMilliseconds);
  try {
    const transformation = await Promise.race([
      formatter
        .transform(source, {
          environment: options.environment,
          file,
          timeoutMilliseconds: options.timeoutMilliseconds,
        })
        .catch((error: unknown): Transformation => ({
          kind: "failed",
          reason: `${formatter.name} failed: ${error instanceof Error ? error.message : String(error)}`,
        })),
      limit.expired,
    ]);
    return { source, transformation };
  } finally {
    limit.cancel();
  }
}

async function reformat(
  file: EditedFile,
  formatter: Formatter,
  options: FormatEditedFilesOptions,
): Promise<string | undefined> {
  const prefix = `format-edited-file: ${file.relativePath}`;
  const { source, transformation } = await transformWithinDeadline(
    file,
    formatter,
    options,
  );
  if (transformation.kind === "failed") {
    return `${prefix} was not formatted: ${singleLine(transformation.reason)}.`;
  }
  if (transformation.kind === "ignored" || transformation.code === source) {
    return undefined;
  }
  await writeFile(file.absolutePath, transformation.code);
  return `${prefix} was reformatted with ${formatter.name}; read it again before editing it.`;
}

async function formatEditedPath(
  path: string,
  options: FormatEditedFilesOptions,
): Promise<string | undefined> {
  const file = await locateInRepository(
    path,
    options.repositoryCommonDirectory,
  );
  const formatter = file && formatterFor(file.relativePath);
  if (file === undefined || formatter === undefined) {
    return undefined;
  }
  const prefix = `format-edited-file: ${file.relativePath}`;
  if (!(await exists(file.absolutePath))) {
    return `${prefix} was not formatted: the file no longer exists.`;
  }
  if (!(await isConfinedRegularFile(file))) {
    return undefined;
  }
  const status = await ignoreStatus(file);
  if (status === "unknown") {
    return `${prefix} was not formatted: git check-ignore failed.`;
  }
  return status === "ignored" ? undefined : reformat(file, formatter, options);
}

async function formatEditedFiles(
  paths: readonly string[],
  options: FormatEditedFilesOptions,
): Promise<readonly string[]> {
  const reports: string[] = [];
  for (const path of paths) {
    const report = await formatEditedPath(path, options).catch(
      (error: unknown) =>
        `format-edited-file: ${path} was not formatted: ${singleLine(error instanceof Error ? error.message : String(error))}.`,
    );
    if (report !== undefined) {
      reports.push(report);
    }
  }
  return reports;
}

export { formatEditedFiles, singleLine };
export { repositoryCommonDirectoryOf } from "./format-edited-file-location.ts";
export type { FormatEditedFilesOptions };
