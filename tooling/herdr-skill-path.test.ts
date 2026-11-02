import { afterEach, expect, test } from "bun:test";
import {
  chmodSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";

const roots: string[] = [];
const executableMode = 0o755;
const skill =
  '---\nname: herdr\ndescription: "Official installed skill"\n---\n# Herdr\n';

afterEach(() => {
  for (const root of roots.splice(0)) {
    rmSync(root, { recursive: true, force: true });
  }
});

function fixture(): Readonly<{
  root: string;
  repository: string;
  prefix: string;
}> {
  const root = mkdtempSync(join(tmpdir(), "herdr-skill-path-"));
  roots.push(root);
  const repository = join(root, "repository");
  const prefix = join(root, "homebrew");
  mkdirSync(repository);
  mkdirSync(join(prefix, "bin"), { recursive: true });
  writeFileSync(
    join(prefix, "bin/herdr"),
    `#!${process.execPath}\nimport ${JSON.stringify(join(import.meta.dir, "herdr-skill-test-command.ts"))};\n`,
  );
  chmodSync(join(prefix, "bin/herdr"), executableMode);
  return { root, repository, prefix };
}

function generate(
  paths: ReturnType<typeof fixture>,
  environment: Readonly<Record<string, string>> = {},
): Bun.SyncSubprocess<"pipe", "pipe"> {
  return Bun.spawnSync(
    [
      process.execPath,
      join(import.meta.dir, "generate-herdr-skill.ts"),
      paths.repository,
    ],
    {
      cwd: paths.root,
      env: {
        ...process.env,
        PATH: "/usr/bin:/bin",
        HOMEBREW_PREFIX: paths.prefix,
        HERDR_SKILL: skill,
        ...environment,
      },
      stdout: "pipe",
      stderr: "pipe",
    },
  );
}

test("generates from the installed Homebrew prefix when its bin is absent from PATH", () => {
  const paths = fixture();
  const result = generate(paths);
  expect(result.stderr.toString()).toBe("");
  expect(result.exitCode).toBe(0);
  expect(
    readFileSync(
      join(paths.repository, "harness/generated/skills/herdr/SKILL.md"),
      "utf8",
    ),
  ).toBe(skill);
});

test("preserves PATH precedence over the Homebrew prefix", () => {
  const paths = fixture();
  const selected = join(paths.root, "selected");
  mkdirSync(selected);
  writeFileSync(
    join(selected, "herdr"),
    `#!${process.execPath}\nprocess.stderr.write("selected PATH binary");\nprocess.exit(1);\n`,
  );
  chmodSync(join(selected, "herdr"), executableMode);
  const result = generate(paths, { PATH: selected });
  expect(result.exitCode).not.toBe(0);
  expect(result.stderr.toString()).toContain("selected PATH binary");
});

test("does not fall back to a host installation when an explicit prefix is missing", () => {
  const paths = fixture();
  const result = generate(paths, {
    HOMEBREW_PREFIX: join(paths.root, "absent"),
  });
  expect(result.exitCode).not.toBe(0);
  expect(result.stderr.toString()).toContain("herdr not found");
});

test.each(["", "relative"])(
  "refuses a nonabsolute Homebrew prefix %j",
  (prefix) => {
    const paths = fixture();
    const result = generate(paths, { HOMEBREW_PREFIX: prefix });
    expect(result.exitCode).not.toBe(0);
    expect(result.stderr.toString()).toContain(
      "HOMEBREW_PREFIX must be absolute",
    );
  },
);
