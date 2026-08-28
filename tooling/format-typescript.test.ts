import { afterEach, expect, test } from "bun:test";
import {
  chmod,
  link,
  mkdir,
  mkdtemp,
  rm,
  symlink,
  writeFile,
} from "node:fs/promises";
import { join, resolve } from "node:path";
import { tmpdir } from "node:os";

const entrypoint = resolve(import.meta.dir, "format-typescript.ts");
const temporaryDirectories: string[] = [];
const executableFileMode = 0o755;
const unavailableCommandExitCode = 42;
const invalidInvocationExitCode = 2;
const trackedFiles = [
  "-leading.ts",
  ".hidden/future.ts",
  "ignored/forced.tsx",
  "types/future file.mts",
  "types/future.cts",
  "types/future.d.ts",
];

type CommandResult = Readonly<{
  status: number;
  stderr: string;
  stdout: string;
}>;

afterEach(async () => {
  await Promise.all(
    temporaryDirectories
      .splice(0)
      .map((directory) => rm(directory, { force: true, recursive: true })),
  );
});

async function run(
  command: readonly string[],
  cwd: string,
  environment: Readonly<Record<string, string | undefined>> = {},
): Promise<CommandResult> {
  const process = Bun.spawn([...command], {
    cwd,
    env: { ...Bun.env, ...environment },
    stderr: "pipe",
    stdout: "pipe",
  });
  const [status, stdout, stderr] = await Promise.all([
    process.exited,
    new Response(process.stdout).text(),
    new Response(process.stderr).text(),
  ]);
  return { status, stderr, stdout };
}

function runEntrypoint(
  commandArguments: readonly string[],
  cwd: string,
  environment: Readonly<Record<string, string | undefined>> = {},
): Promise<CommandResult> {
  return run(
    [process.execPath, entrypoint, ...commandArguments],
    cwd,
    environment,
  );
}

async function createRepository(): Promise<string> {
  const repository = await mkdtemp(join(tmpdir(), "format-typescript-"));
  temporaryDirectories.push(repository);
  const initialization = await run(["git", "init", "-q"], repository);
  expect(initialization.status).toBe(0);
  return repository;
}

async function createFakeCommand(
  directory: string,
  name: string,
  body: string,
): Promise<void> {
  await mkdir(directory, { recursive: true });
  const path = join(directory, name);
  await writeFile(path, `#!/usr/bin/env bash\n${body}\n`);
  await chmod(path, executableFileMode);
}

test("passes every tracked TypeScript path to Oxfmt without including untracked files", async () => {
  const repository = await createRepository();
  await Promise.all(
    [...trackedFiles, "untracked.ts", "not-typescript.js"].map(async (file) => {
      await mkdir(join(repository, file, ".."), { recursive: true });
      await writeFile(join(repository, file), "export const value=1\n");
    }),
  );
  await writeFile(join(repository, ".gitignore"), "ignored/\n");
  const rejectedIgnoredFile = await run(
    ["git", "add", "--", ".gitignore", ...trackedFiles],
    repository,
  );
  const trackedAddition = await run(
    [
      "git",
      "add",
      "--",
      ".gitignore",
      ...trackedFiles.filter((file) => !file.startsWith("ignored/")),
    ],
    repository,
  );
  const forcedAddition = await run(
    ["git", "add", "-f", "--", "ignored/forced.tsx"],
    repository,
  );
  expect(rejectedIgnoredFile.status).not.toBe(0);
  expect(trackedAddition.status).toBe(0);
  expect(forcedAddition.status).toBe(0);

  const initialCheck = await runEntrypoint(["--check"], repository);
  const formatting = await runEntrypoint([], repository);
  const finalCheck = await runEntrypoint(["--check"], repository);
  expect(initialCheck.status).toBe(1);
  expect(formatting.status).toBe(0);
  expect(finalCheck.status).toBe(0);
  for (const file of trackedFiles) {
    expect(await Bun.file(join(repository, file)).text()).not.toContain(
      "value=1",
    );
  }
  expect(await Bun.file(join(repository, "untracked.ts")).text()).toBe(
    "export const value=1\n",
  );
});

