import { afterEach, expect, test } from "bun:test";
import {
  findUnexpectedResourceFiles,
  parseResourceFilePolicy,
} from "./check-resource-files.ts";
import { join, resolve } from "node:path";
import { mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";

const fixtureRoots: string[] = [];
const policy = parseResourceFilePolicy({
  resourceDirectories: {
    agents: { files: ["openai.yaml"], mode: "closed" },
    assets: { mode: "open" },
    evals: {
      files: ["evals.json", "trigger-queries.json"],
      mode: "closed",
    },
    references: { mode: "open" },
    scripts: { mode: "open" },
  },
  rootFiles: ["SKILL.md"],
  version: 1,
});

afterEach(async () => {
  await Promise.all(
    fixtureRoots.splice(0).map((root) => rm(root, { recursive: true })),
  );
});

async function createTrackedSkill(paths: readonly string[]): Promise<string> {
  const repositoryRoot = await mkdtemp(join(tmpdir(), "skill-resource-files-"));
  fixtureRoots.push(repositoryRoot);
  const skillRoot = join(repositoryRoot, "skills", "example");
  await mkdir(skillRoot, { recursive: true });
  for (const path of paths) {
    const absolutePath = join(skillRoot, path);
    await mkdir(resolve(absolutePath, ".."), { recursive: true });
    await writeFile(absolutePath, `${path}\n`);
  }
  const git = Bun.spawnSync(["git", "init", "--quiet", repositoryRoot]);
  expect(git.exitCode).toBe(0);
  const add = Bun.spawnSync(["git", "-C", repositoryRoot, "add", "--", "."]);
  expect(add.exitCode).toBe(0);
  return skillRoot;
}

test("rejects a malformed policy instead of choosing a permissive default", () => {
  expect(() =>
    parseResourceFilePolicy({
      ...policy,
      resourceDirectories: {
        ...policy.resourceDirectories,
        evals: { mode: "permissive" },
      },
    }),
  ).toThrow();
});

test("rejects every file outside a closed directory policy", () => {
  expect(
    findUnexpectedResourceFiles(
      [
        "SKILL.md",
        "agents/provider.yaml",
        "evals/cases.md",
        "evals/files/input.csv",
      ],
      policy,
    ),
  ).toEqual([
    {
      convention: "agents/ admits only openai.yaml",
      path: "agents/provider.yaml",
    },
    {
      convention: "evals/ admits only evals.json and trigger-queries.json",
      path: "evals/cases.md",
    },
    {
      convention: "evals/ admits only evals.json and trigger-queries.json",
      path: "evals/files/input.csv",
    },
  ]);
});

test("accepts both eval variants and extensible resource directories", () => {
  expect(
    findUnexpectedResourceFiles(
      [
        "SKILL.md",
        "agents/openai.yaml",
        "assets/templates/report.md",
        "evals/evals.json",
        "evals/trigger-queries.json",
        "references/provider/contracts.md",
        "scripts/check.ts",
      ],
      policy,
    ),
  ).toEqual([]);
});

test("rejects unexpected root files and directories", () => {
  expect(
    findUnexpectedResourceFiles(
      ["SKILL.md", "README.md", "examples/case.md"],
      policy,
    ),
  ).toEqual([
    {
      convention:
        "skill root admits only SKILL.md and agents/, assets/, evals/, references/ and scripts/",
      path: "README.md",
    },
    {
      convention:
        "skill root admits only SKILL.md and agents/, assets/, evals/, references/ and scripts/",
      path: "examples/case.md",
    },
  ]);
});

test("command fails and identifies every unexpected tracked file", async () => {
  const skillRoot = await createTrackedSkill([
    "SKILL.md",
    "evals/cases.md",
    "evals/trigger-queries.json",
  ]);
  const result = Bun.spawnSync([
    process.execPath,
    resolve(import.meta.dir, "check-resource-files.ts"),
    skillRoot,
  ]);
  const stderr = result.stderr.toString();
  expect(result.exitCode).toBe(1);
  expect(stderr).toContain("evals/cases.md");
  expect(stderr).toContain(
    "evals/ admits only evals.json and trigger-queries.json",
  );
  expect(stderr).not.toContain("evals/trigger-queries.json");
});

test("command accepts the two valid eval layouts", async () => {
  for (const evalFile of ["evals/trigger-queries.json", "evals/evals.json"]) {
    const skillRoot = await createTrackedSkill(["SKILL.md", evalFile]);
    const result = Bun.spawnSync([
      process.execPath,
      resolve(import.meta.dir, "check-resource-files.ts"),
      skillRoot,
    ]);
    expect(result.exitCode).toBe(0);
    expect(result.stdout.toString()).toContain("Resource files: PASS");
  }
});

test("command fails closed when tracked files cannot be enumerated", async () => {
  const skillRoot = await mkdtemp(join(tmpdir(), "skill-resource-files-"));
  fixtureRoots.push(skillRoot);
  await writeFile(join(skillRoot, "SKILL.md"), "fixture\n");
  const result = Bun.spawnSync([
    process.execPath,
    resolve(import.meta.dir, "check-resource-files.ts"),
    skillRoot,
  ]);
  expect(result.exitCode).toBe(1);
  expect(result.stderr.toString()).toContain(
    "Git could not resolve the skill repository",
  );
});
