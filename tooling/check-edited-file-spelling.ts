import {
  type CoverageLookup,
  readSpellingCoverage,
  spellingSelection,
} from "./check-edited-file-spelling-coverage.ts";
import {
  type EditedFile,
  exists,
  isConfinedRegularFile,
  locateInRepository,
} from "./format-edited-file-location.ts";
import type { Environment } from "./bounded-command.ts";
import { lintSpelling } from "./check-edited-file-spelling-cspell.ts";
import { singleLine } from "./format-edited-file.ts";

type CheckEditedFileSpellingOptions = Readonly<{
  environment: Environment;
  repositoryCommonDirectory: string;
  timeoutMilliseconds: number;
}>;

const reportPrefix = "check-edited-file-spelling:";
const sharedDictionary = "home/.config/cspell/user.txt";

function notChecked(file: EditedFile, reason: string): string {
  return `${reportPrefix} ${file.relativePath} was not checked: ${singleLine(reason)}.`;
}

async function checkFile(
  file: EditedFile,
  coverage: CoverageLookup,
  options: CheckEditedFileSpellingOptions,
): Promise<string | undefined> {
  if (coverage.kind === "failed") {
    return notChecked(file, coverage.reason);
  }
  const selection = spellingSelection(coverage.coverage, file.relativePath);
  if (selection === "uncovered") {
    return undefined;
  }
  if (!(await exists(file.absolutePath))) {
    return notChecked(file, "the file no longer exists");
  }
  if (!(await isConfinedRegularFile(file))) {
    return undefined;
  }
  const verdict = await lintSpelling(file.relativePath, selection, {
    cwd: file.root,
    environment: options.environment,
    timeoutMilliseconds: options.timeoutMilliseconds,
  });
  if (verdict.kind === "failed") {
    return notChecked(file, verdict.reason);
  }
  if (verdict.kind === "accepted") {
    return undefined;
  }
  return [
    `${reportPrefix} cspell-check would reject ${file.relativePath}; fix the spelling or add the term to ${sharedDictionary}:`,
    ...verdict.issues.map(
      (issue) => `- line ${issue.line}: ${singleLine(issue.message)}`,
    ),
  ].join("\n");
}

async function checkEditedPath(
  path: string,
  coverageOf: (root: string) => Promise<CoverageLookup>,
  options: CheckEditedFileSpellingOptions,
): Promise<string | undefined> {
  const file = await locateInRepository(
    path,
    options.repositoryCommonDirectory,
  );
  if (file === undefined) {
    return undefined;
  }
  return checkFile(file, await coverageOf(file.root), options);
}

async function checkEditedFileSpelling(
  paths: readonly string[],
  options: CheckEditedFileSpellingOptions,
): Promise<readonly string[]> {
  const coverageByRoot = new Map<string, Promise<CoverageLookup>>();
  const coverageOf = (root: string): Promise<CoverageLookup> => {
    const coverage =
      coverageByRoot.get(root) ??
      readSpellingCoverage({
        cwd: root,
        environment: options.environment,
        timeoutMilliseconds: options.timeoutMilliseconds,
      });
    coverageByRoot.set(root, coverage);
    return coverage;
  };
  const reports: string[] = [];
  for (const path of paths) {
    const report = await checkEditedPath(path, coverageOf, options).catch(
      (error: unknown) =>
        `${reportPrefix} ${path} was not checked: ${singleLine(error instanceof Error ? error.message : String(error))}.`,
    );
    if (report !== undefined) {
      reports.push(report);
    }
  }
  return reports;
}

export { checkEditedFileSpelling };
