import {
  chmodSync,
  mkdirSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";

const root = dirname(import.meta.dir);
const entryPoint = join(import.meta.dir, "codegraph-configure");
const provider = join(import.meta.dir, "codegraph-configure-test-provider.ts");
const temporaryDirectories: string[] = [];

export type Fixture = ReturnType<typeof createFixture>;

export function createFixture(
  overrides: Record<string, string> = {},
  registered = false,
) {
  const directory = join(
    tmpdir(),
    `codegraph-configure-${crypto.randomUUID()}`,
  );
  temporaryDirectories.push(directory);
  const binaries = join(directory, "bin");
  const state = join(directory, "state");
  mkdirSync(binaries, { recursive: true });
  mkdirSync(state, { recursive: true });
  mkdirSync(join(directory, "cursor"), { recursive: true });
  symlinkSync(process.execPath, join(binaries, "bun"));
  const links = Object.fromEntries(
    ["claude", "codex", "codegraph"].map((name) => {
      const path = join(binaries, name);
      symlinkSync(provider, path);
      return [name, path];
    }),
  ) as Record<"claude" | "codex" | "codegraph", string>;
  chmodSync(provider, 0o755);
  const fixture = {
    directory,
    claudeConfig: join(directory, "claude.json"),
    codexConfig: join(directory, "codex.toml"),
    cursorConfig: join(directory, "cursor", "mcp.json"),
    codegraphBinary: links.codegraph,
    environment: {
      HOME: join(directory, "home"),
      CODEX_HOME: join(directory, "codex-home"),
      CODEGRAPH_CLAUDE_BIN: links.claude,
      CODEGRAPH_CODEX_BIN: links.codex,
      CODEGRAPH_BIN: links.codegraph,
      CODEGRAPH_CLAUDE_CONFIG: join(directory, "claude.json"),
      CODEGRAPH_CODEX_CONFIG: join(directory, "codex.toml"),
      CODEGRAPH_CURSOR_CONFIG: join(directory, "cursor", "mcp.json"),
      CODEGRAPH_TEST_LOG: join(directory, "calls.log"),
      CODEGRAPH_TEST_STATE: state,
      PATH: `${binaries}:${process.env.PATH ?? ""}`,
      ...overrides,
    },
  };
  writeFileSync(fixture.claudeConfig, '{"unrelated":"claude"}\n');
  writeFileSync(fixture.codexConfig, 'unrelated = "codex"\n');
  if (registered) {
    writeFileSync(join(state, "claude"), "registered\n");
    writeFileSync(join(state, "codex"), "registered\n");
  }
  return fixture;
}

export function cleanupFixtures(): void {
  for (const path of temporaryDirectories.splice(0)) {
    rmSync(path, { force: true, recursive: true });
  }
}

export function run(fixture: Fixture) {
  const result = Bun.spawnSync([entryPoint], {
    cwd: root,
    env: { ...process.env, ...fixture.environment },
    stdout: "pipe",
    stderr: "pipe",
  });
  return {
    exitCode: result.exitCode,
    stderr: result.stderr.toString(),
  };
}

export function snapshot(fixture: Fixture) {
  return {
    claude: readOrAbsent(fixture.claudeConfig),
    codex: readOrAbsent(fixture.codexConfig),
    cursor: readOrAbsent(fixture.cursorConfig),
  };
}

export function readLog(fixture: Fixture): string {
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
