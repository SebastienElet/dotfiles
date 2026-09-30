import { afterEach, expect, setDefaultTimeout, test } from "bun:test";
import {
  cleanupDeploymentFixtures,
  createDeploymentFixture,
  expectSuccess,
  pathExists,
  project,
  runDeploymentHelper,
  runMake,
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
import { runDeploymentMoon } from "./deployment-moon-runner.ts";

afterEach(cleanupDeploymentFixtures);
const deploymentTimeoutMilliseconds = 30_000;
setDefaultTimeout(deploymentTimeoutMilliseconds);

function deployedFiles(directory: string): readonly string[] {
  return readdirSync(directory).flatMap((name) => {
    const path = join(directory, name);
    return lstatSync(path).isDirectory() ? deployedFiles(path) : [path];
  });
}

function clean(home: string): ReturnType<typeof runDeploymentMoon> {
  return runDeploymentMoon({ home }, ["repository:clean"]);
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
  const tasks = [
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
    "tooling:colgrep-search-install",
    "arnes:binary",
    "agent-memory:binary",
    "agent-handoff:binary",
  ];
  expectSuccess(runDeploymentMoon(fixture, tasks));
  const cache = cacheFixture(fixture.home);
  const files = deployedArtifacts(fixture.home).filter(
    (path) => path !== join(fixture.home, ".gitconfig"),
  );
  expect(files.length).toBeGreaterThan(0);
  const before = identities(files);
  const source = readFileSync(join(project, "harness/AGENTS.md"), "utf8");
  expectSuccess(clean(fixture.home));
  expect(files.filter((path) => pathExists(path))).toEqual([]);
  expect(identities(deployedFiles(cache.directory))).toEqual(cache.before);
  expect(readFileSync(join(project, "harness/AGENTS.md"), "utf8")).toBe(source);
  expectSuccess(clean(fixture.home));
  expect(identities(deployedFiles(cache.directory))).toEqual(cache.before);
  expectSuccess(runDeploymentMoon(fixture, tasks));
  expect(identities(files)).toEqual(before);
});

test("cleans and reinstalls optional Cursor, PostgreSQL and Scrapling links separately", () => {
  const fixture = createDeploymentFixture("clean-moon-optional");
  const tasks = ["harness:cursor-rules", "harness:cursor-skills"];
  const postgresql = (): ReturnType<typeof runMake> =>
    runMake(fixture, ["postgresql"], { repository: project });
  const scrapling = (): ReturnType<typeof runDeploymentHelper> =>
    runDeploymentHelper(fixture, {
      helper: "deploy-link.ts",
      arguments: [
        join(project, "tooling/scrapling-mcp"),
        join(fixture.home, ".local/bin/scrapling_mcp"),
      ],
    });
  mkdirSync(join(fixture.home, ".local/bin"), { recursive: true });
  expectSuccess(runDeploymentMoon(fixture, tasks));
  expectSuccess(postgresql());
  expectSuccess(scrapling());
  const cache = cacheFixture(fixture.home);
  const files = deployedArtifacts(fixture.home);
  const before = identities(files);
  expectSuccess(clean(fixture.home));
  expect(files.filter((path) => pathExists(path))).toEqual([]);
  expect(identities(deployedFiles(cache.directory))).toEqual(cache.before);
  expectSuccess(runDeploymentMoon(fixture, tasks));
  expectSuccess(postgresql());
  expectSuccess(scrapling());
  expect(identities(files)).toEqual(before);
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
