import { afterEach, expect, test } from "bun:test";
import { dirname, join, resolve } from "node:path";
import {
  findTrackedTypeScriptPaths,
  lintTrackedTypeScript,
} from "./lint-typescript.ts";
import { link, mkdir, mkdtemp, rm, symlink, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";

const FAILURE = 1;
const EXPECTED_MASKED_DIAGNOSTICS = 2;
const SUCCESS = 0;
const oxlint = resolve(import.meta.dir, "../node_modules/.bin/oxlint");
const repositoryRoot = resolve(import.meta.dir, "..");
const mutatingLinter = resolve(
  import.meta.dir,
  "lint-typescript-mutating-test-provider.ts",
);
const temporaryDirectories: string[] = [];

type LinkFile = (existingPath: string, newPath: string) => Promise<void>;

afterEach(async (): Promise<void> => {
  await Promise.all(
    temporaryDirectories.splice(SUCCESS).map(async (directory) => {
      await rm(directory, { force: true, recursive: true });
    }),
  );
});

async function run(command: readonly string[], cwd: string): Promise<number> {
  const child = Bun.spawn([...command], {
    cwd,
    stderr: "pipe",
    stdout: "pipe",
  });
  const status = await child.exited;
  return status;
}

async function createRepository(): Promise<string> {
  const repository = await mkdtemp(join(tmpdir(), "oxlint-contract-"));
  temporaryDirectories.push(repository);
  expect(await run(["git", "init", "-q"], repository)).toBe(SUCCESS);
  const sourceConfig = join(repositoryRoot, ".oxlintrc.json");
  await Promise.all([
    symlink(
      join(repositoryRoot, "node_modules"),
      join(repository, "node_modules"),
      "dir",
    ),
    Bun.write(join(repository, ".oxlintrc.json"), Bun.file(sourceConfig)),
    writeFile(
      join(repository, "tsconfig.json"),
      JSON.stringify({
        compilerOptions: { lib: ["ESNext"], strict: true },
        include: ["**/*.ts", "**/*.tsx", "**/*.mts", "**/*.cts"],
      }),
    ),
  ]);
  return repository;
}

async function rejectionMessage(
  promise: Readonly<Promise<unknown>>,
): Promise<string> {
  try {
    await promise;
    return "Promise resolved unexpectedly.";
  } catch (error) {
    return error instanceof Error ? error.message : "Unknown rejection.";
  }
}

async function rejectionForTrackedLink(createLink: LinkFile): Promise<string> {
  const repository = await createRepository();
  const externalDirectory = await mkdtemp(join(tmpdir(), "oxlint-external-"));
  temporaryDirectories.push(externalDirectory);
  const externalSource = join(externalDirectory, "external.ts");
  const linkedSource = join(repository, "linked.ts");
  await writeFile(externalSource, 'export const value = "external";\n');
  await createLink(externalSource, linkedSource);
  expect(await run(["git", "add", "linked.ts"], repository)).toBe(SUCCESS);
  return rejectionMessage(lintTrackedTypeScript(repository, oxlint));
}

function assertLintSuccess(
  result: Readonly<{ status: number; stderr: string; stdout: string }>,
): void {
  if (result.status !== SUCCESS) {
    throw new Error(`${result.stdout}\n${result.stderr}`);
  }
}

test("discovers every tracked TypeScript extension", async (): Promise<void> => {
  const repository = await createRepository();
  const trackedFiles = [
    "-leading.ts",
    ".hidden/future.tsx",
    "ignored/forced.mts",
    "types/future file.cts",
    "types/future.d.ts",
  ];
  await Promise.all(
    [...trackedFiles, "untracked.ts"].map(async (file) => {
      const target = join(repository, file);
      await mkdir(dirname(target), { recursive: true });
      await writeFile(target, 'export const value = "valid";\n');
    }),
  );
  await writeFile(join(repository, ".gitignore"), "ignored/\n");
  expect(
    await run(
      [
        "git",
        "add",
        "--",
        ".gitignore",
        ...trackedFiles.filter((file) => !file.startsWith("ignored/")),
      ],
      repository,
    ),
  ).toBe(SUCCESS);
  expect(
    await run(["git", "add", "-f", "--", "ignored/forced.mts"], repository),
  ).toBe(SUCCESS);

  expect(await findTrackedTypeScriptPaths(repository)).toEqual(
    trackedFiles.toSorted(),
  );
});

test("lints surviving tracked files while a tracked TypeScript file is deleted", async (): Promise<void> => {
  const repository = await createRepository();
  await writeFile(
    join(repository, "kept.ts"),
    'export const kept = "valid";\n',
  );
  await writeFile(
    join(repository, "removed.ts"),
    'export const removed = "valid";\n',
  );
  expect(await run(["git", "add", "kept.ts", "removed.ts"], repository)).toBe(
    SUCCESS,
  );
  await rm(join(repository, "removed.ts"));

  expect(await findTrackedTypeScriptPaths(repository)).toEqual(["kept.ts"]);
  assertLintSuccess(await lintTrackedTypeScript(repository, oxlint));
});

test("fails closed when tracked TypeScript discovery is empty or unavailable", async (): Promise<void> => {
  const emptyRepository = await createRepository();
  const nonRepository = await mkdtemp(join(tmpdir(), "oxlint-no-git-"));
  temporaryDirectories.push(nonRepository);

  const emptyError = await rejectionMessage(
    findTrackedTypeScriptPaths(emptyRepository),
  );
  const unavailableError = await rejectionMessage(
    findTrackedTypeScriptPaths(nonRepository),
  );
  expect(emptyError).toContain("empty or malformed");
  expect(unavailableError).toContain("Git could not list");
});

test("rejects syntax-only and type-aware defects", async (): Promise<void> => {
  const repository = await createRepository();
  const source = join(repository, "source.ts");
  await writeFile(source, 'export const value = "valid";\n');
  expect(await run(["git", "add", "source.ts"], repository)).toBe(SUCCESS);
  const validResult = await lintTrackedTypeScript(repository, oxlint);
  assertLintSuccess(validResult);

  await writeFile(source, "export const = FAILURE;\n");
  const syntaxFailure = await lintTrackedTypeScript(repository, oxlint);
  expect(syntaxFailure.status).toBe(FAILURE);

  await writeFile(
    source,
    'Promise.resolve();\nexport const value = "invalid";\n',
  );
  const typeAwareFailure = await lintTrackedTypeScript(repository, oxlint);
  const diagnostics = `${typeAwareFailure.stdout}\n${typeAwareFailure.stderr}`;
  expect(typeAwareFailure.status).toBe(FAILURE);
  expect(diagnostics).toContain("typescript(no-floating-promises)");
});

test("does not let ignore files or nested configurations mask defects", async (): Promise<void> => {
  const repository = await createRepository();
  const ignoredSource = join(repository, "ignored.ts");
  const nestedDirectory = join(repository, "nested");
  const nestedSource = join(nestedDirectory, "source.ts");
  await mkdir(nestedDirectory);
  await Promise.all([
    writeFile(join(repository, ".eslintignore"), "ignored.ts\n"),
    writeFile(ignoredSource, "Promise.resolve();\n"),
    writeFile(
      join(nestedDirectory, ".oxlintrc.json"),
      JSON.stringify({ categories: { correctness: "off" } }),
    ),
    writeFile(nestedSource, "Promise.resolve();\n"),
  ]);
  expect(
    await run(
      [
        "git",
        "add",
        ".eslintignore",
        "ignored.ts",
        "nested/.oxlintrc.json",
        "nested/source.ts",
      ],
      repository,
    ),
  ).toBe(SUCCESS);

  const result = await lintTrackedTypeScript(repository, oxlint);
  const diagnostics = `${result.stdout}\n${result.stderr}`;
  expect(result.status).toBe(FAILURE);
  expect(
    diagnostics.match(/typescript\(no-floating-promises\)/gu),
  ).toHaveLength(EXPECTED_MASKED_DIAGNOSTICS);
});

test("rejects inline lint suppression directives", async (): Promise<void> => {
  const repository = await createRepository();
  const source = join(repository, "source.ts");
  const directive = [
    "/* oxlint",
    "-disable typescript/no-floating-promises promise/catch-or-return */",
  ].join("");
  await writeFile(source, `${directive}\nPromise.resolve();\n`);
  expect(await run(["git", "add", "source.ts"], repository)).toBe(SUCCESS);

  const message = await rejectionMessage(
    lintTrackedTypeScript(repository, oxlint),
  );
  expect(message).toContain("lint suppression directive");
});

test("refuses tracked TypeScript files not owned by the repository", async (): Promise<void> => {
  const [symbolicLinkMessage, hardLinkMessage] = await Promise.all([
    rejectionForTrackedLink(symlink),
    rejectionForTrackedLink(link),
  ]);
  expect(symbolicLinkMessage).toContain("not confined to the repository");
  expect(hardLinkMessage).toContain("not an owned regular file");
});

test("fails when a tracked source changes while lint runs", async (): Promise<void> => {
  const repository = await createRepository();
  const source = join(repository, "source.ts");
  await writeFile(source, 'export const value = "valid";\n');
  expect(await run(["git", "add", "source.ts"], repository)).toBe(SUCCESS);

  const message = await rejectionMessage(
    lintTrackedTypeScript(repository, process.execPath, [
      process.execPath,
      mutatingLinter,
    ]),
  );
  expect(message).toContain("contains a lint suppression directive");
});
