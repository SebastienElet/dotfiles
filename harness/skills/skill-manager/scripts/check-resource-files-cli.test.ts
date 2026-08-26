import { afterEach, expect, test } from "bun:test";
import {
  chmod,
  mkdir,
  mkdtemp,
  rm,
  symlink,
  writeFile,
} from "node:fs/promises";
import { join, resolve } from "node:path";
import { tmpdir } from "node:os";

const fixtureRoots: string[] = [];
const readableDirectoryMode = 0o700;
type CheckerResult = Readonly<{
  exitCode: number;
  stderr: Uint8Array;
  stdout: Uint8Array;
}>;

afterEach(async () => {
  await Promise.all(
    fixtureRoots.splice(0).map((root) => rm(root, { recursive: true })),
  );
});

async function writeSkillFiles(
  skillRoot: string,
  paths: readonly string[],
): Promise<void> {
  for (const path of paths) {
    const absolutePath = join(skillRoot, path);
    await mkdir(resolve(absolutePath, ".."), { recursive: true });
    await writeFile(absolutePath, `${path}\n`);
  }
}

async function createSkillFixture(
  paths: readonly string[],
  additionalPaths: readonly string[] = [],
): Promise<string> {
  const repositoryRoot = await mkdtemp(join(tmpdir(), "skill-resource-files-"));
  fixtureRoots.push(repositoryRoot);
  const skillRoot = join(repositoryRoot, "skills", "example");
  await mkdir(skillRoot, { recursive: true });
  await writeSkillFiles(skillRoot, paths);
  await writeSkillFiles(skillRoot, additionalPaths);
  const init = Bun.spawnSync(["git", "init", "--quiet", repositoryRoot]);
  expect(init.exitCode).toBe(0);
  return skillRoot;
}

function runChecker(skillRoot: string): CheckerResult {
  return Bun.spawnSync([
    process.execPath,
    resolve(import.meta.dir, "check-resource-files.ts"),
    skillRoot,
  ]);
}

test("command fails and identifies every unexpected file", async () => {
  const skillRoot = await createSkillFixture([
    "SKILL.md",
    "evals/cases.md",
    "evals/trigger-queries.json",
  ]);
  const result = runChecker(skillRoot);
  const stderr = result.stderr.toString();
  expect(result.exitCode).toBe(1);
  expect(stderr).toContain("evals/cases.md");
  expect(stderr).toContain(
    "evals/ admits only evals.json and trigger-queries.json",
  );
  expect(stderr).not.toContain("evals/trigger-queries.json");
});

test("command rejects an unexpected file added after setup", async () => {
  const skillRoot = await createSkillFixture(["SKILL.md"], ["evals/cases.md"]);
  const result = runChecker(skillRoot);
  expect(result.exitCode).toBe(1);
  expect(result.stderr.toString()).toContain("evals/cases.md");
});

test("command rejects an ignored runtime artifact", async () => {
  const skillRoot = await createSkillFixture(["SKILL.md"]);
  const repositoryRoot = resolve(skillRoot, "../..");
  await writeFile(
    join(repositoryRoot, ".gitignore"),
    "skills/example/assets/runtime.log\n",
  );
  await writeSkillFiles(skillRoot, ["assets/runtime.log"]);
  const ignored = Bun.spawnSync([
    "git",
    "-C",
    repositoryRoot,
    "check-ignore",
    "--quiet",
    "--",
    "skills/example/assets/runtime.log",
  ]);
  expect(ignored.exitCode).toBe(0);
  const result = runChecker(skillRoot);
  expect(result.exitCode).toBe(1);
  expect(result.stderr.toString()).toContain("assets/runtime.log");
});

test("command fails closed when Git cannot audit ignore rules", async () => {
  const skillRoot = await createSkillFixture(["SKILL.md"]);
  await rm(resolve(skillRoot, "../../.git"), { recursive: true });
  const result = runChecker(skillRoot);
  expect(result.exitCode).toBe(1);
  expect(result.stderr.toString()).toContain("Git ignore audit failed");
});

test("command rejects an unexpected empty directory", async () => {
  const skillRoot = await createSkillFixture(["SKILL.md"]);
  await mkdir(join(skillRoot, "examples"));
  const result = runChecker(skillRoot);
  expect(result.exitCode).toBe(1);
  expect(result.stderr.toString()).toContain("examples/");
});

test("command rejects regular files named after resource directories", async () => {
  for (const path of ["assets", "references", "scripts"]) {
    const skillRoot = await createSkillFixture(["SKILL.md", path]);
    const result = runChecker(skillRoot);
    expect(result.exitCode).toBe(1);
    expect(result.stderr.toString()).toContain(path);
  }
});

test("command rejects a symbolic link used as the canonical skill root", async () => {
  const skillRoot = await createSkillFixture(["SKILL.md"]);
  const linkedSkillRoot = `${skillRoot}-link`;
  await symlink(skillRoot, linkedSkillRoot);
  const result = runChecker(linkedSkillRoot);
  expect(result.exitCode).toBe(1);
  expect(result.stderr.toString()).toContain("regular directory");
});

test("command rejects symbolic links and special files", async () => {
  const skillRoot = await createSkillFixture(["SKILL.md"]);
  await mkdir(join(skillRoot, "evals"), { recursive: true });
  await symlink(skillRoot, join(skillRoot, "evals", "evals.json"));
  const fifoPath = join(skillRoot, "evals", "trigger-queries.json");
  const fifo = Bun.spawnSync(["mkfifo", fifoPath]);
  expect(fifo.exitCode).toBe(0);
  const result = runChecker(skillRoot);
  const stderr = result.stderr.toString();
  expect(result.exitCode).toBe(1);
  expect(stderr).toContain("evals/evals.json");
  expect(stderr).toContain("evals/trigger-queries.json");
});

test("command fails closed on an unreadable nested directory", async () => {
  const skillRoot = await createSkillFixture(["SKILL.md"]);
  const unreadableDirectory = join(skillRoot, "references", "private");
  await mkdir(unreadableDirectory, { recursive: true });
  await chmod(unreadableDirectory, 0o000);
  try {
    const result = runChecker(skillRoot);
    expect(result.exitCode).toBe(1);
    expect(result.stderr.toString()).toContain("EACCES");
  } finally {
    await chmod(unreadableDirectory, readableDirectoryMode);
  }
});

test("command accepts the two valid eval layouts", async () => {
  for (const evalFile of ["evals/trigger-queries.json", "evals/evals.json"]) {
    const skillRoot = await createSkillFixture(["SKILL.md", evalFile]);
    const result = runChecker(skillRoot);
    expect(result.exitCode).toBe(0);
    expect(result.stdout.toString()).toContain("Resource files: PASS");
  }
});

test("command fails closed when the skill root cannot be resolved", async () => {
  const fixtureRoot = await mkdtemp(join(tmpdir(), "skill-resource-files-"));
  fixtureRoots.push(fixtureRoot);
  const result = runChecker(join(fixtureRoot, "missing"));
  expect(result.exitCode).toBe(1);
  expect(result.stderr.toString()).toContain(
    "The skill root could not be resolved",
  );
});
