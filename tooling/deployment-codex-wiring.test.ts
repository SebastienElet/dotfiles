import { afterEach, describe, expect, test } from "bun:test";
import {
  cleanupDeploymentFixtures,
  createDeploymentFixture,
  expectSuccess,
  project,
  runMake,
} from "./deployment-test-support.ts";
import { join } from "node:path";

afterEach(cleanupDeploymentFixtures);

describe("deployment area: Bun and hook wiring", () => {
  test("Bun consumers reuse the shared Homebrew target exactly once", () => {
    const fixture = createDeploymentFixture("bun-wiring");
    const brewBin = join(fixture.home, "homebrew", "bin");
    for (const target of ["claude-code", "codex", "obsidian-retrieval-test"]) {
      const result = runMake(fixture, [target], {
        dryRun: true,
        repository: project,
        variables: { BREW_BIN: brewBin },
      });
      expectSuccess(result);
      expect(count(result.stdout, "brew install bun")).toBe(1);
    }
  });

  test.each([
    ["codex", "codex", true],
    ["claude-code", "claude", true],
    ["cursor", "cursor", false],
  ] as const)(
    "%s entry point reconciles hooks once",
    (target, agent, deploysHandoff) => {
      const fixture = createDeploymentFixture(`hook-wiring-${target}`);
      const expected = `"${fixture.home}/.local/bin/arnes" setup hooks --agent ${agent}`;
      const hook = runMake(fixture, [`${target}-hooks`], {
        dryRun: true,
        repository: project,
      });
      expectSuccess(hook);
      expect(count(hook.stdout, expected)).toBe(1);
      expect(hook.stdout).toContain("cargo build --release");
      if (deploysHandoff) {
        expect(hook.stdout).toContain(
          join(fixture.home, ".local", "bin", "agent-handoff"),
        );
      }
      const entry = runMake(fixture, [target], {
        dryRun: true,
        repository: project,
      });
      expectSuccess(entry);
      expect(count(entry.stdout, expected)).toBe(1);
      if (target === "codex") {
        expect(entry.stdout).toContain(
          join(project, "harness", "skills", "agent-instructions"),
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
      dryRun: true,
      repository: project,
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
      const expected = `"${join(project, "tooling", "install-hunspell-dictionary")}" "${base}/${source}" "${checksum}" "${join(fixture.home, "Library", "Spelling", destination)}"`;
      expect(count(result.stdout, expected)).toBe(1);
    }
    const claude = runMake(fixture, ["claude-code"], {
      dryRun: true,
      repository: project,
      variables: { BREW_BIN: fixture.bin },
    });
    expectSuccess(claude);
    expect(claude.stdout).toContain("brew install hunspell");
  });
});

function count(value: string, needle: string): number {
  return value.split(needle).length - 1;
}
