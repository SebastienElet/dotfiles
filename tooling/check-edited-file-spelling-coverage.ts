import {
  type BoundedCommandOptions,
  commandFailure,
  runBounded,
} from "./bounded-command.ts";
import { skillMarkdownPathspecs } from "./skill-markdown-paths.ts";
import { z } from "zod";

type SpellingSelection = "file" | "force-checked-file" | "uncovered";

type SpellingCoverage = Readonly<{
  files: readonly string[];
  globs: readonly string[];
  skillMarkdown: readonly string[];
}>;

type CoverageLookup =
  | Readonly<{ coverage: SpellingCoverage; kind: "resolved" }>
  | Readonly<{ kind: "failed"; reason: string }>;

const repositoryProjectSchema = z.object({
  fileGroups: z.object({
    cspellFiles: z.object({
      files: z.array(z.string().min(1)),
      globs: z.array(z.string().min(1)),
    }),
  }),
});

function parseJson(text: string): unknown {
  try {
    return JSON.parse(text);
  } catch {
    return undefined;
  }
}

async function cspellFileGroup(
  options: BoundedCommandOptions,
): Promise<
  | Readonly<{ group: z.infer<typeof repositoryProjectSchema>; kind: "read" }>
  | Readonly<{ kind: "failed"; reason: string }>
> {
  const outcome = await runBounded(
    "moon",
    ["project", "repository", "--json"],
    options,
  );
  if (outcome.kind === "failed") {
    return outcome;
  }
  if (outcome.status !== 0) {
    return {
      kind: "failed",
      reason: commandFailure("moon", outcome.status, outcome.stderr),
    };
  }
  const project = repositoryProjectSchema.safeParse(parseJson(outcome.stdout));
  return project.success
    ? { group: project.data, kind: "read" }
    : {
        kind: "failed",
        reason: `moon returned an unexpected project description: ${z.prettifyError(project.error)}`,
      };
}

async function skillMarkdownFiles(
  options: BoundedCommandOptions,
): Promise<
  | Readonly<{ kind: "listed"; paths: readonly string[] }>
  | Readonly<{ kind: "failed"; reason: string }>
> {
  const outcome = await runBounded(
    "git",
    [
      "ls-files",
      "-z",
      "--cached",
      "--others",
      "--exclude-standard",
      "--",
      ...skillMarkdownPathspecs,
    ],
    options,
  );
  if (outcome.kind === "failed") {
    return outcome;
  }
  return outcome.status === 0
    ? {
        kind: "listed",
        paths: outcome.stdout.split("\0").filter((path) => path !== ""),
      }
    : {
        kind: "failed",
        reason: commandFailure("git ls-files", outcome.status, outcome.stderr),
      };
}

async function readSpellingCoverage(
  options: BoundedCommandOptions,
): Promise<CoverageLookup> {
  const [project, skills] = await Promise.all([
    cspellFileGroup(options),
    skillMarkdownFiles(options),
  ]);
  if (project.kind === "failed") {
    return project;
  }
  if (skills.kind === "failed") {
    return skills;
  }
  const { files, globs } = project.group.fileGroups.cspellFiles;
  return {
    coverage: {
      files,
      globs,
      skillMarkdown: skills.paths,
    },
    kind: "resolved",
  };
}

function spellingSelection(
  coverage: SpellingCoverage,
  relativePath: string,
): SpellingSelection {
  if (
    coverage.files.includes(relativePath) ||
    coverage.globs.some((pattern) => new Bun.Glob(pattern).match(relativePath))
  ) {
    return "file";
  }
  return coverage.skillMarkdown.includes(relativePath)
    ? "force-checked-file"
    : "uncovered";
}

export { readSpellingCoverage, spellingSelection };
export type { CoverageLookup, SpellingSelection };
