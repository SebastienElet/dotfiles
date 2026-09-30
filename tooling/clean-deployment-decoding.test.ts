import { afterEach, expect, test } from "bun:test";
import {
  cleanupDeploymentFixtures,
  createDeploymentFixture,
  runDeploymentHelper,
} from "./deployment-test-support.ts";
import { dirname, join } from "node:path";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";

afterEach(cleanupDeploymentFixtures);
type Fixture = ReturnType<typeof createDeploymentFixture>;

function fixture(): Fixture {
  const context = createDeploymentFixture("clean-decoding");
  mkdirSync(join(context.repository, "home"));
  writeFileSync(
    join(context.repository, "home/.arnes.yaml"),
    "version: 1\nskills: []\n",
  );
  return context;
}

function write(
  context: Fixture,
  relativePath: string,
  contents: string,
): string {
  const path = join(context.home, relativePath);
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, contents);
  return path;
}

function clean(context: Fixture): ReturnType<typeof runDeploymentHelper> {
  return runDeploymentHelper(context, {
    helper: "clean-deployment.ts",
    arguments: [context.repository, context.home, "--apply"],
  });
}

test.each([
  "1e-400",
  "9007199254740991.1",
  "0.12345678901234567890123456789",
  "1e400",
  "-0",
  "0.1000",
  "1E+003",
])(
  "preserves the unrelated numeric lexeme %s while removing its neighbor hook",
  (lexeme) => {
    const context = fixture();
    const command = `'${context.home}/.local/bin/arnes' measure hook --agent claude-code`;
    const original = `{"private":${lexeme},"hooks":{"Stop":[{"hooks":[{"type":"command","command":${JSON.stringify(command)}}]}]}}`;
    const path = write(context, ".claude/settings.json", original);
    expect(clean(context).exitCode).toBe(0);
    const updated = readFileSync(path, "utf8");
    expect(updated).toContain(`"private": ${lexeme}`);
    expect(updated).not.toContain(command);
  },
);

test.each([".claude/settings.json", ".codex/config.toml"])(
  "refuses invalid UTF-8 bytes in %s before mutating any deployment",
  (relativePath) => {
    const context = fixture();
    const prefix = relativePath.endsWith(".json")
      ? '{"private":"value-'
      : 'private = "value-';
    const suffix = relativePath.endsWith(".json") ? '","hooks":{}}' : '"\n';
    const invalidUtf8Byte = 0xff;
    const original = Buffer.concat([
      Buffer.from(prefix),
      Buffer.from([invalidUtf8Byte]),
      Buffer.from(suffix),
    ]);
    const path = write(context, relativePath, "");
    writeFileSync(path, original);
    const generated = write(context, ".codex/AGENTS.md", "keep\n");
    expect(clean(context).exitCode).not.toBe(0);
    expect(readFileSync(path)).toEqual(original);
    expect(readFileSync(generated, "utf8")).toBe("keep\n");
  },
);
