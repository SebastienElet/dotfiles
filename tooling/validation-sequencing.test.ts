import { afterEach, expect, test } from "bun:test";
import { join, resolve } from "node:path";
import { mkdtemp, rm, symlink, writeFile } from "node:fs/promises";
import { lintTrackedTypeScript } from "./lint-typescript.ts";
import { tmpdir } from "node:os";

const repositoryRoot = resolve(import.meta.dir, "..");
const oxlint = resolve(repositoryRoot, "node_modules/.bin/oxlint");
const temporaryDirectories: string[] = [];

afterEach(async () => {
  await Promise.all(
    temporaryDirectories
      .splice(0)
      .map((directory) => rm(directory, { force: true, recursive: true })),
  );
});

const run = async (
  command: readonly string[],
  cwd: string,
): Promise<Readonly<{ status: number; stderr: string; stdout: string }>> => {
  const child = Bun.spawn([...command], {
    cwd,
    stderr: "pipe",
    stdout: "pipe",
  });
  const [status, stderr, stdout] = await Promise.all([
    child.exited,
    new Response(child.stderr).text(),
    new Response(child.stdout).text(),
  ]);
  return { status, stderr, stdout };
};

const assertCommandSuccess = (
  result: Readonly<{ status: number; stderr: string; stdout: string }>,
): void => {
  if (result.status !== 0) {
    throw new Error(`${result.stdout}\n${result.stderr}`);
  }
};

test("keeps a staging failure visible", async () => {
  const repository = await mkdtemp(join(tmpdir(), "validation-sequencing-"));
  temporaryDirectories.push(repository);
  assertCommandSuccess(await run(["git", "init", "-q"], repository));

  const failedAddition = await run(["git", "add", "missing.ts"], repository);
  expect(() => {
    assertCommandSuccess(failedAddition);
  }).toThrow("pathspec");
});

test("lint only validates a new TypeScript file after it enters the index", async () => {
  const repository = await mkdtemp(join(tmpdir(), "validation-sequencing-"));
  temporaryDirectories.push(repository);
  const initialization = await run(["git", "init", "-q"], repository);
  assertCommandSuccess(initialization);
  await Promise.all([
    symlink(
      join(repositoryRoot, "node_modules"),
      join(repository, "node_modules"),
      "dir",
    ),
    Bun.write(
      join(repository, ".oxlintrc.json"),
      Bun.file(join(repositoryRoot, ".oxlintrc.json")),
    ),
    writeFile(
      join(repository, "tsconfig.json"),
      JSON.stringify({ compilerOptions: { strict: true } }),
    ),
    writeFile(
      join(repository, "tracked.ts"),
      'export const value = "valid";\n',
    ),
    writeFile(join(repository, "new.ts"), "Promise.resolve();\n"),
  ]);
  const trackedAddition = await run(["git", "add", "tracked.ts"], repository);
  assertCommandSuccess(trackedAddition);

  const beforeStaging = await lintTrackedTypeScript(repository, oxlint);
  expect(beforeStaging.status).toBe(0);

  const newAddition = await run(["git", "add", "new.ts"], repository);
  assertCommandSuccess(newAddition);
  const afterStaging = await lintTrackedTypeScript(repository, oxlint);
  expect(afterStaging.status).toBe(1);
  expect(`${afterStaging.stdout}\n${afterStaging.stderr}`).toContain(
    "typescript(no-floating-promises)",
  );
});
