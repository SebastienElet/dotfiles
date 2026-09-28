import {
  type BoundedCommandOptions,
  commandFailure,
  runBounded,
} from "./bounded-command.ts";
import type { SpellingSelection } from "./check-edited-file-spelling-coverage.ts";
import { join } from "node:path";

type SpellingIssue = Readonly<{ line: number; message: string }>;

type SpellingVerdict =
  | Readonly<{ kind: "accepted" }>
  | Readonly<{ issues: readonly SpellingIssue[]; kind: "rejected" }>
  | Readonly<{ kind: "failed"; reason: string }>;

const cspellIssuesFound = 1;
const issueLine = /^(?<path>.+):(?<line>\d+):(?<column>\d+) - (?<message>.+)$/u;

function issuesOf(stdout: string): readonly SpellingIssue[] {
  return stdout.split("\n").flatMap((line): readonly SpellingIssue[] => {
    const groups = issueLine.exec(line.trimEnd())?.groups;
    return groups?.line === undefined || groups.message === undefined
      ? []
      : [{ line: Number(groups.line), message: groups.message }];
  });
}

async function lintSpelling(
  relativePath: string,
  selection: Exclude<SpellingSelection, "uncovered">,
  options: BoundedCommandOptions,
): Promise<SpellingVerdict> {
  const home = options.environment.HOME;
  if (home === undefined || home === "") {
    return { kind: "failed", reason: "HOME is not set" };
  }
  const outcome = await runBounded(
    "bun",
    [
      "x",
      "--bun",
      "--no-install",
      "cspell",
      "lint",
      "--config",
      join(home, "cspell.json"),
      "--no-progress",
      "--no-summary",
      "--no-color",
      ...(selection === "force-checked-file" ? ["--force-check"] : []),
      "--file",
      relativePath,
    ],
    { ...options, displayName: "cspell" },
  );
  if (outcome.kind === "failed") {
    return outcome;
  }
  if (outcome.status === 0) {
    return { kind: "accepted" };
  }
  const issues = issuesOf(outcome.stdout);
  return outcome.status === cspellIssuesFound && issues.length > 0
    ? { issues, kind: "rejected" }
    : {
        kind: "failed",
        reason: commandFailure("cspell", outcome.status, outcome.stderr),
      };
}

export { lintSpelling };
export type { SpellingVerdict };
