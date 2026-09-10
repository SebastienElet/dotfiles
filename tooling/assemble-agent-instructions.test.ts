import { afterEach, expect, test } from "bun:test";
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  statSync,
  symlinkSync,
  utimesSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";

const fixtures: string[] = [];
const command = join(import.meta.dir, "assemble-agent-instructions.ts");

afterEach(() => {
  for (const root of fixtures.splice(0)) {
    rmSync(root, { recursive: true, force: true });
  }
});

function fixture(): Readonly<{
  source: string;
  destination: string;
  root: string;
}> {
  const root = mkdtempSync(join(tmpdir(), "moon-agent-instructions-"));
  fixtures.push(root);
  const source = join(root, "harness");
  const destination = join(root, ".codex/AGENTS.md");
  mkdirSync(source);
  writeFileSync(join(source, "AGENTS.md"), "@SOUL.md\n@USER.md\nRules\n");
  writeFileSync(join(source, "SOUL.md"), "Soul\n");
  writeFileSync(join(source, "USER.md"), "User\n");
  return { source, destination, root };
}

function run(
  paths: ReturnType<typeof fixture>,
): Bun.SyncSubprocess<"pipe", "pipe"> {
  return Bun.spawnSync(
    [process.execPath, command, paths.source, paths.destination],
    { stdout: "pipe", stderr: "pipe" },
  );
}

test("assembles imports and stays silent without rewriting on replay", () => {
  const paths = fixture();
  expect(run(paths).exitCode).toBe(0);
  expect(readFileSync(paths.destination, "utf8")).toBe("Rules\nSoul\nUser\n");
  utimesSync(paths.destination, new Date(0), new Date(0));
  const before = statSync(paths.destination);
  const replay = run(paths);
  expect(replay.exitCode).toBe(0);
  expect(replay.stdout.toString()).toBe("");
  expect(replay.stderr.toString()).toBe("");
  expect(statSync(paths.destination)).toMatchObject({
    ino: before.ino,
    mtimeMs: before.mtimeMs,
    ctimeMs: before.ctimeMs,
    mode: before.mode,
    size: before.size,
  });
  expect(readFileSync(paths.destination, "utf8")).toBe("Rules\nSoul\nUser\n");
});

test("replaces the output atomically without writing through a symlink", () => {
  const paths = fixture();
  const external = join(paths.root, "external");
  writeFileSync(external, "preserve\n");
  mkdirSync(join(paths.root, ".codex"));
  symlinkSync(external, paths.destination);
  expect(run(paths).exitCode).toBe(0);
  expect(readFileSync(external, "utf8")).toBe("preserve\n");
  expect(readFileSync(paths.destination, "utf8")).toBe("Rules\nSoul\nUser\n");
});

test("preserves the output when a source cannot be read", () => {
  const paths = fixture();
  mkdirSync(join(paths.root, ".codex"));
  writeFileSync(paths.destination, "keep\n");
  rmSync(join(paths.source, "USER.md"));
  expect(run(paths).exitCode).not.toBe(0);
  expect(readFileSync(paths.destination, "utf8")).toBe("keep\n");
});
