import { afterEach, describe, expect, test } from "bun:test";
import {
  chmod,
  link,
  mkdir,
  mkdtemp,
  rm,
  symlink,
  writeFile,
} from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const entrypoint = resolve(import.meta.dir, "format-typescript.ts");
const temporaryDirectories: string[] = [];

afterEach(async () => {
  await Promise.all(
    temporaryDirectories
      .splice(0)
      .map((directory) => rm(directory, { recursive: true, force: true })),
  );
});

async function run(
  command: string[],
  cwd: string,
  environment: Record<string, string | undefined> = {},
) {
  const process = Bun.spawn(command, {
    cwd,
    env: { ...Bun.env, ...environment },
    stdout: "pipe",
    stderr: "pipe",
  });
  const [status, stdout, stderr] = await Promise.all([
    process.exited,
    new Response(process.stdout).text(),
    new Response(process.stderr).text(),
  ]);
  return { status, stdout, stderr };
}

function runEntrypoint(
  arguments_: string[],
  cwd: string,
  environment: Record<string, string | undefined> = {},
) {
  return run([process.execPath, entrypoint, ...arguments_], cwd, environment);
}

async function createRepository() {
  const repository = await mkdtemp(join(tmpdir(), "format-typescript-"));
  temporaryDirectories.push(repository);
  expect((await run(["git", "init", "-q"], repository)).status).toBe(0);
  return repository;
}

async function createFakeCommand(
  directory: string,
  name: string,
  body: string,
) {
  await mkdir(directory, { recursive: true });
  const path = join(directory, name);
  await writeFile(path, `#!/usr/bin/env bash\n${body}\n`);
  await chmod(path, 0o755);
}

describe("format-typescript entry point", () => {
  test("passes every tracked TypeScript path to Oxfmt without including untracked files", async () => {
    const repository = await createRepository();
    const trackedFiles = [
      "-leading.ts",
      ".hidden/future.ts",
      "ignored/forced.tsx",
      "types/future file.mts",
      "types/future.cts",
      "types/future.d.ts",
    ];
    await Promise.all(
      trackedFiles
        .concat(["untracked.ts", "not-typescript.js"])
        .map(async (file) => {
          await mkdir(join(repository, file, ".."), { recursive: true });
          await writeFile(join(repository, file), "export const value=1\n");
        }),
    );
    await writeFile(join(repository, ".gitignore"), "ignored/\n");
    expect(
      (
        await run(
          ["git", "add", "--", ".gitignore", ...trackedFiles],
          repository,
        )
      ).status,
    ).not.toBe(0);
    expect(
      (
        await run(
          [
            "git",
            "add",
            "--",
            ".gitignore",
            ...trackedFiles.filter((file) => !file.startsWith("ignored/")),
          ],
          repository,
        )
      ).status,
    ).toBe(0);
    expect(
      (await run(["git", "add", "-f", "--", "ignored/forced.tsx"], repository))
        .status,
    ).toBe(0);

    expect((await runEntrypoint(["--check"], repository)).status).toBe(1);
    expect((await runEntrypoint([], repository)).status).toBe(0);
    expect((await runEntrypoint(["--check"], repository)).status).toBe(0);
    for (const file of trackedFiles) {
      expect(await Bun.file(join(repository, file)).text()).not.toContain(
        "value=1",
      );
    }
    expect(await Bun.file(join(repository, "untracked.ts")).text()).toBe(
      "export const value=1\n",
    );
  });

  test("fails closed when discovery is empty or unavailable", async () => {
    const repository = await createRepository();
    await writeFile(
      join(repository, "not-typescript.js"),
      "export const value = 1;\n",
    );
    expect(
      (await run(["git", "add", "not-typescript.js"], repository)).status,
    ).toBe(0);
    const empty = await runEntrypoint(["--check"], repository);
    expect(empty.status).toBe(1);
    expect(empty.stderr).toContain("No tracked TypeScript files found");

    const binaryDirectory = join(repository, "bin");
    await createFakeCommand(binaryDirectory, "git", "exit 42");
    const unavailable = await runEntrypoint(["--check"], repository, {
      PATH: `${binaryDirectory}:${Bun.env.PATH}`,
    });
    expect(unavailable.status).toBe(42);
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
    expect((await run(["git", "add", "linked.ts"], repository)).status).toBe(0);

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
    expect((await run(["git", "add", "linked.ts"], repository)).status).toBe(0);

    const result = await runEntrypoint([], repository);
    expect(result.status).not.toBe(0);
    expect(await Bun.file(externalTarget).text()).toBe(
      "export const outside=1\n",
    );
  });

  test("rejects malformed Git path evidence", async () => {
    const repository = await createRepository();
    const binaryDirectory = join(repository, "bin");
    await createFakeCommand(binaryDirectory, "git", "printf '\\377\\0'");

    const result = await runEntrypoint(["--check"], repository, {
      PATH: `${binaryDirectory}:${Bun.env.PATH}`,
    });
    expect(result.status).toBe(1);
    expect(result.stderr).not.toBe("");
  });

  test("rejects unknown arguments and invalid TypeScript", async () => {
    const repository = await createRepository();
    const invalid = await runEntrypoint(["--write"], repository);
    expect(invalid.status).toBe(2);

    await writeFile(join(repository, "future.ts"), "export const = 1;\n");
    expect((await run(["git", "add", "future.ts"], repository)).status).toBe(0);
    const failed = await runEntrypoint(["--check"], repository);
    expect(failed.status).not.toBe(0);
  });
});
