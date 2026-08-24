import { afterEach, describe, expect, test } from "bun:test";
import {
  lstatSync,
  mkdirSync,
  readFileSync,
  rmSync,
  symlinkSync,
  unlinkSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import {
  cleanupDeploymentFixtures,
  createDeploymentFixture,
  expectSuccess,
  fileIdentity,
  linkTarget,
  project,
  requireCommand,
  runMake,
} from "./deployment-test-support.ts";

afterEach(cleanupDeploymentFixtures);

describe("deployment area: Codex handoff hook", () => {
  test.each([
    ["migrates an old hook", ["old"]],
    ["collapses old and current hooks", ["old", "new"]],
    ["collapses duplicate old hooks", ["old", "old"]],
  ] as const)("%s", (_, initialCommands) => {
    const fixture = createDeploymentFixture("codex-hook-migration");
    const hooks = join(fixture.home, ".codex", "hooks.json");
    const oldCommand = join(project, "scripts", "agent_handoff");
    const newCommand = join(project, "tooling", "agent-handoff");
    mkdirSync(join(fixture.home, ".codex"), { recursive: true });
    writeHooks(
      hooks,
      initialCommands.map((command) =>
        command === "old" ? oldCommand : newCommand,
      ),
    );
    expectSuccess(
      runMake(fixture, ["codex-handoff-hook"], { repository: project }),
    );
    expect(commands(hooks)).toEqual([newCommand]);
  });

  test("replays idempotently", () => {
    const fixture = createDeploymentFixture("codex-hook-replay");
    const hooks = join(fixture.home, ".codex", "hooks.json");
    const newCommand = join(project, "tooling", "agent-handoff");
    mkdirSync(join(fixture.home, ".codex"), { recursive: true });
    writeHooks(hooks, [newCommand]);
    expectSuccess(
      runMake(fixture, ["codex-handoff-hook"], { repository: project }),
    );
    const before = readFileSync(hooks, "utf8");
    expectSuccess(
      runMake(fixture, ["codex-handoff-hook"], { repository: project }),
    );
    expect(readFileSync(hooks, "utf8")).toBe(before);
  });

  test.each([
    "invalid json\n",
    '{"hooks":{"Stop":{}}}\n',
    "null\n",
    "false\n",
    '{"hooks":{"Stop":false}}\n',
    '{"hooks":{"Stop":[]}}\n{"second":"document"}\n',
    "",
  ])("preserves malformed hooks %#", (content) => {
    const fixture = createDeploymentFixture("codex-invalid");
    const hooks = join(fixture.home, ".codex", "hooks.json");
    mkdirSync(join(fixture.home, ".codex"), { recursive: true });
    writeFileSync(hooks, content);
    const before = fileIdentity(hooks);
    const result = runMake(fixture, ["codex-handoff-hook"], {
      repository: project,
    });
    expect(result.exitCode).not.toBe(0);
    expect(fileIdentity(hooks)).toEqual(before);
  });

  test("preserves hooks when jq cannot be started", () => {
    const fixture = createDeploymentFixture("codex-no-jq");
    const hooks = join(fixture.home, ".codex", "hooks.json");
    mkdirSync(join(fixture.home, ".codex"), { recursive: true });
    writeHooks(hooks, []);
    const before = fileIdentity(hooks);
    const result = runMake(fixture, ["codex-handoff-hook"], {
      repository: project,
      make: "/usr/bin/make",
      environment: { PATH: join(fixture.root, "empty-path") },
    });
    expect(result.exitCode).not.toBe(0);
    expect(fileIdentity(hooks)).toEqual(before);
  });

  test("refuses non-files and preserves a predictable temporary symlink", () => {
    const fixture = createDeploymentFixture("codex-paths");
    const hooks = join(fixture.home, ".codex", "hooks.json");
    mkdirSync(join(fixture.home, ".codex"), { recursive: true });
    mkdirSync(hooks);
    expect(
      runMake(fixture, ["codex-handoff-hook"], { repository: project })
        .exitCode,
    ).not.toBe(0);
    expect(lstatSync(hooks).isDirectory()).toBeTrue();
    rmSync(hooks, { recursive: true });
    const fifo = Bun.spawnSync([requireCommand("mkfifo"), hooks]);
    expect(fifo.exitCode).toBe(0);
    expect(
      runMake(fixture, ["codex-handoff-hook"], { repository: project })
        .exitCode,
    ).not.toBe(0);
    expect(lstatSync(hooks).isFIFO()).toBeTrue();
    unlinkSync(hooks);

    const victim = join(fixture.root, "victim");
    const predictable = `${hooks}.tmp`;
    writeFileSync(victim, "keep\n");
    symlinkSync(victim, predictable);
    expectSuccess(
      runMake(fixture, ["codex-handoff-hook"], { repository: project }),
    );
    expect(readFileSync(victim, "utf8")).toBe("keep\n");
    expect(linkTarget(predictable)).toBe(victim);
  });
});

describe("deployment area: Bun and measurement wiring", () => {
  test("Bun consumers reuse the shared Homebrew target exactly once", () => {
    const fixture = createDeploymentFixture("bun-wiring");
    const brewBin = join(fixture.home, "homebrew", "bin");
    for (const target of ["claude-code", "codex", "obsidian-retrieval-test"]) {
      const result = runMake(fixture, [target], {
        repository: project,
        dryRun: true,
        variables: { BREW_BIN: brewBin },
      });
      expectSuccess(result);
      expect(count(result.stdout, "brew install bun")).toBe(1);
    }
  });

  test.each(["codex", "claude-code", "cursor"])(
    "%s entry point installs one measurement hook",
    (agent) => {
      const fixture = createDeploymentFixture(`measurement-wiring-${agent}`);
      const expected = `measure install-hooks --agent ${agent} --command \"${fixture.home}/.local/bin/arnes\"`;
      const hook = runMake(fixture, [`${agent}-measurement-hooks`], {
        repository: project,
        dryRun: true,
      });
      expectSuccess(hook);
      expect(count(hook.stdout, expected)).toBe(1);
      expect(hook.stdout).toContain("cargo build --release");
      const entry = runMake(fixture, [agent], {
        repository: project,
        dryRun: true,
      });
      expectSuccess(entry);
      expect(count(entry.stdout, expected)).toBe(1);
      if (agent === "codex") {
        expect(entry.stdout).toContain(
          join(project, "harness", "skills", "agent-instructions"),
        );
        expect(entry.stdout.indexOf("old_command=")).toBeLessThan(
          entry.stdout.indexOf(expected),
        );
      }
    },
  );
});

describe("deployment area: Hunspell wiring", () => {
  test("installs the formula, runtime, four pinned dictionaries, and Claude dependency", () => {
    const fixture = createDeploymentFixture("hunspell-wiring");
    const revision = "f2ff99058268502bdcf4cad25c1ca2935ad8aa7d";
    const base = `https://raw.githubusercontent.com/LibreOffice/dictionaries/${revision}`;
    const result = runMake(fixture, ["hunspell"], {
      repository: project,
      dryRun: true,
      variables: { BREW_BIN: fixture.bin },
    });
    expectSuccess(result);
    expect(count(result.stdout, "brew install hunspell")).toBe(1);
    expect(count(result.stdout, "brew install bun")).toBe(1);
    for (const [source, checksum, destination] of [
      [
        "fr_FR/dictionaries/fr.aff",
        "c176610cd5dc4846806a65ddd029f422d87978bf58f224aa44222662a16a2de5",
        "fr.aff",
      ],
      [
        "fr_FR/dictionaries/fr.dic",
        "b78a868e31dd6e373b6c3217969afb898a9acde828a5e7ef97308da42218c88c",
        "fr.dic",
      ],
      [
        "en/en_US.aff",
        "e746c882dd6f303c2c46e7452804b9201115a6942cfeb15f18f8edf774d2e24e",
        "en_US.aff",
      ],
      [
        "en/en_US.dic",
        "f0b1a234bd178bdd01875b2a392a9647f888b8fe879f79c52aae62c2759b3647",
        "en_US.dic",
      ],
    ] as const) {
      const expected = `\"${join(project, "tooling", "install-hunspell-dictionary")}\" \"${base}/${source}\" \"${checksum}\" \"${join(fixture.home, "Library", "Spelling", destination)}\"`;
      expect(count(result.stdout, expected)).toBe(1);
    }
    const claude = runMake(fixture, ["claude-code"], {
      repository: project,
      dryRun: true,
      variables: { BREW_BIN: fixture.bin },
    });
    expectSuccess(claude);
    expect(claude.stdout).toContain("brew install hunspell");
  });
});

function writeHooks(path: string, hookCommands: readonly string[]): void {
  writeFileSync(
    path,
    `${JSON.stringify({ hooks: { Stop: [{ hooks: hookCommands.map((command) => ({ type: "command", command })) }] } })}\n`,
  );
}

function commands(path: string): string[] {
  const parsed = JSON.parse(readFileSync(path, "utf8")) as {
    hooks: { Stop: { hooks: { command: string }[] }[] };
  };
  return parsed.hooks.Stop.flatMap(({ hooks }) =>
    hooks.map(({ command }) => command),
  );
}

function count(value: string, needle: string): number {
  return value.split(needle).length - 1;
}
