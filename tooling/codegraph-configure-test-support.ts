import {
  chmodSync,
  mkdirSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { dirname, join } from "node:path";
import { tmpdir } from "node:os";

const root = dirname(import.meta.dir);
const entryPoint = join(import.meta.dir, "codegraph-configure");
const provider = join(import.meta.dir, "codegraph-configure-test-provider.ts");
const temporaryDirectories: string[] = [];
const executableFileMode = 0o755;

interface Fixture {
  claudeConfig: string;
  codegraphBinary: string;
  codexConfig: string;
  cursorConfig: string;
  directory: string;
  environment: Record<string, string> & { CODEGRAPH_TEST_LOG: string };
}

type FixtureView = Omit<Readonly<Fixture>, "environment"> & {
  readonly environment: Readonly<Fixture["environment"]>;
};

interface CommandResult {
  exitCode: number;
  stderr: string;
  stdout: string;
}
interface ConfigurationSnapshot {
  claude: string;
  codex: string;
  cursor: string;
}
interface SpawnOptions {
  cwd: string;
  env: NodeJS.ProcessEnv;
  stderr: "pipe";
  stdout: "pipe";
}
type FixtureValueOptions = Readonly<{
  binaries: string;
  directory: string;
  links: Readonly<Record<"claude" | "codegraph" | "codex", string>>;
  overrides: Readonly<Record<string, string>>;
  state: string;
}>;

function createFixture(
  overrides: Readonly<Record<string, string>> = {},
  registered = false,
): Fixture {
  const directory = join(
    tmpdir(),
    `codegraph-configure-${crypto.randomUUID()}`,
  );
  temporaryDirectories.push(directory);
  const { binaries, links, state } = createFixtureDirectories(directory);
  const fixture = createFixtureValues({
    binaries,
    directory,
    links,
    overrides,
    state,
  });
  writeFileSync(fixture.claudeConfig, '{"unrelated":"claude"}\n');
  writeFileSync(fixture.codexConfig, 'unrelated = "codex"\n');
  if (registered) {
    writeFileSync(join(state, "claude"), "registered\n");
    writeFileSync(join(state, "codex"), "registered\n");
  }
  return fixture;
}

function createFixtureDirectories(directory: string): Readonly<{
  binaries: string;
  links: Readonly<Record<"claude" | "codegraph" | "codex", string>>;
  state: string;
}> {
  const binaries = join(directory, "bin");
  const state = join(directory, "state");
  mkdirSync(binaries, { recursive: true });
  mkdirSync(state, { recursive: true });
  mkdirSync(join(directory, "cursor"), { recursive: true });
  symlinkSync(process.execPath, join(binaries, "bun"));
  const linkProvider = (name: string): string => {
    const path = join(binaries, name);
    symlinkSync(provider, path);
    return path;
  };
  const links = {
    claude: linkProvider("claude"),
    codegraph: linkProvider("codegraph"),
    codex: linkProvider("codex"),
  };
  chmodSync(provider, executableFileMode);
  return { binaries, links, state };
}

function createFixtureValues({
  binaries,
  directory,
  links,
  overrides,
  state,
}: FixtureValueOptions): Fixture {
  return {
    claudeConfig: join(directory, "claude.json"),
    codegraphBinary: links.codegraph,
    codexConfig: join(directory, "codex.toml"),
    cursorConfig: join(directory, "cursor", "mcp.json"),
    directory,
    environment: {
      CODEGRAPH_BIN: links.codegraph,
      CODEGRAPH_CLAUDE_BIN: links.claude,
      CODEGRAPH_CLAUDE_CONFIG: join(directory, "claude.json"),
      CODEGRAPH_CODEX_BIN: links.codex,
      CODEGRAPH_CODEX_CONFIG: join(directory, "codex.toml"),
      CODEGRAPH_CURSOR_CONFIG: join(directory, "cursor", "mcp.json"),
      CODEGRAPH_INCLUDE_CURSOR: "1",
      CODEGRAPH_TEST_LOG: join(directory, "calls.log"),
      CODEGRAPH_TEST_STATE: state,
      CODEX_HOME: join(directory, "codex-home"),
      HOME: join(directory, "home"),
      PATH: `${binaries}:${process.env.PATH ?? ""}`,
      ...overrides,
    },
  };
}

function cleanupFixtures(): void {
  for (const path of temporaryDirectories.splice(0)) {
    rmSync(path, { force: true, recursive: true });
  }
}

function run(fixture: FixtureView): CommandResult {
  const result = Bun.spawnSync([entryPoint], spawnOptions(fixture));
  return {
    exitCode: result.exitCode,
    stderr: result.stderr.toString(),
    stdout: result.stdout.toString(),
  };
}

function start(fixture: FixtureView): ReturnType<typeof Bun.spawn> {
  return Bun.spawn([entryPoint], spawnOptions(fixture));
}

function spawnOptions(fixture: FixtureView): SpawnOptions {
  return {
    cwd: root,
    env: { ...process.env, ...fixture.environment },
    stderr: "pipe" as const,
    stdout: "pipe" as const,
  };
}

function snapshot(fixture: FixtureView): ConfigurationSnapshot {
  return {
    claude: readOrAbsent(fixture.claudeConfig),
    codex: readOrAbsent(fixture.codexConfig),
    cursor: readOrAbsent(fixture.cursorConfig),
  };
}

function readLog(fixture: FixtureView): string {
  return readOrAbsent(fixture.environment.CODEGRAPH_TEST_LOG).replace(
    "<absent>",
    "",
  );
}

function readOrAbsent(path: string): string {
  try {
    return readFileSync(path, "utf8");
  } catch {
    return "<absent>";
  }
}

export {
  cleanupFixtures,
  createFixture,
  type Fixture,
  readLog,
  run,
  snapshot,
  start,
};
