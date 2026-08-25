import { afterEach, describe, expect, test } from "bun:test";
import { join, resolve } from "node:path";
import {
  mkdir,
  mkdtemp,
  rename,
  rm,
  symlink,
  writeFile,
} from "node:fs/promises";
import { formatTypeScriptPaths } from "./format-typescript.ts";
import { tmpdir } from "node:os";

const entrypoint = resolve(import.meta.dir, "format-typescript.ts");
const temporaryDirectories: string[] = [];

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
): Promise<CommandResult> {
  const process = Bun.spawn([...command], {
    cwd,
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

async function createRepository(): Promise<string> {
  const repository = await mkdtemp(
    join(tmpdir(), "format-typescript-confinement-"),
  );
  temporaryDirectories.push(repository);
  const initialization = await run(["git", "init", "-q"], repository);
  expect(initialization.status).toBe(0);
  return repository;
}

type ReplacementFixture = Readonly<{
  externalDirectory: string;
  movedDirectory: string;
  repository: string;
  trackedDirectory: string;
}>;

async function createReplacementFixture(): Promise<ReplacementFixture> {
  const repository = await createRepository();
  const trackedDirectory = join(repository, "tracked");
  const movedDirectory = join(repository, "moved");
  const externalDirectory = await mkdtemp(
    join(tmpdir(), "format-typescript-external-"),
  );
  temporaryDirectories.push(externalDirectory);
  await mkdir(trackedDirectory);
  await writeFile(join(repository, "first.ts"), "export const first=1\n");
  await writeFile(
    join(trackedDirectory, "victim.ts"),
    "export const victim=1\n",
  );
  await writeFile(
    join(externalDirectory, "victim.ts"),
    "export const outside=1\n",
  );
  return { externalDirectory, movedDirectory, repository, trackedDirectory };
}

function replaceDirectoryDuringFormatting(fixture: ReplacementFixture): void {
  expect(
    formatTypeScriptPaths(["first.ts", "tracked/victim.ts"], {
      check: false,
      formatSource: async (path) => {
        if (path === "tracked/victim.ts") {
          await rename(fixture.trackedDirectory, fixture.movedDirectory);
          await symlink(
            fixture.externalDirectory,
            fixture.trackedDirectory,
            "dir",
          );
        }
        const name = path === "first.ts" ? "first" : "victim";
        return { code: `export const ${name} = 1;\n`, errors: [] };
      },
      repositoryRoot: fixture.repository,
    }),
  ).rejects.toThrow("Git could not publish the TypeScript formatting patch");
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
    const addition = await run(["git", "add", "tracked/victim.ts"], repository);
    expect(addition.status).toBe(0);
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

  test("refuses publication when a directory path is replaced after formatting starts", async () => {
    const fixture = await createReplacementFixture();
    replaceDirectoryDuringFormatting(fixture);
    expect(
      await Bun.file(join(fixture.externalDirectory, "victim.ts")).text(),
    ).toBe("export const outside=1\n");
    expect(await Bun.file(join(fixture.repository, "first.ts")).text()).toBe(
      "export const first=1\n",
    );
    expect(
      await Bun.file(join(fixture.movedDirectory, "victim.ts")).text(),
    ).toBe("export const victim=1\n");
  });
});
