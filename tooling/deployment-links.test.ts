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
  lstatSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  symlinkSync,
  utimesSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { runDeploymentMoon } from "./deployment-moon-runner.ts";

afterEach(cleanupDeploymentFixtures);

const deploymentTimeoutMilliseconds = 15_000;
setDefaultTimeout(deploymentTimeoutMilliseconds);

test("deploys the Cursor plugin selection through Moon and preserves its link on replay", () => {
  const fixture = createDeploymentFixture("agent-measure");
  const destination = join(
    fixture.home,
    ".config/agent-measure/cursor-plugins.json",
  );
  expectSuccess(runDeploymentMoon(fixture, ["home:agent-measure-config"]));
  expect(linkTarget(destination)).toBe(
    join(project, "home/.config/agent-measure/cursor-plugins.json"),
  );
  const before = fileIdentity(destination);
  const inode = lstatSync(destination).ino;
  const replay = runDeploymentMoon(fixture, ["home:agent-measure-config"]);
  expect(replay).toEqual({ exitCode: 0, stdout: "", stderr: "" });
  expect(fileIdentity(destination)).toEqual(before);
  expect(lstatSync(destination).ino).toBe(inode);
});

test("refuses an existing Cursor plugin selection without changing its contents", () => {
  const fixture = createDeploymentFixture("agent-measure-file");
  const destination = join(
    fixture.home,
    ".config/agent-measure/cursor-plugins.json",
  );
  mkdirSync(join(fixture.home, ".config/agent-measure"), { recursive: true });
  writeFileSync(destination, '["personal-plugin"]\n');
  const before = fileIdentity(destination);
  const result = runDeploymentMoon(fixture, ["home:agent-measure-config"]);
  expect(result.exitCode).not.toBe(0);
  expect(result.stderr).toContain(
    "exists and is not the expected symbolic link",
  );
  expect(fileIdentity(destination)).toEqual(before);
});

test("refuses a divergent Cursor plugin selection link without changing its target", () => {
  const fixture = createDeploymentFixture("agent-measure-link");
  const destination = join(
    fixture.home,
    ".config/agent-measure/cursor-plugins.json",
  );
  const foreign = join(fixture.root, "personal.json");
  mkdirSync(join(fixture.home, ".config/agent-measure"), { recursive: true });
  writeFileSync(foreign, '["personal-plugin"]\n');
  symlinkSync(foreign, destination);
  const before = fileIdentity(foreign);
  const inode = lstatSync(destination).ino;
  const result = runDeploymentMoon(fixture, ["home:agent-measure-config"]);
  expect(result.exitCode).not.toBe(0);
  expect(linkTarget(destination)).toBe(foreign);
  expect(lstatSync(destination).ino).toBe(inode);
  expect(fileIdentity(foreign)).toEqual(before);
});
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

test("migrates a Git include once when the home config links to XDG", () => {
  const fixture = createDeploymentFixture("xdg-git-alias");
  const xdgConfig = join(fixture.home, ".config", "git", "config");
  const homeConfig = join(fixture.home, ".gitconfig");
  mkdirSync(join(fixture.home, ".config", "git"), { recursive: true });
  writeFileSync(
    xdgConfig,
    "[include]\n  path = ~/.gitconfig.delta\n  path = ~/.config/git/config.delta\n",
  );
  symlinkSync(xdgConfig, homeConfig);

  expectSuccess(
    runDeploymentHelper(
      fixture,
      { helper: "deploy-git-includes.ts", arguments: [] },
      {
        XDG_CONFIG_HOME: join(fixture.home, ".config"),
        GIT_CONFIG_GLOBAL: undefined,
        GIT_CONFIG: undefined,
        GIT_CONFIG_COUNT: "0",
        GIT_CONFIG_PARAMETERS: undefined,
      },
    ),
  );
  const migrated = readFileSync(xdgConfig, "utf8");
  expect(migrated).toContain("~/.config/git/config.delta");
  expect(migrated).not.toContain("~/.gitconfig.delta");
  expect(linkTarget(homeConfig)).toBe(xdgConfig);
});

test("deploys Starship and tmux links and replays idempotently", () => {
  const fixture = createDeploymentFixture("starship");
  const tmux = join(fixture.home, ".config", "tmux", "tmux.conf");
  const starship = join(fixture.home, ".config", "starship.toml");
  const targets = ["home:tmux", "home:starship"];
  expectSuccess(runDeploymentMoon(fixture, targets));
  expect(linkTarget(tmux)).toBe(
    join(project, "home", ".config", "tmux", "tmux.conf"),
  );
  expect(linkTarget(starship)).toBe(
    join(project, "home", ".config", "starship.toml"),
  );
  const before = [tmux, starship].map((path) => fileIdentity(path));
  const replay = runDeploymentMoon(fixture, targets);
  expectSuccess(replay);
  expect(replay.stdout).toBe("");
  expect(replay.stderr).toBe("");
  expect([tmux, starship].map((path) => fileIdentity(path))).toEqual(before);
});

test("preserves a divergent Starship symlink", () => {
  const fixture = createDeploymentFixture("starship-collision");
  const destination = join(fixture.home, ".config", "starship.toml");
  const unexpected = join(fixture.root, "unexpected");
  mkdirSync(join(fixture.home, ".config"), { recursive: true });
  mkdirSync(unexpected);
  symlinkSync(unexpected, destination);
  const result = runDeploymentMoon(fixture, ["home:starship"]);
  expect(result.exitCode).not.toBe(0);
  expect(result.stderr).toContain(
    "exists and is not the expected symbolic link",
  );
  expect(linkTarget(destination)).toBe(unexpected);
});

test("deploys shared instructions and skills and replays idempotently", () => {
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
    join(
      project,
      "node_modules/@sebastienelet/agent-skills/skills/agent-instructions/references/maintenance.md",
    ),
  );
  expect(linkTarget(codexSkill)).toBe(
    join(
      project,
      "node_modules/@sebastienelet/agent-skills/skills/agent-instructions",
    ),
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
});

test("preserves an occupied Claude rule", () => {
  const fixture = createDeploymentFixture("claude-rule-collision");
  const claudeRule = join(
    fixture.home,
    ".claude",
    "rules",
    "agent-instructions.md",
  );
  mkdirSync(join(fixture.home, ".claude", "rules"), { recursive: true });
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
    readFileSync(join(project, "harness", "USER.md"), "utf8") +
    readFileSync(join(project, "harness", "visual-presentation.md"), "utf8")
  );
}
