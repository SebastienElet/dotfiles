import type {
  CommandResult,
  DeploymentFixture,
} from "../deployment-test-support.ts";
import { afterEach, expect, test } from "bun:test";
import {
  cleanupDeploymentFixtures,
  createDeploymentFixture,
  project,
  requireCommand,
  runMake,
} from "../deployment-test-support.ts";
import { dirname, join } from "node:path";
import {
  lstatSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readlinkSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { fileURLToPath } from "node:url";
import { tmpdir } from "node:os";

const repository = fileURLToPath(new URL("../..", import.meta.url));
const psqlrcSource = join(repository, "home", ".psqlrc");
const fixtures: string[] = [];

function makePostgresql(home: string): Readonly<{
  exitCode: number;
  stdout: string;
  stderr: string;
}> {
  const result = Bun.spawnSync(
    [
      "/usr/bin/make",
      "--no-print-directory",
      "-f",
      join(repository, "Makefile"),
      "postgresql",
    ],
    {
      env: {
        HOME: home,
        PATH: `${dirname(process.execPath)}:/usr/bin:/bin`,
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

function createHome(): string {
  const home = mkdtempSync(join(tmpdir(), "optional-links-test-"));
  fixtures.push(home);
  return home;
}

afterEach(() => {
  for (const fixture of fixtures.splice(0)) {
    rmSync(fixture, { recursive: true, force: true });
  }
  cleanupDeploymentFixtures();
});

test("links psqlrc and replays silently", () => {
  const home = createHome();
  expect(makePostgresql(home).exitCode).toBe(0);
  const destination = join(home, ".psqlrc");
  expect(readlinkSync(destination)).toBe(psqlrcSource);
  const before = lstatSync(destination).ino;

  const replay = makePostgresql(home);

  expect(replay).toEqual({ exitCode: 0, stdout: "", stderr: "" });
  expect(lstatSync(destination).ino).toBe(before);
});

test("refuses a divergent psqlrc without replacing it", () => {
  const home = createHome();
  const destination = join(home, ".psqlrc");
  writeFileSync(destination, "personal\n");

  const result = makePostgresql(home);

  expect(result.exitCode).not.toBe(0);
  expect(result.stderr).toContain(
    `${destination} exists and is not the expected symbolic link`,
  );
  expect(readFileSync(destination, "utf8")).toBe("personal\n");
});

function scraplingFixture(): Readonly<{
  destination: string;
  fixture: DeploymentFixture;
  trace: string;
}> {
  const fixture = createDeploymentFixture("optional-scrapling");
  const destination = join(fixture.home, ".local", "bin", "scrapling_mcp");
  const trace = join(fixture.root, "docker-trace");
  mkdirSync(dirname(destination), { recursive: true });
  symlinkSync(
    join(project, "tooling", "docker-install-test-provider.ts"),
    join(fixture.bin, "docker"),
  );
  writeFileSync(trace, "");
  return { destination, fixture, trace };
}

function makeScrapling(
  fixture: DeploymentFixture,
  trace: string,
): CommandResult {
  return runMake(fixture, ["scrapling"], {
    repository: project,
    variables: {
      LOCAL_BIN: join(fixture.home, ".local", "bin"),
      DOCKER_UNAVAILABLE_POLICY: "allow-skip",
    },
    environment: {
      DOCKER_INSTALL_TEST_SCENARIO: "daemon-unavailable",
      DOCKER_INSTALL_TEST_STATE: trace,
      DOCKER_INSTALL_TEST_TARGET: "scrapling",
      PATH: `${fixture.bin}:${dirname(process.execPath)}:/usr/bin:/bin`,
    },
  });
}

test("make scrapling creates its link and preserves it on replay", () => {
  const { destination, fixture, trace } = scraplingFixture();
  const source = join(project, "tooling", "scrapling-mcp");

  const firstRun = makeScrapling(fixture, trace);
  expect(firstRun.exitCode).toBe(0);
  expect(firstRun.stdout).toContain(
    "docker-install target=scrapling result=skipped",
  );
  expect(readFileSync(trace, "utf8")).toContain("info\n");
  expect(readlinkSync(destination)).toBe(source);
  const inode = lstatSync(destination).ino;

  expect(makeScrapling(fixture, trace).exitCode).toBe(0);
  expect(lstatSync(destination).ino).toBe(inode);
});

test("make scrapling refuses an occupied link destination", () => {
  const { destination, fixture, trace } = scraplingFixture();
  writeFileSync(destination, "personal\n");

  const result = makeScrapling(fixture, trace);

  expect(result.exitCode).not.toBe(0);
  expect(readFileSync(destination, "utf8")).toBe("personal\n");
  expect(readFileSync(trace, "utf8")).toBe("");
});

function cursorFixture(): Readonly<{
  fixture: DeploymentFixture;
  trace: string;
}> {
  const fixture = createDeploymentFixture("optional-cursor");
  const trace = join(fixture.root, "arnes-trace");
  const arnes = join(fixture.home, ".local", "bin", "arnes");
  mkdirSync(dirname(arnes), { recursive: true });
  symlinkSync(join(project, "tooling", "deployment-test-command.ts"), arnes);
  writeFileSync(trace, "");
  return { fixture, trace };
}

function makeCursor(fixture: DeploymentFixture, trace: string): CommandResult {
  const moon = process.env.DEPLOYMENT_MOON ?? requireCommand("moon");
  return runMake(fixture, ["cursor"], {
    repository: project,
    variables: {
      MOON_EXEC: `${moon} exec --quiet --ignore-ci-checks --no-actions --upstream direct`,
    },
    environment: {
      FAKE_FAIL_ON: "doctor hooks",
      FAKE_TRACE: trace,
      MOON_HOME: process.env.MOON_HOME ?? join(process.env.HOME ?? "", ".moon"),
      PATH: `${fixture.bin}:${dirname(process.execPath)}:/usr/bin:/bin`,
      PROTO_HOME:
        process.env.PROTO_HOME ?? join(process.env.HOME ?? "", ".proto"),
      PROTO_OFFLINE: "true",
    },
  });
}

test("make cursor deploys the rule and skills without global dependencies", () => {
  const { fixture, trace } = cursorFixture();
  const rule = join(
    fixture.home,
    ".cursor",
    "rules",
    "memory-governance-cursor.mdc",
  );
  const skill = join(fixture.home, ".cursor", "skills", "skill-manager");

  expect(makeCursor(fixture, trace).exitCode).toBe(0);
  expect(readlinkSync(rule)).toBe(
    join(project, "harness", "rules", "memory-governance-cursor.mdc"),
  );
  expect(readlinkSync(skill)).toBe(
    join(project, "harness", "skills", "skill-manager"),
  );
  expect(readFileSync(trace, "utf8")).toContain(
    '"arguments":["setup","hooks","--agent","cursor"]',
  );
  const ruleInode = lstatSync(rule).ino;

  expect(makeCursor(fixture, trace).exitCode).toBe(0);
  expect(lstatSync(rule).ino).toBe(ruleInode);
});

test("make cursor preserves a divergent rule and returns failure", () => {
  const { fixture, trace } = cursorFixture();
  const rule = join(
    fixture.home,
    ".cursor",
    "rules",
    "memory-governance-cursor.mdc",
  );
  mkdirSync(dirname(rule), { recursive: true });
  writeFileSync(rule, "personal\n");

  const result = makeCursor(fixture, trace);

  expect(result.exitCode).not.toBe(0);
  expect(readFileSync(rule, "utf8")).toBe("personal\n");
});
