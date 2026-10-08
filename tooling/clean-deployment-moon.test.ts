import { afterEach, expect, setDefaultTimeout, test } from "bun:test";
import {
  cleanupDeploymentFixtures,
  createDeploymentFixture,
  expectSuccess,
  pathExists,
  project,
  runDeploymentHelper,
} from "./deployment-test-support.ts";
import { dirname, join, sep } from "node:path";
import {
  lstatSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  readlinkSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { prepareMoonWorkspaceFixture } from "./deployment-moon-test-support.ts";
import { runDeploymentMoon } from "./deployment-moon-runner.ts";

afterEach(cleanupDeploymentFixtures);
const deploymentTimeoutMilliseconds = 30_000;
setDefaultTimeout(deploymentTimeoutMilliseconds);
const portableMinimalTasks = [
  "home:nvim",
  "home:wezterm",
  "home:git-delta",
  "home:starship",
  "home:tmux",
  "home:cspell-config",
  "home:arnes-config",
  "harness:claude-instructions",
  "harness:claude-rules",
  "harness:claude-skills",
  "harness:codex-instructions",
  "harness:codex-agents",
  "harness:codex-skills",
  "arnes:binary",
  "agent-memory:binary",
  "agent-handoff:binary",
];

function deployedFiles(directory: string): readonly string[] {
  return readdirSync(directory).flatMap((name) => {
    const path = join(directory, name);
    return lstatSync(path).isDirectory() ? deployedFiles(path) : [path];
  });
}

function clean(
  home: string,
  environment: Readonly<NodeJS.ProcessEnv>,
  repositoryRoot: string = project,
): ReturnType<typeof runDeploymentMoon> {
  return runDeploymentMoon(
    { home, repositoryRoot },
    ["repository:clean"],
    environment,
  );
}

function identities(paths: readonly string[]): readonly Readonly<{
  path: string;
  kind: string;
  content: string | Buffer;
}>[] {
  return paths.map((path) => ({
    path,
    kind: lstatSync(path).isSymbolicLink() ? "link" : "file",
    content: lstatSync(path).isSymbolicLink()
      ? readlinkSync(path)
      : readFileSync(path),
  }));
}

function deploymentSnapshot(
  deployed: readonly string[],
  existing: readonly string[],
): Readonly<{
  deployed: ReturnType<typeof identities>;
  existing: ReturnType<typeof identities>;
}> {
  return { deployed: identities(deployed), existing: identities(existing) };
}

function deployedArtifacts(home: string): readonly string[] {
  const runtimeCaches = [
    join(home, "Library/Caches"),
    join(home, ".cache"),
    join(home, ".bun/install/cache"),
  ];
  return deployedFiles(home).filter(
    (path) => !runtimeCaches.some((cache) => path.startsWith(`${cache}${sep}`)),
  );
}

function cacheFixture(home: string): Readonly<{
  directory: string;
  before: ReturnType<typeof identities>;
}> {
  const directory = join(home, ".bun/install/cache");
  const invalidUtf8Byte = 0xff;
  mkdirSync(directory, { recursive: true });
  writeFileSync(
    join(directory, "foreign-package-cache"),
    Buffer.from([0, invalidUtf8Byte, 1]),
  );
  return { directory, before: identities(deployedFiles(directory)) };
}

test("cleans and reinstalls portable minimal Moon deployments without global dependencies", () => {
  const fixture = createDeploymentFixture("clean-moon-minimal");
  prepareMoonWorkspaceFixture(fixture.repository);
  const deployment = { home: fixture.home, repositoryRoot: fixture.repository };
  const existingFiles = deployedArtifacts(fixture.home);
  const environment = {
    MOON_BASE: "main",
    MOON_HEAD: "HEAD",
    BUN_RUNTIME_TRANSPILER_CACHE_PATH: join(
      fixture.root,
      "runtime-transpiler-cache",
    ),
  };
  expectSuccess(
    runDeploymentMoon(deployment, portableMinimalTasks, environment),
  );
  const cache = cacheFixture(fixture.home);
  const files = deployedArtifacts(fixture.home).filter(
    (path) =>
      !existingFiles.includes(path) &&
      path !== join(fixture.home, ".gitconfig"),
  );
  expect(files.length).toBeGreaterThan(0);
  const before = deploymentSnapshot(files, existingFiles);
  const source = readFileSync(
    join(fixture.repository, "harness/AGENTS.md"),
    "utf8",
  );
  expectSuccess(clean(fixture.home, environment, fixture.repository));
  expect(files.filter((path) => pathExists(path))).toEqual([]);
  expect(identities(existingFiles)).toEqual(before.existing);
  expect(identities(deployedFiles(cache.directory))).toEqual(cache.before);
  expect(
    readFileSync(join(fixture.repository, "harness/AGENTS.md"), "utf8"),
  ).toBe(source);
  expectSuccess(clean(fixture.home, environment, fixture.repository));
  expect(identities(deployedFiles(cache.directory))).toEqual(cache.before);
  expectSuccess(
    runDeploymentMoon(deployment, portableMinimalTasks, environment),
  );
  expect(deploymentSnapshot(files, existingFiles)).toEqual(before);
});

test("cleans and reinstalls optional Cursor, Herdr, PostgreSQL and Scrapling links separately", () => {
  const fixture = createDeploymentFixture("clean-moon-optional");
  const existingFiles = deployedArtifacts(fixture.home);
  const environment = {
    BUN_RUNTIME_TRANSPILER_CACHE_PATH: join(
      fixture.root,
      "runtime-transpiler-cache",
    ),
  };
  const tasks = ["harness:cursor-rules", "harness:cursor-skills", "home:herdr"];
  const postgresql = (): ReturnType<typeof runDeploymentMoon> =>
    runDeploymentMoon(fixture, ["home:postgresql"], environment);
  const scrapling = (): ReturnType<typeof runDeploymentMoon> =>
    runDeploymentMoon(fixture, ["tooling:scrapling-mcp"], environment);
  expectSuccess(runDeploymentMoon(fixture, tasks, environment));
  expectSuccess(postgresql());
  expectSuccess(scrapling());
  const cache = cacheFixture(fixture.home);
  const files = deployedArtifacts(fixture.home).filter(
    (path) => !existingFiles.includes(path),
  );
  const before = deploymentSnapshot(files, existingFiles);
  expectSuccess(clean(fixture.home, environment));
  expect(files.filter((path) => pathExists(path))).toEqual([]);
  expect(identities(existingFiles)).toEqual(before.existing);
  expect(identities(deployedFiles(cache.directory))).toEqual(cache.before);
  expectSuccess(runDeploymentMoon(fixture, tasks, environment));
  expectSuccess(postgresql());
  expectSuccess(scrapling());
  expect(deploymentSnapshot(files, existingFiles)).toEqual(before);
});

test("cleanup preserves a deployed Fish source and its installed plugin files", () => {
  const fixture = createDeploymentFixture("clean-fish-source");
  mkdirSync(join(fixture.repository, "home/.config/fish/functions"), {
    recursive: true,
  });
  writeFileSync(
    join(fixture.repository, "home/.arnes.yaml"),
    "version: 1\nskills: []\n",
  );
  const plugin = join(
    fixture.repository,
    "home/.config/fish/functions/fzf_configure_bindings.fish",
  );
  writeFileSync(plugin, "owned source plus installed plugin\n");
  const destination = join(fixture.home, ".config/fish");
  mkdirSync(dirname(destination), { recursive: true });
  symlinkSync(dirname(dirname(plugin)), destination);
  expectSuccess(
    runDeploymentHelper(fixture, {
      helper: "clean-deployment.ts",
      arguments: [fixture.repository, fixture.home, "--apply"],
    }),
  );
  expect(pathExists(destination)).toBeFalse();
  expect(readFileSync(plugin, "utf8")).toBe(
    "owned source plus installed plugin\n",
  );
});

test("cleans the Cursor harness projection while preserving other local plugins", () => {
  const fixture = createDeploymentFixture("cursor-clean");
  const plugin = join(fixture.home, ".cursor/plugins/local/dotfiles-harness");
  const commonRule = join(plugin, "rules/common-instructions.mdc");
  expect(
    runDeploymentMoon(fixture, ["harness:cursor-instructions"]).exitCode,
  ).toBe(0);
  const foreign = join(
    fixture.home,
    ".cursor/plugins/local/third-party/plugin.json",
  );
  mkdirSync(dirname(foreign), { recursive: true });
  writeFileSync(foreign, "keep\n");
  expect(
    runDeploymentHelper(fixture, {
      helper: "clean-deployment.ts",
      arguments: [project, fixture.home, "--apply"],
    }).exitCode,
  ).toBe(0);
  expect(pathExists(join(plugin, ".cursor-plugin/plugin.json"))).toBeFalse();
  expect(pathExists(commonRule)).toBeFalse();
  expect(readFileSync(foreign, "utf8")).toBe("keep\n");
});
