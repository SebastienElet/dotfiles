import { afterEach, expect, test } from "bun:test";
import {
  chmod,
  copyFile,
  mkdir,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from "node:fs/promises";
import { delimiter, dirname, join, resolve } from "node:path";
import { tmpdir } from "node:os";

const entrypoint = resolve(import.meta.dir, "cspell-texts.ts");
const temporaryDirectories: string[] = [];
const executableMode = 0o700;
const lintArgumentPrefixLength = 3;
const lintFailureExitCode = 23;

type GateOutcome = Readonly<{
  exitCode: number;
  stderr: string;
  stdout: string;
}>;

const temporaryDirectory = async (): Promise<string> => {
  const directory = await mkdtemp(join(tmpdir(), "cspell-texts-"));
  temporaryDirectories.push(directory);
  return directory;
};

const fakeCspell = async (
  directory: string,
  exitCode: number,
): Promise<string> => {
  const path = join(directory, "cspell");
  await writeFile(
    path,
    `#!/usr/bin/env bun
const record = process.env.CSPELL_TEST_RECORD;
if (record === undefined) process.exit(97);
await Bun.write(record, JSON.stringify(Bun.argv.slice(2)));
process.exit(${exitCode});
`,
    "utf8",
  );
  await chmod(path, executableMode);
  return path;
};

const runGate = async (
  path: string,
  recordPath: string,
  scriptPath: string = entrypoint,
): Promise<GateOutcome> => {
  const child = Bun.spawn(
    [process.execPath, scriptPath, "fixture-config.json"],
    {
      env: {
        ...process.env,
        CSPELL_TEST_RECORD: recordPath,
        PATH: `${dirname(process.execPath)}${delimiter}/usr/bin${delimiter}/bin${delimiter}${path}`,
      },
      stderr: "pipe",
      stdout: "pipe",
    },
  );
  const [exitCode, stdout, stderr] = await Promise.all([
    child.exited,
    new Response(child.stdout).text(),
    new Response(child.stderr).text(),
  ]);
  return { exitCode, stderr, stdout };
};

afterEach(async () => {
  await Promise.all(
    temporaryDirectories
      .splice(0)
      .map((directory) => rm(directory, { force: true, recursive: true })),
  );
});

test("runs the real CSpell command over the owned text surface", async () => {
  const directory = await temporaryDirectory();
  await fakeCspell(directory, 0);
  const recordPath = join(directory, "argv.json");

  const outcome = await runGate(directory, recordPath);
  const argv: unknown = JSON.parse(await readFile(recordPath, "utf8"));

  expect(outcome).toEqual({ exitCode: 0, stderr: "", stdout: "" });
  expect(argv).toBeArray();
  expect(argv).toContain("harness/skills/harness-reflection/SKILL.md");
  expect(argv).toContain("harness/invariants/registry.json");
  expect(argv).toContain(
    "docs/superpowers/specs/2026-09-02-registre-invariants-harnais-design.md",
  );
  expect(argv).toContain(
    ".superpowers/sdd/2026-09-02-registre-invariants-harnais/breaker-adjudication-report.md",
  );
  expect(
    Array.isArray(argv) ? argv.slice(0, lintArgumentPrefixLength) : [],
  ).toEqual(["lint", "--config", "fixture-config.json"]);
});

test("propagates a CSpell lint failure", async () => {
  const directory = await temporaryDirectory();
  await fakeCspell(directory, lintFailureExitCode);

  const outcome = await runGate(directory, join(directory, "argv.json"));

  expect(outcome.exitCode).toBe(lintFailureExitCode);
});

test("fails closed when CSpell cannot be executed", async () => {
  const directory = await temporaryDirectory();

  const outcome = await runGate(directory, join(directory, "argv.json"));

  expect(outcome.exitCode).not.toBe(0);
  expect(outcome.stderr).toContain("unable to execute CSpell");
});

test("runs from a clean checkout without project packages installed", async () => {
  const directory = await temporaryDirectory();
  const binDirectory = join(directory, "bin");
  await mkdir(binDirectory);
  await fakeCspell(binDirectory, 0);
  const copiedEntrypoint = join(directory, "cspell-texts.ts");
  await copyFile(entrypoint, copiedEntrypoint);

  const outcome = await runGate(
    binDirectory,
    join(directory, "argv.json"),
    copiedEntrypoint,
  );

  expect(outcome).toEqual({ exitCode: 0, stderr: "", stdout: "" });
});
