import { afterAll, expect, test } from "bun:test";
import { dirname, join, resolve } from "node:path";
import { mkdir, mkdtemp, rm } from "node:fs/promises";
import { homedir } from "node:os";
import { z } from "zod";

const repositoryRoot = resolve(import.meta.dir, "..");
const temporaryRoots: string[] = [];
const staticTargets = [
  "typescript:format-check",
  "typescript:lint",
  "typescript:typecheck",
] as const;
const staticTargetSet = new Set<string>(staticTargets);
const taskToolchainsSchema = z.union([z.string(), z.array(z.string())]);
const moonTaskSchema = z.looseObject({
  command: z.string(),
  inputs: z.array(z.string()).optional(),
  toolchains: taskToolchainsSchema.optional(),
});
const moonProjectSchema = z.looseObject({
  language: z.literal("unknown"),
  tasks: z.record(
    z.string(),
    moonTaskSchema.extend({ toolchains: z.literal("system") }),
  ),
});
const affectedProjectSchema = z.looseObject({
  tasks: z.array(z.string()).optional(),
});
const affectedSchema = z.looseObject({
  projects: z.record(z.string(), affectedProjectSchema).optional(),
  tasks: z.record(z.string(), z.unknown()).optional(),
});

type CommandResult = Readonly<{
  exitCode: number;
  stderr: string;
  stdout: string;
}>;
type AffectedQuery = Readonly<{
  base: string;
  head: string;
  moon: string;
  repository: string;
}>;
type ExpectChange = (
  message: string,
  expected: readonly string[],
) => Promise<void>;

async function run(
  command: readonly string[],
  cwd: string,
  environment: Readonly<Record<string, string>> = {},
): Promise<CommandResult> {
  const process = Bun.spawn([...command], {
    cwd,
    env: { ...Bun.env, ...environment },
    stderr: "pipe",
    stdout: "pipe",
  });
  const [exitCode, stderr, stdout] = await Promise.all([
    process.exited,
    new Response(process.stderr).text(),
    new Response(process.stdout).text(),
  ]);

  return { exitCode, stderr, stdout };
}

async function requireSuccess(
  command: readonly string[],
  cwd: string,
): Promise<string> {
  const result = await run(command, cwd);
  expect(result.exitCode).toBe(0);
  return result.stdout.trim();
}

async function copyFile(source: string, destination: string): Promise<void> {
  await mkdir(dirname(destination), { recursive: true });
  await Bun.write(destination, Bun.file(source));
}

async function createContractRepository(): Promise<string> {
  const temporaryRoot = await mkdtemp("/tmp/moon-typescript-");
  const repository = join(temporaryRoot, "repository");
  temporaryRoots.push(temporaryRoot);
  await requireSuccess(
    ["git", "clone", "--local", "--no-hardlinks", repositoryRoot, repository],
    temporaryRoot,
  );
  for (const path of [
    ".moon/workspace.yml",
    ".github/workflows/check-typescript.yml",
    "tooling/moon-typescript-gates.test.ts",
  ]) {
    expect(await Bun.file(join(repositoryRoot, path)).exists()).toBeTrue();
    await copyFile(join(repositoryRoot, path), join(repository, path));
  }
  const moonProjectPath = join(repositoryRoot, "moon.yml");
  expect(await Bun.file(moonProjectPath).exists()).toBeTrue();
  moonProjectSchema.parse(
    Bun.YAML.parse(await Bun.file(moonProjectPath).text()),
  );
  await copyFile(moonProjectPath, join(repository, "moon.yml"));
  await Bun.write(join(repository, ".contract-baseline"), "baseline\n");
  await requireSuccess(["git", "add", "-A"], repository);
  await requireSuccess(
    [
      "git",
      "-c",
      "user.name=Moon contract",
      "-c",
      "user.email=moon-contract@example.invalid",
      "commit",
      "-m",
      "contract baseline",
    ],
    repository,
  );
  return repository;
}

