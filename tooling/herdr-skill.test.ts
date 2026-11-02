import { afterEach, expect, test } from "bun:test";
import {
  chmodSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  readlinkSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { dirname, join } from "node:path";
import { tmpdir } from "node:os";

const fixtures: string[] = [];
const executableMode = 0o755;
const timeoutTestMilliseconds = 15_000;
const concurrencyTestMilliseconds = 30_000;
const concurrencyRounds = 12;
const officialSkill =
  '---\nname: herdr\ndescription: "Control Herdr from its managed panes."\n---\n\n# Herdr\n';

afterEach(() => {
  for (const root of fixtures.splice(0)) {
    rmSync(root, { recursive: true, force: true });
  }
});

function fixture(): Readonly<{
  repository: string;
  home: string;
  bin: string;
  destination: string;
}> {
  const root = mkdtempSync(join(tmpdir(), "herdr-skill-"));
  fixtures.push(root);
  const repository = join(root, "repository");
  const home = join(root, "home");
  const bin = join(root, "bin");
  for (const path of [join(repository, "home"), home, bin]) {
    mkdirSync(path, { recursive: true });
  }
  writeFileSync(
    join(repository, "home/.arnes.yaml"),
    "version: 1\nskills:\n  - slug: herdr\n    source: harness/generated/skills/herdr\n    installations:\n      - {agent: claude, scope: user}\n      - {agent: codex, scope: user}\n      - {agent: cursor, scope: user}\n",
  );
  writeFileSync(
    join(bin, "herdr"),
    `#!${process.execPath}\nimport ${JSON.stringify(join(import.meta.dir, "herdr-skill-test-command.ts"))};\n`,
  );
  chmodSync(join(bin, "herdr"), executableMode);
  return {
    repository,
    home,
    bin,
    destination: join(repository, "harness/generated/skills/herdr/SKILL.md"),
  };
}

function install(
  paths: ReturnType<typeof fixture>,
  agent: string,
  output = officialSkill,
): Bun.SyncSubprocess<"pipe", "pipe"> {
  return Bun.spawnSync(
    [
      process.execPath,
      join(import.meta.dir, "install-agent-skills.ts"),
      paths.repository,
      paths.home,
      agent,
    ],
    {
      env: { ...process.env, PATH: paths.bin, HERDR_SKILL: output },
      stdout: "pipe",
      stderr: "pipe",
    },
  );
}

test.each([
  ["claude", ".claude/skills"],
  ["codex", ".agents/skills"],
  ["cursor", ".cursor/skills"],
])(
  "generates and deploys the binary's exact skill for %s",
  (agent, directory) => {
    const paths = fixture();
    const result = install(paths, agent);
    expect(result.stderr.toString()).toBe("");
    expect(result.exitCode).toBe(0);
    expect(readFileSync(paths.destination, "utf8")).toBe(officialSkill);
    expect(readlinkSync(join(paths.home, directory, "herdr"))).toBe(
      join(paths.repository, "harness/generated/skills/herdr"),
    );
    const updated = officialSkill.replace("# Herdr", "# Updated Herdr");
    expect(install(paths, agent, updated).exitCode).toBe(0);
    expect(
      readFileSync(join(paths.home, directory, "herdr/SKILL.md"), "utf8"),
    ).toBe(updated);
  },
);

test.each([
  "",
  `\uFEFF${officialSkill}`,
  "# No frontmatter\nname: herdr\n",
  "---\nname: herdr\ndescription: missing closing delimiter\n",
  "---\nname: foreign\ndescription: wrong skill\n---\n",
  "---\ndescription: missing name\n---\n",
  "---\nname: herdr\n---\n",
  "---\nname: herdr\ndescription: 1\n---\n",
  "---\nname: herdr\ndescription: '   '\n---\n",
  "---\nname: [herdr\n---\n",
  "---\n- name: herdr\n- description: wrong structure\n---\n",
  "preamble\n---\nname: herdr\ndescription: misplaced\n---\n",
])(
  "refuses invalid binary output %j without publishing or deploying it",
  (output) => {
    const paths = fixture();
    const rejected = install(paths, "codex", output);
    expect(rejected.exitCode).not.toBe(0);
    expect(rejected.stderr.toString()).not.toBe("");
    expect(existsSync(paths.destination)).toBe(false);
    expect(existsSync(join(paths.home, ".agents/skills/herdr"))).toBe(false);
    expect(install(paths, "codex").exitCode).toBe(0);
    expect(install(paths, "codex", output).exitCode).not.toBe(0);
    expect(readFileSync(paths.destination, "utf8")).toBe(officialSkill);
  },
);

test("refuses a missing binary even when an older generated skill exists", () => {
  const paths = fixture();
  expect(install(paths, "codex").exitCode).toBe(0);
  rmSync(join(paths.bin, "herdr"));
  const rejected = install(paths, "codex");
  expect(rejected.exitCode).not.toBe(0);
  expect(rejected.stderr.toString()).toContain("herdr not found");
  expect(readFileSync(paths.destination, "utf8")).toBe(officialSkill);
});

test.each([
  officialSkill.replaceAll("\n", "\r\n"),
  officialSkill.replace("name: herdr", 'name: "herdr"'),
])(
  "accepts YAML quoting and CRLF while preserving the binary output",
  (output) => {
    const paths = fixture();
    expect(install(paths, "codex", output).exitCode).toBe(0);
    expect(readFileSync(paths.destination, "utf8")).toBe(output);
  },
);

test("reports a failed publication and removes its staging file", () => {
  const paths = fixture();
  mkdirSync(paths.destination, { recursive: true });
  const result = install(paths, "codex");
  expect(result.exitCode).not.toBe(0);
  expect(readdirSync(join(paths.destination, ".."))).toEqual(["SKILL.md"]);
});

function generate(
  paths: ReturnType<typeof fixture>,
  environment: Readonly<Record<string, string>>,
): Bun.SyncSubprocess<"pipe", "pipe"> {
  return Bun.spawnSync(
    [
      process.execPath,
      join(import.meta.dir, "generate-herdr-skill.ts"),
      paths.repository,
    ],
    {
      cwd: paths.home,
      env: {
        ...process.env,
        PATH: paths.bin,
        HERDR_SKILL: officialSkill,
        ...environment,
      },
      stdout: "pipe",
      stderr: "pipe",
    },
  );
}

test.each([
  { HERDR_STATUS: "42", HERDR_ERROR: "upstream skill unavailable" },
  { HERDR_MODE: "invalid-utf8" },
])(
  "refuses subprocess failure %j through the generator entry point",
  (environment: Readonly<Record<string, string>>) => {
    const paths = fixture();
    expect(generate(paths, {}).exitCode).toBe(0);
    const result = generate(paths, environment);
    expect(result.exitCode).not.toBe(0);
    expect(result.stderr.toString()).not.toBe("");
    expect(readFileSync(paths.destination, "utf8")).toBe(officialSkill);
  },
);

test(
  "bounds a hung binary without replacing the generated skill",
  () => {
    const paths = fixture();
    expect(generate(paths, {}).exitCode).toBe(0);
    const result = generate(paths, { HERDR_MODE: "hang" });
    expect(result.exitCode).not.toBe(0);
    expect(result.stderr.toString()).toContain("herdr --skill failed");
    expect(readFileSync(paths.destination, "utf8")).toBe(officialSkill);
  },
  timeoutTestMilliseconds,
);

test.each([
  "harness",
  "harness/generated",
  "harness/generated/skills",
  "harness/generated/skills/herdr",
])(
  "refuses an escaping %s ancestor before writing outside the repository",
  (ancestor) => {
    const paths = fixture();
    const outside = join(paths.home, "outside");
    mkdirSync(outside);
    writeFileSync(join(outside, "keep"), "personal\n");
    const linked = join(paths.repository, ancestor);
    mkdirSync(dirname(linked), { recursive: true });
    symlinkSync(outside, linked);
    const result = generate(paths, {});
    expect(result.exitCode).not.toBe(0);
    expect(readdirSync(outside)).toEqual(["keep"]);
    expect(readFileSync(join(outside, "keep"), "utf8")).toBe("personal\n");
    expect(install(paths, "codex").exitCode).not.toBe(0);
    expect(readdirSync(outside)).toEqual(["keep"]);
  },
);

test("preserves a symbolic generated skill and its target", () => {
  const paths = fixture();
  const outside = join(paths.home, "personal.md");
  writeFileSync(outside, "personal\n");
  mkdirSync(dirname(paths.destination), { recursive: true });
  symlinkSync(outside, paths.destination);
  expect(generate(paths, {}).exitCode).not.toBe(0);
  expect(readlinkSync(paths.destination)).toBe(outside);
  expect(readFileSync(outside, "utf8")).toBe("personal\n");
});

test("allows an ancestor link whose target stays inside the repository", () => {
  const paths = fixture();
  const local = join(paths.repository, "local-harness");
  mkdirSync(local);
  symlinkSync(local, join(paths.repository, "harness"));
  expect(install(paths, "codex").exitCode).toBe(0);
  expect(readFileSync(paths.destination, "utf8")).toBe(officialSkill);
});

test(
  "deploys all three agents concurrently on a fresh generated source",
  async () => {
    for (let round = 0; round < concurrencyRounds; round += 1) {
      const paths = fixture();
      const barrier = join(paths.home, "barrier");
      mkdirSync(barrier);
      const children = ["claude", "codex", "cursor"].map((agent) =>
        Bun.spawn(
          [
            process.execPath,
            join(import.meta.dir, "install-agent-skills.ts"),
            paths.repository,
            paths.home,
            agent,
          ],
          {
            env: {
              ...process.env,
              PATH: paths.bin,
              HERDR_SKILL: officialSkill,
              HERDR_BARRIER: barrier,
            },
            stdout: "ignore",
            stderr: "pipe",
          },
        ),
      );
      for (const child of children) {
        const [status, stderr] = await Promise.all([
          child.exited,
          new Response(child.stderr).text(),
        ]);
        expect(stderr).toBe("");
        expect(status).toBe(0);
      }
      expect(readFileSync(paths.destination, "utf8")).toBe(officialSkill);
    }
  },
  concurrencyTestMilliseconds,
);
