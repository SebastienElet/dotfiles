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

test("deploys XDG links while preserving obsolete user links", () => {
  const fixture = createDeploymentFixture("xdg-links");
  const legacy = [".wezterm.lua", ".tmux.conf", ".gitconfig.delta"];
  for (const name of legacy) {
    symlinkSync(join(project, "home", name), join(fixture.home, name));
    expect(pathExists(join(project, "home", name))).toBeFalse();
  }
  const targets = xdgTargets(fixture.home);
  expectSuccess(
    runDeploymentMoon(fixture, ["home:wezterm", "home:tmux", "home:git-delta"]),
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
    runDeploymentMoon(fixture, ["home:wezterm", "home:tmux", "home:git-delta"]),
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
  expectSuccess(runDeploymentMoon(fixture, ["home:git-delta"]));
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
  writeFileSync(xdgConfig, "");
  writeFileSync(homeConfig, "");
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
  expect(includePathsFromFile(xdgConfig)).toEqual([
    "~/.config/git/other.conf",
    "~/xgitconfig.delta",
  ]);
  const effective = Bun.spawnSync(
    [requireCommand("git"), "config", "--get-all", "include.path"],
    {
      cwd: fixture.root,
      env: {
        ...process.env,
        HOME: fixture.home,
        XDG_CONFIG_HOME: join(fixture.home, ".config"),
        GIT_CONFIG_SYSTEM: "/dev/null",
      },
      stdout: "pipe",
      stderr: "pipe",
    },
  );
  expect(effective.stdout.toString().trim().split("\n")).toEqual([
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

test("migrates a Git include once when the home config links to XDG", () => {
  const fixture = createDeploymentFixture("xdg-git-alias");
  const xdgConfig = join(fixture.home, ".config", "git", "config");
  const homeConfig = join(fixture.home, ".gitconfig");
  mkdirSync(join(fixture.home, ".config", "git"), { recursive: true });
  writeFileSync(xdgConfig, "");
  for (const value of ["~/.gitconfig.delta", "~/.config/git/config.delta"]) {
    addInclude(fixture, xdgConfig, value);
  }
  symlinkSync(xdgConfig, homeConfig);

  expectSuccess(runGitIncludes(fixture));
  expect(includePathsFromFile(xdgConfig)).toEqual([
    "~/.config/git/config.delta",
  ]);
  expect(linkTarget(homeConfig)).toBe(xdgConfig);
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
  writeFileSync(xdgConfig, "");
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
  writeFileSync(xdgConfig, "");
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
  expect(includePaths(fixture)).not.toContain("~/.gitconfig.delta");
});
type Fixture = ReturnType<typeof createDeploymentFixture>;

function prepareGitDeltaFixture(fixture: Fixture): void {
  mkdirSync(join(fixture.home, ".config"), { recursive: true });
  for (const [name, source] of [
    [".wezterm.lua", join(project, "home", ".wezterm.lua")],
    [".tmux.conf", join(project, "home", ".tmux.conf")],
    [".gitconfig.delta", join(project, "home", ".gitconfig.delta")],
  ] as const) {
    symlinkSync(source, join(fixture.home, name));
  }
  const config = join(fixture.home, ".gitconfig");
  writeFileSync(config, "");
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
    environment,
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
  const result = Bun.spawnSync([requireCommand("git"), ...args], {
    stdout: "pipe",
  });
  if (result.exitCode === 1) {
    return [];
  }
  if (result.exitCode !== 0) {
    throw new Error("cannot read Git includes");
  }
  return result.stdout.toString().trim().split("\n");
}

function git(fixture: Fixture, arguments_: readonly string[]): void {
  const result = Bun.spawnSync([requireCommand("git"), ...arguments_], {
    env: { ...process.env, HOME: fixture.home },
  });
  if (result.exitCode !== 0) {
    throw new Error(result.stderr.toString());
  }
}

function xdgTargets(home: string): [string, string, string, string] {
  return [
    join(home, ".config", "wezterm", "wezterm.lua"),
    join(home, ".config", "tmux", "tmux.conf"),
    join(home, ".config", "git", "config.delta"),
    join(home, ".config", "git", "ignore"),
  ];
}