async function commitAll(repository: string, message: string): Promise<string> {
  await requireSuccess(["git", "add", "-A"], repository);
  await requireSuccess(
    [
      "git",
      "-c",
      "user.name=Moon contract",
      "-c",
      "user.email=moon-contract@example.invalid",
      "commit",
      "-m",
      message,
    ],
    repository,
  );
  return requireSuccess(["git", "rev-parse", "HEAD"], repository);
}

async function affectedTargets(query: AffectedQuery): Promise<string[]> {
  const { base, head, moon, repository } = query;
  const result = await run([moon, "query", "affected"], repository, {
    MOON_BASE: base,
    MOON_HEAD: head,
  });
  expect(result.exitCode).toBe(0);
  expect(result.stderr).toBe("");
  const affected = affectedSchema.parse(JSON.parse(result.stdout));
  const projectTargets: string[] = [];
  for (const project of Object.values(affected.projects ?? {})) {
    projectTargets.push(...(project.tasks ?? []));
  }
  const targets = [...Object.keys(affected.tasks ?? {}), ...projectTargets];
  return [...new Set(targets)]
    .filter((target) => staticTargetSet.has(target))
    .toSorted();
}

async function append(repository: string, path: string): Promise<void> {
  const file = Bun.file(join(repository, path));
  await Bun.write(join(repository, path), `${await file.text()}\n`);
}

async function exerciseTrackedFileLifecycle(
  repository: string,
  expectChange: ExpectChange,
): Promise<void> {
  await append(repository, "README.md");
  await expectChange("documentation", []);
  await Bun.write(
    join(repository, "outside.ts"),
    "export const outside = 1;\n",
  );
  await expectChange("add TypeScript outside tooling", [
    "typescript:format-check",
    "typescript:lint",
  ]);
  await requireSuccess(
    ["git", "mv", "outside.ts", "tooling/outside.ts"],
    repository,
  );
  await expectChange("move TypeScript into tooling", staticTargets);
  await rm(join(repository, "tooling/outside.ts"));
  await expectChange("delete tracked TypeScript", staticTargets);
}

async function exerciseOracleInputs(
  repository: string,
  expectChange: ExpectChange,
): Promise<void> {
  for (const [path, targets] of [
    [".oxlintrc.json", ["typescript:lint"]],
    ["tsconfig.json", ["typescript:lint", "typescript:typecheck"]],
    ["oxfmt.config.ts", ["typescript:format-check", "typescript:lint"]],
    ["package.json", staticTargets],
    ["bun.lock", staticTargets],
    [".moon/workspace.yml", staticTargets],
    ["moon.yml", staticTargets],
    [".github/workflows/check-typescript.yml", staticTargets],
    ["tooling/moon-typescript-gates.test.ts", staticTargets],
  ] as const) {
    await append(repository, path);
    await expectChange(`change ${path}`, targets);
  }
}

afterAll(async (): Promise<void> => {
  await Promise.all(
    temporaryRoots.map((temporaryRoot) =>
      rm(temporaryRoot, { force: true, recursive: true }),
    ),
  );
});

test.skipIf(Bun.env.MOON_TYPESCRIPT_CONTRACT !== "1")(
  "production Moon selects static TypeScript gates from tracked files and oracle inputs",
  async (): Promise<void> => {
    const moon = Bun.which("moon") ?? join(homedir(), ".moon/bin/moon");
    expect(await Bun.file(moon).exists()).toBeTrue();
    const repository = await createContractRepository();
    let base = await requireSuccess(["git", "rev-parse", "HEAD"], repository);
    expect(
      await affectedTargets({ base, head: base, moon, repository }),
    ).toEqual([]);

    const expectChange = async (
      message: string,
      expected: readonly string[],
    ): Promise<void> => {
      const head = await commitAll(repository, message);
      expect(await affectedTargets({ base, head, moon, repository })).toEqual(
        expected.toSorted(),
      );
      base = head;
    };
    await exerciseTrackedFileLifecycle(repository, expectChange);
    await exerciseOracleInputs(repository, expectChange);
  },
);
