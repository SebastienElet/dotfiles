import {
  chmodSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { z } from "zod";

const entryPoint = join(import.meta.dir, "scrapling-mcp");
const executableFileMode = 0o755;
const provider = join(import.meta.dir, "scrapling-mcp-test-provider.ts");
const fixtures: Fixture[] = [];

type Scenario = Readonly<{
  present?: boolean;
  running?: boolean;
  compatible?: boolean;
  infoFailure?: boolean;
  listFailure?: boolean;
  inspectFailure?: boolean;
  startFailure?: boolean;
  runFailure?: boolean;
  concurrent?: boolean;
  execExit?: number;
  execStdout?: string;
  execStderr?: string;
  invalidInspect?: boolean;
  invalidUtf8?: string;
  hang?: string;
}>;

type RunResult = Readonly<{
  exitCode: number;
  stderr: string;
  stdout: string;
}>;

const callsSchema = z.array(z.array(z.string()));

type Fixture = Readonly<{
  root: string;
  entryPoint: string;
  environment: Readonly<NodeJS.ProcessEnv>;
  state: string;
}>;

function createFixture(
  scenario: Scenario = {},
  environment: Readonly<Record<string, string>> = {},
): Fixture {
  const root = mkdtempSync(join(tmpdir(), "scrapling-mcp-"));
  const binaries = join(root, "bin");
  const state = join(root, "state");
  mkdirSync(binaries);
  mkdirSync(state);
  symlinkSync(process.execPath, join(binaries, "bun"));
  symlinkSync(provider, join(binaries, "docker"));
  const installedEntryPoint = join(binaries, "scrapling_mcp");
  symlinkSync(entryPoint, installedEntryPoint);
  chmodSync(provider, executableFileMode);
  writeFileSync(join(state, "scenario.json"), JSON.stringify(scenario));
  if (scenario.present === true) {
    mkdirSync(join(state, "container"));
  }
  if (scenario.running === true) {
    writeFileSync(join(state, "running"), "");
  }
  const fixture = {
    entryPoint: installedEntryPoint,
    environment: {
      ...process.env,
      PATH: `${binaries}:${process.env.PATH ?? ""}`,
      SCRAPLING_DOCKER_TIMEOUT_MS: "5000",
      SCRAPLING_TEST_STATE: state,
      ...environment,
    },
    root,
    state,
  };
  fixtures.push(fixture);
  return fixture;
}

function run(fixture: Fixture): RunResult {
  const commandResult = Bun.spawnSync([fixture.entryPoint], {
    env: fixture.environment,
    stderr: "pipe",
    stdin: "ignore",
    stdout: "pipe",
  });
  return {
    exitCode: commandResult.exitCode,
    stderr: commandResult.stderr.toString(),
    stdout: commandResult.stdout.toString(),
  };
}

function start(fixture: Fixture): Bun.Subprocess<"ignore", "pipe", "pipe"> {
  return Bun.spawn([fixture.entryPoint], {
    env: fixture.environment,
    stderr: "pipe",
    stdin: "ignore",
    stdout: "pipe",
  });
}

async function result(fixture: Fixture): Promise<RunResult> {
  const process = start(fixture);
  const [exitCode, stdout, stderr] = await Promise.all([
    process.exited,
    new Response(process.stdout).text(),
    new Response(process.stderr).text(),
  ]);
  return { exitCode, stderr, stdout };
}

function calls(fixture: Fixture): readonly (readonly string[])[] {
  try {
    const parsedCalls = readFileSync(join(fixture.state, "calls"), "utf8")
      .trim()
      .split("\n")
      .filter(Boolean)
      .map((line): unknown => JSON.parse(line));
    return callsSchema.parse(parsedCalls);
  } catch {
    return [];
  }
}

function cleanupFixtures(): void {
  for (const fixture of fixtures.splice(0)) {
    rmSync(fixture.root, { force: true, recursive: true });
  }
}

export { calls, cleanupFixtures, createFixture, result, run };
export type { Fixture, Scenario };
