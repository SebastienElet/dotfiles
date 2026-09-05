import { loadCases, loadTrigger } from "./sources.ts";
import { join } from "node:path";
import { readReport } from "./evidence.ts";
import { readdirSync } from "node:fs";

function validateEvaluations(repository: string): string {
  const cases = loadCases(repository);
  const result = Bun.spawnSync([
    "git",
    "-C",
    repository,
    "ls-files",
    "-z",
    "harness/skills/*/evals/trigger-queries.json",
    ".agents/skills/*/evals/trigger-queries.json",
  ]);
  if (result.exitCode !== 0) {
    throw new Error("Cannot enumerate tracked activation contracts");
  }
  const paths = result.stdout.toString().split("\0").filter(Boolean);
  if (
    !paths.includes("harness/skills/code-search/evals/trigger-queries.json")
  ) {
    throw new Error("Missing code-search activation contract");
  }
  for (const path of paths) {
    try {
      loadTrigger(repository, path);
    } catch (error) {
      throw new Error(`Invalid activation contract: ${path}`, { cause: error });
    }
  }
  return `${cases.length} behavioral cases; ${paths.length} activation contracts valid`;
}

function validateEvidence(repository: string): string {
  const directory = join(repository, "harness/evals/evidence");
  const files = readdirSync(directory)
    .filter((path) => path.endsWith(".json"))
    .toSorted();
  for (const file of files) {
    try {
      readReport(join(directory, file));
    } catch (error) {
      throw new Error(`Invalid evidence: ${file}`, { cause: error });
    }
  }
  return `${files.length} historical reports valid; live evidence is optional`;
}

export { validateEvaluations, validateEvidence };
