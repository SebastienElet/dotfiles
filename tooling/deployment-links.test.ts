import { afterEach, expect, setDefaultTimeout, test } from "bun:test";
import {
  cleanupDeploymentFixtures,
  createDeploymentFixture,
  expectSuccess,
  fileIdentity,
  linkTarget,
  project,
  runDeploymentHelper,
} from "./deployment-test-support.ts";
import {
  mkdirSync,
  readFileSync,
  readdirSync,
  symlinkSync,
  unlinkSync,
  utimesSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { runDeploymentMoon } from "./deployment-moon-runner.ts";

afterEach(cleanupDeploymentFixtures);

const deploymentTimeoutMilliseconds = 15_000;
setDefaultTimeout(deploymentTimeoutMilliseconds);
test("refuses existing directories without linking inside them", () => {
  const fixture = createDeploymentFixture("existing-directory");
  const source = join(fixture.repository, "home", ".config", "fish");
  const destination = join(fixture.home, ".config", "fish");
  mkdirSync(source, { recursive: true });
  mkdirSync(destination, { recursive: true });
  utimesSync(destination, new Date("2020-01-01"), new Date("2020-01-01"));
  utimesSync(source, new Date("2021-01-01"), new Date("2021-01-01"));

  const result = runDeploymentHelper(fixture, {
    helper: "deploy-link.ts",
    arguments: [source, destination],
  });

  expect(result.exitCode).not.toBe(0);
  expect(readdirSync(destination)).toEqual([]);
});

test("refuses a destination symlink to a directory without mutating it", () => {
  const fixture = createDeploymentFixture("directory-link");
  const source = join(fixture.repository, "home", ".config", "fish");
  const destination = join(fixture.home, ".config", "fish");
  const actual = join(fixture.root, "actual");
  mkdirSync(source, { recursive: true });
  mkdirSync(join(fixture.home, ".config"), { recursive: true });
  mkdirSync(actual);
  symlinkSync(actual, destination);
  utimesSync(actual, new Date("2020-01-01"), new Date("2020-01-01"));
  utimesSync(source, new Date("2021-01-01"), new Date("2021-01-01"));

  const result = runDeploymentHelper(fixture, {
    helper: "deploy-link.ts",
    arguments: [source, destination],
  });

  expect(result.exitCode).not.toBe(0);
  expect(linkTarget(destination)).toBe(actual);
  expect(readdirSync(actual)).toEqual([]);
});

test(
  "deploys Starship and tmux links, replays idempotently, and preserves a wrong link",
  () => {
    const fixture = createDeploymentFixture("starship");
    const tmux = join(fixture.home, ".config", "tmux", "tmux.conf");
    const starship = join(fixture.home, ".config", "starship.toml");
    expectSuccess(runDeploymentMoon(fixture, ["home:tmux"]));
    expect(linkTarget(tmux)).toBe(
      join(project, "home", ".config", "tmux", "tmux.conf"),
    );
    expectSuccess(runDeploymentMoon(fixture, ["home:starship"]));
    expect(linkTarget(starship)).toBe(
      join(project, "home", ".config", "starship.toml"),
    );

    const before = fileIdentity(starship);
    const replay = runDeploymentMoon(fixture, ["home:starship"]);
    expectSuccess(replay);
    expect(replay.stdout).toBe("");
    expect(replay.stderr).toBe("");
    expect(fileIdentity(starship)).toEqual(before);

    expectDivergentSymlinkRejected(fixture, starship);
  },
  deploymentTimeoutMilliseconds,
);

test("deploys the guarded ColGrep entry point without replacing a destination", () => {
  const fixture = createDeploymentFixture("colgrep-search");
  const destination = join(fixture.home, ".local", "bin", "colgrep-search");

  expectSuccess(runDeploymentMoon(fixture, ["tooling:colgrep-search-install"]));
  expect(linkTarget(destination)).toBe(
    join(project, "tooling", "colgrep-search-cli.ts"),
  );
  expectSuccess(runDeploymentMoon(fixture, ["tooling:colgrep-search-install"]));

  unlinkSync(destination);
  writeFileSync(destination, "keep\n");
  const divergent = runDeploymentMoon(fixture, [
    "tooling:colgrep-search-install",
  ]);
  expect(divergent.exitCode).not.toBe(0);
  expect(readFileSync(destination, "utf8")).toBe("keep\n");
});

function expectDivergentSymlinkRejected(
  fixture: ReturnType<typeof createDeploymentFixture>,
  destination: string,
): void {
  unlinkSync(destination);
  const unexpected = join(fixture.root, "unexpected");
  mkdirSync(unexpected);
  symlinkSync(unexpected, destination);
  const result = runDeploymentMoon(fixture, ["home:starship"]);
  expect(result.exitCode).not.toBe(0);
  expect(result.stderr).toContain(
    `exists and is not the expected symbolic link`,
  );
  expect(linkTarget(destination)).toBe(unexpected);
}

test("deploys shared instructions and skills, rejects divergent rules, and replays idempotently", () => {
  const fixture = createDeploymentFixture("agent-instructions");
  const claudeRule = join(
    fixture.home,
    ".claude",
    "rules",
    "agent-instructions.md",
  );
  const codexInstructions = join(fixture.home, ".codex", "AGENTS.md");
  const codexSkill = join(
    fixture.home,
    ".agents",
    "skills",
    "agent-instructions",
  );
  expectSuccess(
    runDeploymentMoon(fixture, [
      "harness:claude-rules",
      "harness:codex-instructions",
      "harness:codex-skills",
    ]),
  );
  expect(linkTarget(claudeRule)).toBe(
    join(project, "harness", "rules", "agent-instructions.md"),
  );
  expect(linkTarget(codexSkill)).toBe(
    join(project, "harness", "skills", "agent-instructions"),
  );
  expect(readFileSync(codexInstructions, "utf8")).toBe(
    expectedCodexInstructions(),
  );
  const before = fileIdentity(codexInstructions);
  expectSuccess(
    runDeploymentMoon(fixture, [
      "harness:claude-rules",
      "harness:codex-instructions",
    ]),
  );
  expect(fileIdentity(codexInstructions)).toEqual(before);
  unlinkSync(claudeRule);
  writeFileSync(claudeRule, "keep\n");
  const divergent = runDeploymentMoon(fixture, ["harness:claude-rules"]);
  expect(divergent.exitCode).not.toBe(0);
  expect(divergent.stderr).toContain(
    "exists and is not the expected symbolic link",
  );
  expect(readFileSync(claudeRule, "utf8")).toBe("keep\n");
});

function expectedCodexInstructions(): string {
  return (
    readFileSync(join(project, "harness", "AGENTS.md"), "utf8").replaceAll(
      /^@.*\n/gmu,
      "",
    ) +
    readFileSync(join(project, "harness", "SOUL.md"), "utf8") +
    readFileSync(join(project, "harness", "USER.md"), "utf8")
  );
}
