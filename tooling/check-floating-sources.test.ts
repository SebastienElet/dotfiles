import { afterEach, expect, test } from "bun:test";
import { dirname, join } from "node:path";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";

const fixtures: string[] = [];
const exceptedFiles = {
  ".moon/tasks/harness-claude.yml":
    "curl -fsSL https://claude.ai/install.sh | bash -s latest\n",
  ".moon/tasks/harness-codex.yml": "script: volta install @openai/codex\n",
  "tooling/agent-reach-package.ts":
    '"uvx mcp-server-linkedin@latest --login",\n',
  "install.sh":
    "git clone --depth 1 https://github.com/SebastienElet/dotfiles.git .dotfiles\n",
};

afterEach(() => {
  for (const root of fixtures.splice(0)) {
    rmSync(root, { force: true, recursive: true });
  }
});

function createRepository(
  files: Readonly<Record<string, string>>,
  homebrewPackages: readonly string[] = [],
): string {
  const root = mkdtempSync(join(tmpdir(), "floating-sources-test-"));
  fixtures.push(root);
  const lock = {
    formulae: Object.fromEntries(
      homebrewPackages.map((name) => [name, "1.0.0"]),
    ),
    casks: {},
  };
  for (const [path, content] of Object.entries({
    ...exceptedFiles,
    "homebrew.lock.json": JSON.stringify(lock),
    ...files,
  })) {
    mkdirSync(dirname(join(root, path)), { recursive: true });
    writeFileSync(join(root, path), content);
  }
  for (const command of [
    ["init", "--quiet"],
    ["add", "--all"],
  ]) {
    Bun.spawnSync(["git", "-C", root, ...command]);
  }
  return root;
}

function check(root: string): Readonly<{ exitCode: number; stderr: string }> {
  const result = Bun.spawnSync(
    [
      process.execPath,
      join(import.meta.dir, "check-floating-sources.ts"),
      root,
    ],
    { stderr: "pipe", stdout: "pipe" },
  );
  return { exitCode: result.exitCode, stderr: result.stderr.toString() };
}

test("accepts exact sources and the explicit exceptions", () => {
  const root = createRepository(
    {
      "moon.yml": [
        "curl -fsSL -o theme https://raw.githubusercontent.com/o/r/6810349b28055dce54076712fc05fc68da4b8ec0/theme",
        "volta install pnpm@12.15.0",
        "brew install bat",
        "go install github.com/o/tool@v1.7.7",
        "cargo install --git https://github.com/o/r --rev 686734c85208af44c90201c3378e7f4c4c8b74dd --locked",
        "fisher install o/plugin@6a6136998879dcc1f29a405dfdd6b92c5f229c39",
        "runs-on: ubuntu-24.04",
        "os: [macos-26, ubuntu-24.04]",
      ].join("\n"),
    },
    ["bat"],
  );

  expect(check(root)).toEqual({ exitCode: 0, stderr: "" });
});

test.each([
  ["curl -fSL https://github.com/o/r/raw/main/theme", "branch reference"],
  [
    "curl -fsSL https://raw.githubusercontent.com/o/r/HEAD/install.sh",
    "branch reference",
  ],
  ["npx tool@latest", "latest release"],
  [
    "curl -L https://github.com/o/r/releases/latest/download/tool",
    "latest release",
  ],
  [
    "curl -sSf https://sh.example |\n  ENV=1 sh -s -- -y",
    "installer piped into a shell",
  ],
  ["volta install pnpm", "unversioned Volta package"],
  ["npm install -g tool", "unversioned npm global package"],
  ["bunx tool --help", "unversioned package runner"],
  ["cargo install tool", "unpinned Cargo installation"],
  ["go install github.com/o/tool", "unversioned Go installation"],
  ["fisher install o/plugin", "unversioned Fisher plugin"],
  ["git clone https://github.com/o/plugin", "Git clone without a revision"],
  ["    runs-on: ubuntu-latest", "floating CI runner label"],
  ["        os: [macos-latest, ubuntu-24.04]", "floating CI runner label"],
])("refuses %p", (line, rule) => {
  const result = check(createRepository({ "tasks.yml": `${line}\n` }));

  expect(result.exitCode).toBe(1);
  expect(result.stderr).toContain(`tasks.yml:1: ${rule}`);
});

test("refuses a Homebrew installation missing from the lock", () => {
  const result = check(
    createRepository({ "tasks.yml": "brew install --cask grok-bot\n" }, []),
  );

  expect(result.exitCode).toBe(1);
  expect(result.stderr).toContain(
    "Homebrew package grok-bot missing from homebrew.lock.json",
  );
});

test("an exception covers only its own file", () => {
  const result = check(
    createRepository({ "other.yml": "script: volta install @openai/codex\n" }),
  );

  expect(result.exitCode).toBe(1);
  expect(result.stderr).toContain("other.yml:1: unversioned Volta package");
});

test("refuses an exception that no longer matches its file", () => {
  const result = check(
    createRepository({
      ".moon/tasks/harness-codex.yml": "script: echo retired\n",
    }),
  );

  expect(result.exitCode).toBe(1);
  expect(result.stderr).toContain(
    "stale exception in .moon/tasks/harness-codex.yml",
  );
});

test("ignores documentation and test fixtures", () => {
  const root = createRepository({
    "README.md": "curl -fsSL https://example.test/install.sh | sh\n",
    "tooling/example.test.ts": '"npx tool@latest"\n',
  });

  expect(check(root)).toEqual({ exitCode: 0, stderr: "" });
});

test("ignores files that are not in the Git index", () => {
  const root = createRepository({});
  writeFileSync(join(root, "untracked.yml"), "volta install pnpm\n");

  expect(check(root)).toEqual({ exitCode: 0, stderr: "" });
});
