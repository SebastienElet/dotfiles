import { afterEach, describe, expect, setDefaultTimeout, test } from "bun:test";
import {
  closeSync,
  mkdirSync,
  openSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
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
  runMake,
} from "./deployment-test-support.ts";

afterEach(cleanupDeploymentFixtures);
setDefaultTimeout(15_000);

describe("deployment area: XDG configuration", () => {
  test("deploys XDG links while preserving obsolete user links", () => {
    const fixture = createDeploymentFixture("xdg-links");
    const legacy = [".wezterm.lua", ".tmux.conf", ".gitconfig.delta"];
    for (const name of legacy) {
      symlinkSync(join(project, "home", name), join(fixture.home, name));
      expect(pathExists(join(project, "home", name))).toBeFalse();
    }
    const targets = xdgTargets(fixture.home);
    expectSuccess(runMake(fixture, targets, { repository: project }));
    expect(targets.map(linkTarget)).toEqual([
      join(project, "home", ".config", "wezterm", "wezterm.lua"),
      join(project, "home", ".config", "tmux", "tmux.conf"),
      join(project, "home", ".config", "git", "config.delta"),
    ]);
    expect(legacy.map((name) => linkTarget(join(fixture.home, name)))).toEqual(
      legacy.map((name) => join(project, "home", name)),
    );
  });

  test("public targets wire their XDG destinations", () => {
    const fixture = createDeploymentFixture("xdg-wiring");
    const targets = xdgTargets(fixture.home);
    for (const [name, expected] of [
      ["wezterm", targets[0]],
      ["tmux", targets[1]],
      ["git-delta", targets[2]],
    ] as const) {
      const result = runMake(fixture, [name], {
        repository: project,
        dryRun: true,
      });
      expectSuccess(result);
      expect(result.stdout).toContain(expected);
    }
  });

  test("migrates Git includes exactly once and preserves unrelated values", () => {
    const fixture = createDeploymentFixture("xdg-git");
    prepareGitDeltaFixture(fixture);
    expectSuccess(
      runMake(fixture, xdgTargets(fixture.home), { repository: project }),
    );
    expectSuccess(runGitDelta(fixture));
    expect(includePaths(fixture)).toEqual([
      "~/.config/git/config.delta",
      "~/.config/git/other.conf",
      "~/xgitconfig.delta",
    ]);
    const config = join(fixture.home, ".gitconfig");
    const links = xdgTargets(fixture.home);
    const before = {
      config: fileIdentity(config),
      links: links.map(fileIdentity),
    };
    expectSuccess(runGitDelta(fixture));
    expect({
      config: fileIdentity(config),
      links: links.map(fileIdentity),
    }).toEqual(before);
  });

  test("rolls back a failed include addition", () => {
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
    const result = runGitDelta(fixture, [
      "config",
      "--global",
      "--add",
      "include.path",
      "~/.config/git/config.delta",
    ]);
    expect(result.exitCode).not.toBe(0);
    expect(fileIdentity(config)).toEqual(before);
  });

  test("preserves both includes when removal fails after addition", () => {
    const fixture = createDeploymentFixture("xdg-remove-failure");
    prepareGitDeltaFixture(fixture);
    git(fixture, [
      "config",
      "--file",
      join(fixture.home, ".gitconfig"),
      "--unset-all",
      "include.path",
      "^~/.config/git/config[.]delta$",
    ]);
    const result = runGitDelta(fixture, [
      "config",
      "--global",
      "--unset-all",
      "include.path",
      "^~/[.]gitconfig[.]delta$",
    ]);
    expect(result.exitCode).not.toBe(0);
    expect(includePaths(fixture)).toContain("~/.config/git/config.delta");
    expect(includePaths(fixture)).toContain("~/.gitconfig.delta");
  });

  test("does not mutate configuration when reading includes fails", () => {
    const fixture = createDeploymentFixture("xdg-read-failure");
    prepareGitDeltaFixture(fixture);
    const config = join(fixture.home, ".gitconfig");
    const before = fileIdentity(config);
    const result = runGitDelta(fixture, [
      "config",
      "--global",
      "--get-all",
      "include.path",
    ]);
    expect(result.exitCode).not.toBe(0);
    expect(fileIdentity(config)).toEqual(before);
  });
});

type Fixture = ReturnType<typeof createDeploymentFixture>;

function prepareGitDeltaFixture(fixture: Fixture): void {
  for (const binary of ["brew", "delta"]) {
    closeSync(openSync(join(fixture.bin, binary), "w"));
  }
  mkdirSync(join(fixture.home, ".config"), { recursive: true });
  for (const [name, source] of [
    [".wezterm.lua", join(project, "home", ".wezterm.lua")],
    [".tmux.conf", join(project, "home", ".tmux.conf")],
    [".gitconfig.delta", join(project, "home", ".gitconfig.delta")],
  ] as const)
    symlinkSync(source, join(fixture.home, name));
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

function runGitDelta(fixture: Fixture, failure?: readonly string[]) {
  const environment: NodeJS.ProcessEnv = {};
  if (failure !== undefined) {
    installProvider(fixture, "git");
    environment.PATH = `${fixture.bin}:${process.env.PATH ?? ""}`;
    environment.DEPLOYMENT_PROVIDER_MODE = "git";
    environment.DEPLOYMENT_REAL_COMMAND = requireCommand("git");
    environment.DEPLOYMENT_FAIL_ARGUMENTS = JSON.stringify(failure);
  }
  return runMake(fixture, ["git-delta"], {
    repository: project,
    environment,
    variables: { BREW_BIN: fixture.bin },
  });
}

function includePaths(fixture: Fixture): string[] {
  const result = Bun.spawnSync(
    [
      requireCommand("git"),
      "config",
      "--file",
      join(fixture.home, ".gitconfig"),
      "--get-all",
      "include.path",
    ],
    { stdout: "pipe" },
  );
  if (result.exitCode !== 0) throw new Error("cannot read Git includes");
  return result.stdout.toString().trim().split("\n");
}

function git(fixture: Fixture, arguments_: readonly string[]): void {
  const result = Bun.spawnSync([requireCommand("git"), ...arguments_], {
    env: { ...process.env, HOME: fixture.home },
  });
  if (result.exitCode !== 0) throw new Error(result.stderr.toString());
}

function xdgTargets(home: string): [string, string, string] {
  return [
    join(home, ".config", "wezterm", "wezterm.lua"),
    join(home, ".config", "tmux", "tmux.conf"),
    join(home, ".config", "git", "config.delta"),
  ];
}