test("formats surviving tracked files while a tracked TypeScript file is deleted", async () => {
  const repository = await createRepository();
  await writeFile(join(repository, "kept.ts"), "export const kept=1\n");
  await writeFile(join(repository, "removed.ts"), "export const removed=1\n");
  const addition = await run(
    ["git", "add", "kept.ts", "removed.ts"],
    repository,
  );
  expect(addition.status).toBe(0);
  await rm(join(repository, "removed.ts"));

  const result = await runEntrypoint([], repository);

  expect(result.status).toBe(0);
  expect(await Bun.file(join(repository, "kept.ts")).text()).toBe(
    "export const kept = 1;\n",
  );
});

test("fails closed when discovery is empty or unavailable", async () => {
  const repository = await createRepository();
  await writeFile(
    join(repository, "not-typescript.js"),
    "export const value = 1;\n",
  );
  const addition = await run(["git", "add", "not-typescript.js"], repository);
  expect(addition.status).toBe(0);
  const empty = await runEntrypoint(["--check"], repository);
  expect(empty.status).toBe(1);
  expect(empty.stderr).toContain("No tracked TypeScript files found");

  const binaryDirectory = join(repository, "bin");
  await createFakeCommand(
    binaryDirectory,
    "git",
    `exit ${unavailableCommandExitCode}`,
  );
  const unavailable = await runEntrypoint(["--check"], repository, {
    PATH: `${binaryDirectory}:${Bun.env.PATH}`,
  });
  expect(unavailable.status).toBe(unavailableCommandExitCode);
});

test("refuses a tracked symlink before formatting its external target", async () => {
  const repository = await createRepository();
  const externalDirectory = await mkdtemp(
    join(tmpdir(), "format-typescript-external-"),
  );
  temporaryDirectories.push(externalDirectory);
  const externalTarget = join(externalDirectory, "outside.ts");
  await writeFile(externalTarget, "export const outside=1\n");
  await symlink(externalTarget, join(repository, "linked.ts"));
  const addition = await run(["git", "add", "linked.ts"], repository);
  expect(addition.status).toBe(0);

  const result = await runEntrypoint([], repository);
  expect(result.status).not.toBe(0);
  expect(await Bun.file(externalTarget).text()).toBe(
    "export const outside=1\n",
  );
});

test("refuses a tracked hard link before formatting its external inode", async () => {
  const repository = await createRepository();
  const externalDirectory = await mkdtemp(
    join(tmpdir(), "format-typescript-external-"),
  );
  temporaryDirectories.push(externalDirectory);
  const externalTarget = join(externalDirectory, "outside.ts");
  await writeFile(externalTarget, "export const outside=1\n");
  await link(externalTarget, join(repository, "linked.ts"));
  const addition = await run(["git", "add", "linked.ts"], repository);
  expect(addition.status).toBe(0);

  const result = await runEntrypoint([], repository);
  expect(result.status).not.toBe(0);
  expect(await Bun.file(externalTarget).text()).toBe(
    "export const outside=1\n",
  );
});

test("rejects malformed Git path evidence", async () => {
  const repository = await createRepository();
  const binaryDirectory = join(repository, "bin");
  await createFakeCommand(binaryDirectory, "git", String.raw`printf '\377\0'`);

  const result = await runEntrypoint(["--check"], repository, {
    PATH: `${binaryDirectory}:${Bun.env.PATH}`,
  });
  expect(result.status).toBe(1);
  expect(result.stderr).not.toBe("");
});

test("rejects unknown arguments and invalid TypeScript", async () => {
  const repository = await createRepository();
  const invalid = await runEntrypoint(["--write"], repository);
  expect(invalid.status).toBe(invalidInvocationExitCode);

  await writeFile(join(repository, "future.ts"), "export const = 1;\n");
  const addition = await run(["git", "add", "future.ts"], repository);
  expect(addition.status).toBe(0);
  const failed = await runEntrypoint(["--check"], repository);
  expect(failed.status).not.toBe(0);
});
