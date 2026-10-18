import { afterEach, expect, test } from "bun:test";
import {
  lstatSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readlinkSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";

const fixtures: string[] = [];
const command = join(import.meta.dir, "install-agent-skills.ts");

afterEach(() => {
  for (const root of fixtures.splice(0)) {
    rmSync(root, { force: true, recursive: true });
  }
});

function fixture(): Readonly<{ repository: string; home: string }> {
  const root = mkdtempSync(join(tmpdir(), "moon-agent-skills-"));
  fixtures.push(root);
  const repository = join(root, "repository");
  const home = join(root, "home");
  mkdirSync(join(repository, "home"), { recursive: true });
  mkdirSync(home);
  for (const slug of ["shared", "claude-only"]) {
    mkdirSync(join(repository, "harness/skills", slug), { recursive: true });
    writeFileSync(
      join(repository, "harness/skills", slug, "SKILL.md"),
      "skill\n",
    );
  }
  writeFileSync(
    join(repository, "home/.arnes.yaml"),
    Bun.YAML.stringify({
      version: 1,
      skills: [
        {
          slug: "shared",
          installations: [
            { agent: "codex", scope: "user" },
            { agent: "claude", scope: "user" },
          ],
        },
        {
          slug: "claude-only",
          installations: [
            { agent: "claude", scope: "user" },
            { agent: "codex", scope: "project" },
          ],
        },
      ],
    }),
  );
  return { repository, home };
}

function run(
  paths: ReturnType<typeof fixture>,
  agent = "codex",
): Bun.SyncSubprocess<"pipe", "pipe"> {
  return Bun.spawnSync(
    [process.execPath, command, paths.repository, paths.home, agent],
    { stdout: "pipe", stderr: "pipe" },
  );
}

test("deploys only selected user skills and preserves their identity on replay", () => {
  const paths = fixture();
  expect(run(paths).exitCode).toBe(0);
  const destination = join(paths.home, ".agents/skills/shared");
  expect(readlinkSync(destination)).toBe(
    join(paths.repository, "harness/skills/shared"),
  );
  expect(
    Bun.file(join(paths.home, ".agents/skills/claude-only/SKILL.md")).size,
  ).toBe(0);
  const before = lstatSync(destination);
  const replay = run(paths);
  expect(replay.exitCode).toBe(0);
  expect(replay.stdout.toString()).toBe("");
  expect(replay.stderr.toString()).toBe("");
  expect(lstatSync(destination)).toEqual(before);
});

test("preserves a conflicting user destination", () => {
  const paths = fixture();
  const destination = join(paths.home, ".agents/skills/shared");
  mkdirSync(join(paths.home, ".agents/skills"), { recursive: true });
  writeFileSync(destination, "keep\n");
  expect(run(paths).exitCode).not.toBe(0);
  expect(readFileSync(destination, "utf8")).toBe("keep\n");
});

test.each(["../../escape", "bad/slash"])(
  "rejects unsafe skill slug %s before deployment",
  (slug) => {
    const paths = fixture();
    writeFileSync(
      join(paths.repository, "home/.arnes.yaml"),
      Bun.YAML.stringify({
        version: 1,
        skills: [{ slug, installations: [{ agent: "codex", scope: "user" }] }],
      }),
    );
    expect(run(paths).exitCode).not.toBe(0);
    expect(
      Bun.file(join(paths.home, ".agents/skills/shared/SKILL.md")).size,
    ).toBe(0);
  },
);

test("rejects an unknown host and an unreadable manifest", () => {
  const paths = fixture();
  expect(run(paths, "unknown").exitCode).not.toBe(0);
  rmSync(join(paths.repository, "home/.arnes.yaml"));
  expect(run(paths).exitCode).not.toBe(0);
});

test("rejects missing skill sources before installing any links", () => {
  const paths = fixture();
  rmSync(join(paths.repository, "harness/skills/claude-only"), {
    recursive: true,
  });
  expect(run(paths, "claude").exitCode).not.toBe(0);
  expect(
    Bun.file(join(paths.home, ".claude/skills/shared/SKILL.md")).size,
  ).toBe(0);
});
