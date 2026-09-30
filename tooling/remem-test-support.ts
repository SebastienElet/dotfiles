import {
  chmodSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { tmpdir } from "node:os";
import { z } from "zod";

const repository = fileURLToPath(new URL("..", import.meta.url));
const fakeCommand = join(repository, "tooling", "remem-fake-command.ts");
const traceSchema = z
  .object({ command: z.string(), arguments: z.array(z.string()).readonly() })
  .readonly();
const executableMode = 0o755;
const fixtures: string[] = [];

type RememFixture = Readonly<{
  home: string;
  binaries: string;
  trace: string;
}>;
type UtilityResult = Readonly<{
  exitCode: number;
  stdout: string;
  stderr: string;
}>;

function installFake(path: string): void {
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(
    path,
    `#!/usr/bin/env bun\nimport ${JSON.stringify(fakeCommand)};\n`,
  );
  chmodSync(path, executableMode);
}

function createRememFixture(): RememFixture {
  const home = mkdtempSync(join(tmpdir(), "remem-install-test-"));
  fixtures.push(home);
  const binaries = join(home, "path");
  installFake(join(binaries, "claude"));
  installFake(join(binaries, "launchctl"));
  installFake(join(home, ".volta", "bin", "codex"));
  installFake(join(home, ".local", "bin", "remem"));
  return { home, binaries, trace: join(home, "trace.jsonl") };
}

function runUtility(
  fixture: RememFixture,
  utilityCommand: readonly string[],
  environment: Readonly<Record<string, string>> = {},
): UtilityResult {
  const [utility = "", ...utilityArguments] = utilityCommand;
  const result = Bun.spawnSync(
    [
      process.execPath,
      "--config=/dev/null",
      "--no-env-file",
      join(repository, "tooling", utility),
      ...utilityArguments,
    ],
    {
      env: {
        HOME: fixture.home,
        PATH: `${fixture.binaries}:${dirname(process.execPath)}:/usr/bin:/bin`,
        FAKE_TRACE: fixture.trace,
        ...environment,
      },
      stdout: "pipe",
      stderr: "pipe",
    },
  );
  return {
    exitCode: result.exitCode,
    stdout: result.stdout.toString(),
    stderr: result.stderr.toString(),
  };
}

function readTrace(
  fixture: RememFixture,
): readonly z.infer<typeof traceSchema>[] {
  if (!existsSync(fixture.trace)) {
    return [];
  }
  return readFileSync(fixture.trace, "utf8")
    .split("\n")
    .filter((line) => line !== "")
    .map((line) => traceSchema.parse(JSON.parse(line)));
}

function cleanupRememFixtures(): void {
  for (const fixture of fixtures.splice(0)) {
    rmSync(fixture, { recursive: true, force: true });
  }
}

export {
  cleanupRememFixtures,
  createRememFixture,
  readTrace,
  repository,
  runUtility,
};
export type { RememFixture };
