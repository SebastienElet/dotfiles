import { afterEach, describe, expect, test } from "bun:test";
import { dirname, join } from "node:path";
import {
  existsSync,
  mkdirSync,
  readFileSync,
  readlinkSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { z } from "zod";

const providerConfigurationSchema = z
  .object({
    mcpServers: z.object({ codegraph: z.object({ command: z.string() }) }),
    unrelated: z.string().optional(),
  })
  .loose();

const root = dirname(import.meta.dir);
const entryPoint = join(import.meta.dir, "codegraph-configure");
const provider = join(import.meta.dir, "codegraph-configure-test-provider.ts");
const temporaryDirectories: string[] = [];
const configurationErrorExitCode = 2;
const deploymentTestTimeoutMilliseconds = 15_000;

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
interface RealCliFixture {
  claudeConfig: string;
  codegraphBinary: string;
  codexConfig: string;
  cursorConfig: string;
  environment: Readonly<Record<string, string | undefined>>;
  log: string;
}

afterEach(() => {
  for (const path of temporaryDirectories.splice(0)) {
    rmSync(path, { force: true, recursive: true });
  }
});

describe("CodeGraph deployment", () => {
  registerConfiguratorDeploymentTest();
  registerVoltaDeploymentTest();
  registerIgnoreDeploymentTest();
});

function registerConfiguratorDeploymentTest(): void {
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
}

function registerVoltaDeploymentTest(): void {
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
}

function registerIgnoreDeploymentTest(): void {
  test(
    "CodeGraph ignore deployment is idempotent and refuses a foreign target",
    () => {
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

      expect(result.exitCode).toBe(configurationErrorExitCode);
      expect(result.stderr).toContain(
        "exists and is not the expected symbolic link",
      );
    },
    deploymentTestTimeoutMilliseconds,
  );
}

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
    const realCliPaths = z
      .tuple([z.string(), z.string(), z.string()])
      .parse([realClaude, realCodex, realVoltaHome]);
    const {
      claudeConfig,
      codegraphBinary,
      codexConfig,
      cursorConfig,
      environment,
      log,
    } = createRealCliFixture(...realCliPaths);
    const first = runEntryPoint(environment);
    expect(first.exitCode).toBe(0);
    expect(first.stdout).toContain("codegraph");
    expect(first.stderr).toBe("");
    const firstConfigurations = readConfigurations(
      claudeConfig,
      codexConfig,
      cursorConfig,
    );
    expect(runEntryPoint(environment).exitCode).toBe(0);
    expect(readConfigurations(claudeConfig, codexConfig, cursorConfig)).toEqual(
      firstConfigurations,
    );

    const claude = providerConfigurationSchema.parse(
      JSON.parse(firstConfigurations.claude),
    );
    const cursor = providerConfigurationSchema.parse(
      JSON.parse(firstConfigurations.cursor),
    );
    expect(claude.unrelated).toBe("claude");
    expect(claude.mcpServers.codegraph.command).toBe(codegraphBinary);
    expect(firstConfigurations.codex).toContain('unrelated = "codex"');
    expect(firstConfigurations.codex).toContain("[mcp_servers.codegraph]");
    expect(cursor.mcpServers.codegraph.command).toBe(codegraphBinary);
    expect(existsSync(log)).toBe(true);
  },
  deploymentTestTimeoutMilliseconds,
);

function createRealCliFixture(
  claudeBinary: string,
  codexBinary: string,
  voltaHome: string,
): Readonly<RealCliFixture> {
  const directory = createTemporaryDirectory();
  const home = join(directory, "home");
  const codexHome = join(directory, "codex-home");
  const binaries = join(directory, "bin");
  const claudeConfig = join(home, ".claude.json");
  const codexConfig = join(codexHome, "config.toml");
  const cursorConfig = join(directory, "cursor", "mcp.json");
  const log = join(directory, "calls.log");
  const codegraphBinary = join(binaries, "codegraph");
  mkdirSync(home, { recursive: true });
  mkdirSync(codexHome, { recursive: true });
  mkdirSync(binaries, { recursive: true });
  symlinkSync(provider, codegraphBinary);
  symlinkSync(process.execPath, join(binaries, "bun"));
  writeFileSync(claudeConfig, '{"unrelated":"claude"}\n');
  writeFileSync(codexConfig, 'unrelated = "codex"\n');
  return {
    claudeConfig,
    codegraphBinary,
    codexConfig,
    cursorConfig,
    environment: {
      ...process.env,
      CODEGRAPH_BIN: codegraphBinary,
      CODEGRAPH_CLAUDE_BIN: claudeBinary,
      CODEGRAPH_CLAUDE_CONFIG: claudeConfig,
      CODEGRAPH_CODEX_BIN: codexBinary,
      CODEGRAPH_CODEX_CONFIG: codexConfig,
      CODEGRAPH_CURSOR_CONFIG: cursorConfig,
      CODEGRAPH_TEST_LOG: log,
      CODEGRAPH_TEST_STATE: join(directory, "state"),
      CODEX_HOME: codexHome,
      HOME: home,
      PATH: `${binaries}:${process.env.PATH ?? ""}`,
      VOLTA_HOME: voltaHome,
    },
    log,
  };
}

function createTemporaryDirectory(): string {
  const directory = join(
    tmpdir(),
    `codegraph-deployment-${crypto.randomUUID()}`,
  );
  temporaryDirectories.push(directory);
  mkdirSync(directory, { recursive: true });
  return directory;
}

function makeDryRun(home: string, target: string): CommandResult {
  return runMake([
    "-Bn",
    `HOME=${home}`,
    `BREW_BIN=${join(home, "brew", "bin")}`,
    `VOLTA_BIN=${join(home, ".volta", "bin")}`,
    `LOCAL_BIN=${join(home, ".local", "bin")}`,
    target,
  ]);
}

function runMake(arguments_: readonly string[]): CommandResult {
  return run(["make", "-s", "-C", root, ...arguments_], process.env);
}

function runEntryPoint(
  environment: Readonly<Record<string, string | undefined>>,
): CommandResult {
  return run([entryPoint], environment);
}

function run(
  command: readonly string[],
  environment: Readonly<Record<string, string | undefined>>,
): CommandResult {
  const result = Bun.spawnSync([...command], {
    env: environment,
    stderr: "pipe",
    stdout: "pipe",
  });
  return {
    exitCode: result.exitCode,
    stderr: result.stderr.toString(),
    stdout: result.stdout.toString(),
  };
}

function readConfigurations(
  claude: string,
  codex: string,
  cursor: string,
): ConfigurationSnapshot {
  return {
    claude: readFileSync(claude, "utf8"),
    codex: readFileSync(codex, "utf8"),
    cursor: readFileSync(cursor, "utf8"),
  };
}
