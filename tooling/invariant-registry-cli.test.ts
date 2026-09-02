import { afterEach, expect, test } from "bun:test";
import { join, relative, resolve } from "node:path";
import { mkdtemp, readFile, rm, symlink, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";

const entrypoint = resolve(import.meta.dir, "invariant-registry-cli.ts");
const repositoryRoot = resolve(import.meta.dir, "..");
const fixtureDirectory = resolve(
  import.meta.dir,
  "invariant-registry-fixtures",
);
const invalidUtf8Byte = 0xff;
const fixturePrefix = ".registry-cli-";
const temporaryDirectories: string[] = [];

type CliOutcome = Readonly<{
  exitCode: number;
  stderr: string;
  stdout: string;
}>;

afterEach(async (): Promise<void> => {
  await Promise.all(
    temporaryDirectories
      .splice(0)
      .map((directory) => rm(directory, { force: true, recursive: true })),
  );
});

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

const mutatedFixture = async (
  fixtureName: string,
  mutate: (source: string) => string,
): Promise<string> => {
  const contents = await readFile(join(fixtureDirectory, fixtureName), "utf8");
  return createRegistry(mutate(contents));
};

test("validates the canonical empty registry", async () => {
  const outcome = await runRegistryCli();

  expect(outcome.exitCode).toBe(0);
  expect(outcome.stdout).toBe(
    "Invariant registry passed: harness/invariants/registry.json\n",
  );
  expect(outcome.stderr).toBe("");
});

test.each([
  ["missing file", "missing.json", "unable to read invariant registry"],
  ["invalid JSON", "{", "valid JSON"],
  [
    "unknown version",
    '{"version":2,"invariants":[]}',
    "invalid invariant registry",
  ],
] as const)("fails closed for %s", async (_name, contents, diagnostic) => {
  const path =
    contents === "missing.json" ? contents : await createRegistry(contents);
  const outcome = await runRegistryCli(path);

  expect(outcome.exitCode).not.toBe(0);
  expect(outcome.stdout).toBe("");
  expect(outcome.stderr).toContain(diagnostic);
});

test("fails closed for invalid UTF-8", async () => {
  const path = await createRegistry("\u0000");
  await writeFile(
    resolve(repositoryRoot, path),
    Buffer.from([invalidUtf8Byte]),
  );
  const outcome = await runRegistryCli(path);

  expect(outcome.exitCode).not.toBe(0);
  expect(outcome.stdout).toBe("");
  expect(outcome.stderr).toContain("valid UTF-8");
});

test("refuses a registry symlink that resolves outside the repository", async () => {
  const target = await createExternalFile(
    "registry.json",
    '{"version":1,"invariants":[]}',
  );
  const outcome = await runRegistryCli(await createLinkedRegistry(target));

  expect(outcome.exitCode).not.toBe(0);
  expect(outcome.stdout).toBe("");
  expect(outcome.stderr).toContain("within the repository");
});

test("sanitizes schema failures from registry stderr", async () => {
  const secretField = "unrecognized-secret-field";
  const secretValue = "unrecognized-secret-value";
  const path = await createRegistry(
    JSON.stringify({
      [secretField]: secretValue,
      invariants: [],
      version: 1,
    }),
  );
  const outcome = await runRegistryCli(path);

  expect(outcome.exitCode).not.toBe(0);
  expect(outcome.stdout).toBe("");
  expect(outcome.stderr).toContain("invalid invariant registry");
  expect(outcome.stderr).not.toContain(secretField);
  expect(outcome.stderr).not.toContain(secretValue);
});

test.each([
  "../outside.json",
  "/private/tmp/outside.json",
  String.raw`C:\outside.json`,
] as const)("refuses an input path outside the repository", async (path) => {
  const outcome = await runRegistryCli(path);

  expect(outcome.exitCode).not.toBe(0);
  expect(outcome.stdout).toBe("");
  expect(outcome.stderr).toContain("within the repository");
});

test("validates the active PR 206 fixture", async () => {
  const outcome = await runRegistryCli(
    "tooling/invariant-registry-fixtures/pr-206-secret-redaction.json",
  );

  expect(outcome.exitCode).toBe(0);
  expect(outcome.stdout).toContain("Invariant registry passed");
  expect(outcome.stderr).toBe("");
});

test("rejects the PR 206 fixture when its active oracle is absent", async () => {
  const path = await mutatedFixture("pr-206-secret-redaction.json", (source) =>
    source.replace(
      "tooling/git-main-branch-entry.test.ts",
      "tooling/missing-oracle.test.ts",
    ),
  );
  const outcome = await runRegistryCli(path);

  expect(outcome.exitCode).not.toBe(0);
  expect(outcome.stdout).toBe("");
  expect(outcome.stderr).toContain("Oracle test path does not exist");
});

test("rejects the PR 206 fixture when its active oracle resolves outside", async () => {
  const target = await createExternalFile("oracle.test.ts", "");
  const oraclePath = await createLinkedOracle(target);
  const path = await mutatedFixture("pr-206-secret-redaction.json", (source) =>
    source.replace("tooling/git-main-branch-entry.test.ts", oraclePath),
  );
  const outcome = await runRegistryCli(path);

  expect(outcome.exitCode).not.toBe(0);
  expect(outcome.stdout).toBe("");
  expect(outcome.stderr).toContain("Oracle test path does not exist");
});

test("validates the retired PR 207 fixture", async () => {
  const outcome = await runRegistryCli(
    "tooling/invariant-registry-fixtures/pr-207-invalid-utf8.json",
  );

  expect(outcome.exitCode).toBe(0);
  expect(outcome.stdout).toContain("Invariant registry passed");
  expect(outcome.stderr).toBe("");
});

test("records factual retired PR 207 evidence", async () => {
  const fixture = await readFile(
    join(fixtureDirectory, "pr-207-invalid-utf8.json"),
    "utf8",
  );

  expect(fixture).toContain("reject-invalid-utf8-measurement-output");
  expect(fixture).toContain(
    "The historical repository measurement consumer rejects invalid UTF-8 output.",
  );
  expect(fixture).toContain(
    "https://github.com/SebastienElet/dotfiles/pull/207#issuecomment-5388145825",
  );
  expect(fixture).toContain(
    "The historical repository measurement consumer was retired.",
  );
  expect(fixture).not.toContain('"oracle"');
});

test("rejects the PR 207 fixture without a retirement reason", async () => {
  const path = await mutatedFixture("pr-207-invalid-utf8.json", (source) =>
    source.replace(
      "The historical repository measurement consumer was retired.",
      " ",
    ),
  );
  const outcome = await runRegistryCli(path);

  expect(outcome.exitCode).not.toBe(0);
  expect(outcome.stdout).toBe("");
  expect(outcome.stderr).toContain("invalid invariant registry");
});
