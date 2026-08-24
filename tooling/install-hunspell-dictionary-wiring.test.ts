import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";

const repository = dirname(import.meta.dir);

describe("Hunspell dictionary gates", () => {
  test("runs dictionary behavior on both deployment platforms", () => {
    const workflow = readFileSync(
      join(repository, ".github", "workflows", "test-deployment.yml"),
      "utf8",
    );

    expect(
      workflow.match(/      - tooling\/install-hunspell-dictionary\*/g),
    ).toHaveLength(2);
    expect(workflow).toContain("os: [macos-latest, ubuntu-latest]");
    expect(workflow).toContain(
      "bun test tooling/install-hunspell-dictionary.test.ts tooling/install-hunspell-dictionary-failures.test.ts",
    );
  });

  test("verifies installed dictionaries with real Hunspell", () => {
    const workflow = readFileSync(
      join(repository, ".github", "workflows", "test.yml"),
      "utf8",
    );

    expect(workflow).toContain("hunspell -D");
    expect(workflow).toContain("hunspell -d fr,en_US -l");
  });
});
