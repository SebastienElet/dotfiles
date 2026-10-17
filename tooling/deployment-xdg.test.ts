import { afterEach, expect, setDefaultTimeout, test } from "bun:test";
import {
  cleanupDeploymentFixtures,
  createDeploymentFixture,
  expectSuccess,
  fileIdentity,
  installProvider,
  linkTarget,
  pathExists,
  project,
  requireCommand,
  runDeploymentHelper,
} from "./deployment-test-support.ts";
import { mkdirSync, symlinkSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { runDeploymentMoon } from "./deployment-moon-runner.ts";

afterEach(cleanupDeploymentFixtures);
const deploymentTimeoutMilliseconds = 15_000;
setDefaultTimeout(deploymentTimeoutMilliseconds);
const gitIsolation = {
  GIT_CONFIG_GLOBAL: undefined,
  GIT_CONFIG: undefined,
  GIT_CONFIG_COUNT: "0",
  GIT_CONFIG_PARAMETERS: undefined,
};

test("deploys XDG links while preserving obsolete user links", () => {
  const fixture = createDeploymentFixture("xdg-links");
  const legacy = [".wezterm.lua", ".tmux.conf", ".gitconfig.delta"];
  for (const name of legacy) {
    symlinkSync(join(project, "home", name), join(fixture.home, name));
    expect(pathExists(join(project, "home", name))).toBeFalse();
  }
  const targets = xdgTargets(fixture.home);
  expectSuccess(
    runXdgDeploymentMoon(fixture, [
      "home:wezterm",
      "home:tmux",
      "home:git-delta",
    ]),
  );
  expect(targets.map((target) => linkTarget(target))).toEqual([
    join(project, "home", ".config", "wezterm", "wezterm.lua"),
    join(project, "home", ".config", "tmux", "tmux.conf"),
    join(project, "home", ".config", "git", "config.delta"),
    join(project, "home", ".config", "git", "ignore"),
  ]);
  expect(legacy.map((name) => linkTarget(join(fixture.home, name)))).toEqual(
    legacy.map((name) => join(project, "home", name)),
  );
});

test("migrates Git includes exactly once and preserves unrelated values", () => {
  const fixture = createDeploymentFixture("xdg-git");
  prepareGitDeltaFixture(fixture);
  expectSuccess(
    runXdgDeploymentMoon(fixture, [
      "home:wezterm",
      "home:tmux",
      "home:git-delta",
    ]),
  );
  expect(includePaths(fixture)).toEqual([
    "~/.config/git/config.delta",
    "~/.config/git/other.conf",
    "~/xgitconfig.delta",
  ]);
  const config = join(fixture.home, ".gitconfig");
  const links = xdgTargets(fixture.home);
  const before = {
    config: fileIdentity(config),
    links: links.map((link) => fileIdentity(link)),
  };
  expectSuccess(runXdgDeploymentMoon(fixture, ["home:git-delta"]));
  expect({
    config: fileIdentity(config),
    links: links.map((link) => fileIdentity(link)),
  }).toEqual(before);
});

test("migrates an XDG Git include when the home Git config also exists", () => {
  const fixture = createDeploymentFixture("xdg-git-include");
  const xdgConfig = join(fixture.home, ".config", "git", "config");
  const homeConfig = join(fixture.home, ".gitconfig");
  mkdirSync(join(fixture.home, ".config", "git"), { recursive: true });
  for (const value of [
    "~/.gitconfig.delta",
    "~/.config/git/other.conf",
    "~/xgitconfig.delta",
    "~/.gitconfig.delta",
  ]) {
    addInclude(fixture, xdgConfig, value);
  }
  addInclude(fixture, homeConfig, "~/.config/git/home.conf");
  expectSuccess(runGitIncludes(fixture));
  expect(
    git(fixture, ["config", "--get-all", "include.path"]).trim().split("\n"),
  ).toEqual([
    "~/.config/git/other.conf",
    "~/xgitconfig.delta",
    "~/.config/git/home.conf",
    "~/.config/git/config.delta",
  ]);
  const before = [fileIdentity(xdgConfig), fileIdentity(homeConfig)];
  const second = runGitIncludes(fixture);
  expectSuccess(second);
  expect(second.stdout).toBe("");
  expect(second.stderr).toBe("");
  expect([fileIdentity(xdgConfig), fileIdentity(homeConfig)]).toEqual(before);
});

test("preserves configuration when include addition fails", () => {
  const fixture = createDeploymentFixture("xdg-add-failure");
  prepareGitDeltaFixture(fixture);
  git(fixture, [
    "config",
    "--file",
    join(fixture.home, ".gitconfig"),
    "--unset-all",
    "include.path",
    "^~/.config/git/config[.]delta$",
  ]);
  const config = join(fixture.home, ".gitconfig");
  const before = fileIdentity(config);
  const result = runGitIncludesWithFailure(fixture, [
    "config",
    "--global",
    "--add",
    "include.path",
    "~/.config/git/config.delta",
  ]);
  expect(result.exitCode).not.toBe(0);
  expect(result.stderr).toContain("Could not update Git includes");
  expect(fileIdentity(config)).toEqual(before);
  expectSuccess(runGitIncludes(fixture));
  expect(includePaths(fixture)).toContain("~/.config/git/config.delta");
  expect(includePaths(fixture)).not.toContain("~/.gitconfig.delta");
});

test("preserves both includes when removal fails after addition", () => {
  const fixture = createDeploymentFixture("xdg-remove-failure");
  const xdgConfig = join(fixture.home, ".config", "git", "config");
  const homeConfig = join(fixture.home, ".gitconfig");
  mkdirSync(join(fixture.home, ".config", "git"), { recursive: true });
  writeFileSync(homeConfig, "");
  addInclude(fixture, xdgConfig, "~/.gitconfig.delta");
  const result = runGitIncludesWithFailure(fixture, [
    "config",
    "--file",
    xdgConfig,
    "--unset-all",
    "include.path",
    "^~/[.]gitconfig[.]delta$",
  ]);
  expect(result.exitCode).not.toBe(0);
  expect(result.stderr).toContain("Could not update Git includes");
  expect(includePathsFromFile(homeConfig)).toContain(
    "~/.config/git/config.delta",
  );
  expect(includePathsFromFile(xdgConfig)).toContain("~/.gitconfig.delta");
  expectSuccess(runGitIncludes(fixture));
  expect(includePathsFromFile(xdgConfig)).toEqual([]);
});

test("does not mutate configuration when reading includes fails", () => {
  const fixture = createDeploymentFixture("xdg-read-failure");
  prepareGitDeltaFixture(fixture);
  const xdgConfig = join(fixture.home, ".config", "git", "config");
  mkdirSync(join(fixture.home, ".config", "git"), { recursive: true });
  addInclude(fixture, xdgConfig, "~/.gitconfig.delta");
  const homeConfig = join(fixture.home, ".gitconfig");
  const before = [fileIdentity(xdgConfig), fileIdentity(homeConfig)];
  const result = runGitIncludesWithFailure(fixture, [
    "config",
    "--null",
    "--file",
    xdgConfig,
    "--get-all",
    "include.path",
  ]);
  expect(result.exitCode).not.toBe(0);
  expect(result.stderr).toContain("Could not read Git includes");
  expect([fileIdentity(xdgConfig), fileIdentity(homeConfig)]).toEqual(before);
  expectSuccess(runGitIncludes(fixture));
  expect(includePathsFromFile(xdgConfig)).toEqual([]);
});
type Fixture = ReturnType<typeof createDeploymentFixture>;

function runXdgDeploymentMoon(
  fixture: Fixture,
  tasks: readonly string[],
): ReturnType<typeof runDeploymentMoon> {
  return runDeploymentMoon(fixture, tasks, {
    ...gitIsolation,
    XDG_CONFIG_HOME: join(fixture.home, ".config"),
  });
}

function prepareGitDeltaFixture(fixture: Fixture): void {
  mkdirSync(join(fixture.home, ".config"), { recursive: true });
  for (const name of [".wezterm.lua", ".tmux.conf", ".gitconfig.delta"]) {
    symlinkSync(join(project, "home", name), join(fixture.home, name));
  }
  const config = join(fixture.home, ".gitconfig");
  for (const value of [
    "~/.gitconfig.delta",
    "~/.config/git/config.delta",
    "~/.config/git/other.conf",
    "~/xgitconfig.delta",
    "~/.gitconfig.delta",
  ]) {
    git(fixture, ["config", "--file", config, "--add", "include.path", value]);
  }
}

function runGitIncludesWithFailure(
  fixture: Fixture,
  failure: readonly string[],
): ReturnType<typeof runDeploymentHelper> {
  installProvider(fixture, "git");
  return runGitIncludes(fixture, {
    PATH: `${fixture.bin}:${process.env.PATH ?? ""}`,
    DEPLOYMENT_PROVIDER_MODE: "git",
    DEPLOYMENT_REAL_COMMAND: requireCommand("git"),
    DEPLOYMENT_FAIL_ARGUMENTS: JSON.stringify(failure),
  });
}

function runGitIncludes(
  fixture: Fixture,
  environment: Readonly<NodeJS.ProcessEnv> = {},
): ReturnType<typeof runDeploymentHelper> {
  return runDeploymentHelper(
    fixture,
    { helper: "deploy-git-includes.ts", arguments: [] },
    {
      ...environment,
      ...gitIsolation,
      XDG_CONFIG_HOME: join(fixture.home, ".config"),
    },
  );
}

function addInclude(fixture: Fixture, config: string, value: string): void {
  git(fixture, ["config", "--file", config, "--add", "include.path", value]);
}

function includePaths(fixture: Fixture): string[] {
  return includePathsFromFile(join(fixture.home, ".gitconfig"));
}

function includePathsFromFile(config: string): string[] {
  const args = ["config", "--file", config, "--get-all", "include.path"];
  const gitCommand = requireCommand("git");
  const result = Bun.spawnSync([gitCommand, ...args], { stdout: "pipe" });
  if (result.exitCode === 1) {
    return [];
  }
  if (result.exitCode !== 0) {
    throw new Error("cannot read Git includes");
  }
  return result.stdout.toString().trim().split("\n");
}

function git(fixture: Fixture, arguments_: readonly string[]): string {
  const result = Bun.spawnSync([requireCommand("git"), ...arguments_], {
    cwd: fixture.root,
    env: {
      ...process.env,
      ...gitIsolation,
      HOME: fixture.home,
      XDG_CONFIG_HOME: join(fixture.home, ".config"),
      GIT_CONFIG_SYSTEM: "/dev/null",
    },
  });
  if (result.exitCode !== 0) {
    throw new Error(result.stderr.toString());
  }
  return result.stdout.toString();
}

function xdgTargets(home: string): [string, string, string, string] {
  return [
    join(home, ".config", "wezterm", "wezterm.lua"),
    join(home, ".config", "tmux", "tmux.conf"),
    join(home, ".config", "git", "config.delta"),
    join(home, ".config", "git", "ignore"),
  ];
}
