import {
  chmodSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const entryPoint = join(import.meta.dir, "scrapling-mcp");
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

export type Fixture = Readonly<{
  root: string;
  entryPoint: string;
  environment: NodeJS.ProcessEnv;
  state: string;
}>;

export function createFixture(
  scenario: Scenario = {},
  environment: Record<string, string> = {},
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
  chmodSync(provider, 0o755);
  writeFileSync(join(state, "scenario.json"), JSON.stringify(scenario));
  if (scenario.present) mkdirSync(join(state, "container"));
  if (scenario.running) writeFileSync(join(state, "running"), "");
  const fixture = {
    root,
    entryPoint: installedEntryPoint,
    state,
    environment: {
      ...process.env,
      PATH: `${binaries}:${process.env.PATH ?? ""}`,
      SCRAPLING_TEST_STATE: state,
      SCRAPLING_DOCKER_TIMEOUT_MS: "5000",
      ...environment,
    },
  };
  fixtures.push(fixture);
  return fixture;
}

export function run(fixture: Fixture) {
  const result = Bun.spawnSync([fixture.entryPoint], {
    env: fixture.environment,
    stdin: "ignore",
    stdout: "pipe",
    stderr: "pipe",
  });
  return {
    exitCode: result.exitCode,
    stdout: result.stdout.toString(),
    stderr: result.stderr.toString(),
  };
}

export function start(fixture: Fixture) {
  return Bun.spawn([fixture.entryPoint], {
    env: fixture.environment,
    stdin: "ignore",
    stdout: "pipe",
    stderr: "pipe",
  });
}

export async function result(process: ReturnType<typeof start>) {
  const [exitCode, stdout, stderr] = await Promise.all([
    process.exited,
    new Response(process.stdout).text(),
    new Response(process.stderr).text(),
  ]);
  return { exitCode, stdout, stderr };
}

export function calls(fixture: Fixture): string[][] {
  try {
    return readFileSync(join(fixture.state, "calls"), "utf8")
      .trim()
      .split("\n")
      .filter(Boolean)
      .map((line) => JSON.parse(line) as string[]);
  } catch {
    return [];
  }
}

export function cleanupFixtures(): void {
  for (const fixture of fixtures.splice(0)) {
    rmSync(fixture.root, { recursive: true, force: true });
  }
}
