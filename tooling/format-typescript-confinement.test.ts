import { afterEach, describe, expect, test } from "bun:test";
import {
  mkdir,
  mkdtemp,
  rename,
  rm,
  symlink,
  writeFile,
} from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { formatTypeScriptPaths } from "./format-typescript.ts";

const entrypoint = resolve(import.meta.dir, "format-typescript.ts");
const temporaryDirectories: string[] = [];

afterEach(async () => {
  await Promise.all(
    temporaryDirectories
      .splice(0)
      .map((directory) => rm(directory, { recursive: true, force: true })),
  );
});

async function run(command: string[], cwd: string) {
  const process = Bun.spawn(command, { cwd, stdout: "pipe", stderr: "pipe" });
  const [status, stdout, stderr] = await Promise.all([
    process.exited,
    new Response(process.stdout).text(),
    new Response(process.stderr).text(),
  ]);
  return { status, stdout, stderr };
}

async function createRepository() {
  const repository = await mkdtemp(
    join(tmpdir(), "format-typescript-confinement-"),
  );
  temporaryDirectories.push(repository);
  expect((await run(["git", "init", "-q"], repository)).status).toBe(0);
  return repository;
}

describe("format-typescript confinement", () => {
  test("refuses tracked files reached through internal or dangling directory symlinks", async () => {
    const repository = await createRepository();
    const trackedDirectory = join(repository, "tracked");
    const internalTarget = join(repository, "internal-target");
    await mkdir(trackedDirectory);
    await writeFile(
      join(trackedDirectory, "victim.ts"),
      "export const victim=1\n",
    );
    expect(
      (await run(["git", "add", "tracked/victim.ts"], repository)).status,
    ).toBe(0);
    await rename(trackedDirectory, internalTarget);
    await symlink("internal-target", trackedDirectory, "dir");

    const internal = await run([process.execPath, entrypoint], repository);
    expect(internal.status).toBe(1);
    expect(await Bun.file(join(internalTarget, "victim.ts")).text()).toBe(
      "export const victim=1\n",
    );

    await rm(trackedDirectory);
    await symlink("missing-target", trackedDirectory, "dir");
    const dangling = await run([process.execPath, entrypoint], repository);
    expect(dangling.status).toBe(1);
  });

  test("writes through the validated descriptor when a directory path is replaced", async () => {
    const repository = await createRepository();
    const trackedDirectory = join(repository, "tracked");
    const movedDirectory = join(repository, "moved");
    const externalDirectory = await mkdtemp(
      join(tmpdir(), "format-typescript-external-"),
    );
    temporaryDirectories.push(externalDirectory);
    await mkdir(trackedDirectory);
    await writeFile(
      join(trackedDirectory, "victim.ts"),
      "export const victim=1\n",
    );
    await writeFile(
      join(externalDirectory, "victim.ts"),
      "export const outside=1\n",
    );

    const different = await formatTypeScriptPaths(
      ["tracked/victim.ts"],
      false,
      async () => {
        await rename(trackedDirectory, movedDirectory);
        await symlink(externalDirectory, trackedDirectory, "dir");
        return { code: "export const victim = 1;\n", errors: [] };
      },
      repository,
    );

    expect(different).toEqual(["tracked/victim.ts"]);
    expect(await Bun.file(join(externalDirectory, "victim.ts")).text()).toBe(
      "export const outside=1\n",
    );
    expect(await Bun.file(join(movedDirectory, "victim.ts")).text()).toBe(
      "export const victim = 1;\n",
    );
  });
});
