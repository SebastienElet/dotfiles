import { afterEach, describe, expect, test } from "bun:test";
import {
  existsSync,
  mkdirSync,
  readFileSync,
  readlinkSync,
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

afterEach(() => {
  for (const path of temporaryDirectories.splice(0)) {
    rmSync(path, { force: true, recursive: true });
  }
});

describe("CodeGraph deployment", () => {
  test("the supported Make entry point deploys the shipped configurator", () => {
    const directory = createTemporaryDirectory();
    const result = makeDryRun(directory, "codegraph");

    expect(result.exitCode).toBe(0);
    expect(result.stdout).toContain("tooling/codegraph-configure");
    expect(result.stdout).toContain("CODEGRAPH_CLAUDE_BIN=");
    expect(result.stdout).toContain("CODEGRAPH_CODEX_BIN=");
    expect(result.stdout).toContain("CODEGRAPH_BIN=");
    expect(result.stdout).toContain(
      `${directory}/brew/bin/bun tooling/codegraph-configure`,
    );
    expect(result.stdout).toContain("brew install bun");
  });

  test("CodeGraph installation remains delegated to unpinned Volta commands", () => {
    const directory = createTemporaryDirectory();
    const codex = makeDryRun(
      directory,
      join(directory, ".volta", "bin", "codex"),
    );
    const codegraph = makeDryRun(directory, "codegraph-cli");

    expect(codex.exitCode).toBe(0);
    expect(codex.stdout).toContain("volta install @openai/codex");
    expect(codex.stdout).not.toContain("npm install -g @openai/codex");
    expect(codegraph.exitCode).toBe(0);
    expect(codegraph.stdout).toContain("volta install @colbymchenry/codegraph");
    expect(codegraph.stdout).not.toContain("@colbymchenry/codegraph@");
  });

  test("CodeGraph ignore deployment is idempotent and refuses a foreign target", () => {
    const directory = createTemporaryDirectory();
    const ignorePath = join(directory, "git", "ignore");
    expect(
      runMake(["codegraph-ignore", `CODEGRAPH_GLOBAL_IGNORE=${ignorePath}`])
        .exitCode,
    ).toBe(0);
    expect(readlinkSync(ignorePath)).toBe(
      join(root, ".config", "git", "ignore"),
    );
    expect(
      runMake(["codegraph-ignore", `CODEGRAPH_GLOBAL_IGNORE=${ignorePath}`])
        .exitCode,
    ).toBe(0);

    rmSync(ignorePath);
    writeFileSync(ignorePath, "foreign\n");
    const result = runMake([
      "codegraph-ignore",
      `CODEGRAPH_GLOBAL_IGNORE=${ignorePath}`,
    ]);

    expect(result.exitCode).toBe(2);
    expect(result.stderr).toContain(
      "exists and is not the expected symbolic link",
    );
  });
});

const realClaude = process.env.CODEGRAPH_REAL_CLAUDE_BIN;
const realCodex = process.env.CODEGRAPH_REAL_CODEX_BIN;
const realVoltaHome =
  process.env.VOLTA_HOME ??
  (realCodex === undefined ? undefined : dirname(dirname(realCodex)));

test.skipIf(
  realClaude === undefined ||
    realCodex === undefined ||
    realVoltaHome === undefined,
)(
  "the real Claude and Codex CLIs preserve unrelated configuration",
  () => {
    const directory = createTemporaryDirectory();
    const home = join(directory, "home");
    const codexHome = join(directory, "codex-home");
    const binaries = join(directory, "bin");
    const state = join(directory, "state");
    const claudeConfig = join(home, ".claude.json");
    const codexConfig = join(codexHome, "config.toml");
    const cursorConfig = join(directory, "cursor", "mcp.json");
    const log = join(directory, "calls.log");
    mkdirSync(home, { recursive: true });
    mkdirSync(codexHome, { recursive: true });
    mkdirSync(binaries, { recursive: true });
    symlinkSync(provider, join(binaries, "codegraph"));
    symlinkSync(process.execPath, join(binaries, "bun"));
    writeFileSync(claudeConfig, '{"unrelated":"claude"}\n');
    writeFileSync(codexConfig, 'unrelated = "codex"\n');

    const environment = {
      ...process.env,
      HOME: home,
      VOLTA_HOME: realVoltaHome,
      CODEX_HOME: codexHome,
      CODEGRAPH_CLAUDE_BIN: realClaude,
      CODEGRAPH_CODEX_BIN: realCodex,
      CODEGRAPH_BIN: join(binaries, "codegraph"),
      CODEGRAPH_CLAUDE_CONFIG: claudeConfig,
      CODEGRAPH_CODEX_CONFIG: codexConfig,
      CODEGRAPH_CURSOR_CONFIG: cursorConfig,
      CODEGRAPH_TEST_LOG: log,
      CODEGRAPH_TEST_STATE: state,
      PATH: `${binaries}:${process.env.PATH ?? ""}`,
    };

    const first = runEntryPoint(environment);
    expect(first).toEqual({ exitCode: 0, stdout: "", stderr: "" });
    const firstConfigurations = readConfigurations(
      claudeConfig,
      codexConfig,
      cursorConfig,
    );
    expect(runEntryPoint(environment).exitCode).toBe(0);
    expect(readConfigurations(claudeConfig, codexConfig, cursorConfig)).toEqual(
      firstConfigurations,
    );

    const claude = JSON.parse(firstConfigurations.claude);
    const cursor = JSON.parse(firstConfigurations.cursor);
    expect(claude.unrelated).toBe("claude");
    expect(claude.mcpServers.codegraph.command).toBe(
      join(binaries, "codegraph"),
    );
    expect(firstConfigurations.codex).toContain('unrelated = "codex"');
    expect(firstConfigurations.codex).toContain("[mcp_servers.codegraph]");
    expect(cursor.mcpServers.codegraph.command).toBe(
      join(binaries, "codegraph"),
    );
    expect(existsSync(log)).toBe(true);
  },
  15_000,
);

function createTemporaryDirectory(): string {
  const directory = join(
    tmpdir(),
    `codegraph-deployment-${crypto.randomUUID()}`,
  );
  temporaryDirectories.push(directory);
  mkdirSync(directory, { recursive: true });
  return directory;
}

function makeDryRun(home: string, target: string) {
  return runMake([
    "-Bn",
    `HOME=${home}`,
    `BREW_BIN=${join(home, "brew", "bin")}`,
    `VOLTA_BIN=${join(home, ".volta", "bin")}`,
    `LOCAL_BIN=${join(home, ".local", "bin")}`,
    target,
  ]);
}

function runMake(arguments_: string[]) {
  return run(["make", "-s", "-C", root, ...arguments_], process.env);
}

function runEntryPoint(environment: Record<string, string | undefined>) {
  return run([entryPoint], environment);
}

function run(
  command: string[],
  environment: Record<string, string | undefined>,
) {
  const result = Bun.spawnSync(command, {
    env: environment,
    stdout: "pipe",
    stderr: "pipe",
  });
  return {
    exitCode: result.exitCode,
    stdout: result.stdout.toString(),
    stderr: result.stderr.toString(),
  };
}

function readConfigurations(claude: string, codex: string, cursor: string) {
  return {
    claude: readFileSync(claude, "utf8"),
    codex: readFileSync(codex, "utf8"),
    cursor: readFileSync(cursor, "utf8"),
  };
}
