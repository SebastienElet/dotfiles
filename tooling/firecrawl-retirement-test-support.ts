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

const entryPoint = join(import.meta.dir, "retire-firecrawl");
const provider = join(import.meta.dir, "firecrawl-retirement-test-provider.ts");
const executableMode = 0o755;
const fixtures: string[] = [];

type Scenario =
  | "absent"
  | "concurrent-configuration"
  | "daemon-unavailable"
  | "existing"
  | "images-only"
  | "malformed-docker"
  | "persistent-docker"
  | "rollback-concurrent-configuration";

type Fixture = Readonly<{
  claudeConfig: string;
  cursorConfig: string;
  environment: Readonly<Record<string, string>>;
  log: string;
  root: string;
}>;

function createFixture(scenario: Scenario): Fixture {
  const root = mkdtempSync(join(tmpdir(), "firecrawl-retirement-"));
  const home = join(root, "home");
  const binaryDirectory = join(root, "bin");
  const log = join(root, "calls.log");
  const claudeConfig = join(home, ".claude.json");
  const cursorConfig = join(home, ".cursor", "mcp.json");
  fixtures.push(root);
  mkdirSync(join(home, ".cursor"), { recursive: true });
  mkdirSync(binaryDirectory);
  writeFileSync(log, "");
  chmodSync(provider, executableMode);
  symlinkSync(provider, join(binaryDirectory, "codex"));
  symlinkSync(provider, join(binaryDirectory, "docker"));
  return {
    claudeConfig,
    cursorConfig,
    environment: {
      DOCKER_UNAVAILABLE_POLICY: "require-docker",
      FIRECRAWL_RETIREMENT_CODEX_BIN: join(binaryDirectory, "codex"),
      FIRECRAWL_RETIREMENT_TEST_CLAUDE_CONFIG: claudeConfig,
      FIRECRAWL_RETIREMENT_DOCKER_BIN: join(binaryDirectory, "docker"),
      FIRECRAWL_RETIREMENT_TEST_LOG: log,
      FIRECRAWL_RETIREMENT_TEST_SCENARIO: scenario,
      HOME: home,
    },
    log,
    root,
  };
}

function run(fixture: Fixture): Readonly<{
  exitCode: number;
  stderr: string;
  stdout: string;
}> {
  const result = Bun.spawnSync([entryPoint], {
    env: { ...process.env, ...fixture.environment },
    stderr: "pipe",
    stdout: "pipe",
  });
  return {
    exitCode: result.exitCode,
    stderr: result.stderr.toString(),
    stdout: result.stdout.toString(),
  };
}

function readJson(path: string): unknown {
  return JSON.parse(readFileSync(path, "utf8"));
}

function readLog(fixture: Fixture): string {
  return readFileSync(fixture.log, "utf8");
}

function cleanupFixtures(): void {
  for (const fixture of fixtures.splice(0)) {
    rmSync(fixture, { force: true, recursive: true });
  }
}

export { cleanupFixtures, createFixture, type Fixture, readJson, readLog, run };
