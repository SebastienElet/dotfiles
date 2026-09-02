import {
  type InvariantRegistry,
  parseInvariantRegistry,
} from "./invariant-registry-contract.ts";
import { join, relative, resolve } from "node:path";
import { mkdtemp, readFile, rm, symlink, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";

const entrypoint = resolve(import.meta.dir, "invariant-registry-cli.ts");
const repositoryRoot = resolve(import.meta.dir, "..");
const fixtureDirectory = resolve(
  import.meta.dir,
  "invariant-registry-fixtures",
);
const fixturePrefix = ".registry-cli-";
const temporaryDirectories: string[] = [];

type CliOutcome = Readonly<{
  exitCode: number;
  stderr: string;
  stdout: string;
}>;

const cleanup = async (): Promise<void> => {
  await Promise.all(
    temporaryDirectories
      .splice(0)
      .map((directory) => rm(directory, { force: true, recursive: true })),
  );
};

const runRegistryCli = async (registryPath?: string): Promise<CliOutcome> => {
  const directory = await mkdtemp(join(tmpdir(), fixturePrefix));
  temporaryDirectories.push(directory);
  const command = [process.execPath, entrypoint];
  if (registryPath !== undefined) {
    command.push(registryPath);
  }
  const child = Bun.spawn(command, {
    cwd: directory,
    stderr: "pipe",
    stdout: "pipe",
  });
  const [exitCode, stdout, stderr] = await Promise.all([
    child.exited,
    new Response(child.stdout).text(),
    new Response(child.stderr).text(),
  ]);
  return { exitCode, stderr, stdout };
};

const createRegistry = async (contents: string): Promise<string> => {
  const directory = await mkdtemp(join(fixtureDirectory, fixturePrefix));
  temporaryDirectories.push(directory);
  const path = join(directory, "registry.json");
  await writeFile(path, contents);
  return relative(repositoryRoot, path);
};

const createExternalFile = async (
  name: string,
  contents: string,
): Promise<string> => {
  const directory = await mkdtemp(join(tmpdir(), fixturePrefix));
  temporaryDirectories.push(directory);
  const path = join(directory, name);
  await writeFile(path, contents);
  return path;
};

const createLinkedRegistry = async (target: string): Promise<string> => {
  const directory = await mkdtemp(join(fixtureDirectory, fixturePrefix));
  temporaryDirectories.push(directory);
  const path = join(directory, "registry.json");
  await symlink(target, path);
  return relative(repositoryRoot, path);
};

const createLinkedOracle = async (target: string): Promise<string> => {
  const directory = await mkdtemp(join(fixtureDirectory, fixturePrefix));
  temporaryDirectories.push(directory);
  const path = join(directory, "oracle.test.ts");
  await symlink(target, path);
  return relative(repositoryRoot, path);
};

const fixturePath = (name: string): string => join(fixtureDirectory, name);

const readRegistry = async (path: string): Promise<InvariantRegistry> =>
  parseInvariantRegistry(JSON.parse(await readFile(path, "utf8")));

const mutatedFixture = async (
  fixtureName: string,
  mutate: (source: string) => string,
): Promise<string> =>
  createRegistry(mutate(await readFile(fixturePath(fixtureName), "utf8")));

export {
  cleanup,
  createExternalFile,
  createLinkedOracle,
  createLinkedRegistry,
  createRegistry,
  fixturePath,
  mutatedFixture,
  readRegistry,
  repositoryRoot,
  runRegistryCli,
};
