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
  writeFileSync(
    join(source, "AGENTS.md"),
    "@SOUL.md\n@USER.md\n@visual-presentation.md\nRules\n",
  );
  writeFileSync(join(source, "SOUL.md"), "Soul\n");
  writeFileSync(join(source, "USER.md"), "User\n");
  writeFileSync(join(source, "visual-presentation.md"), "Visual\n");
  return { source, destination, root };
}

function run(
  paths: ReturnType<typeof fixture>,
  format?: string,
): Bun.SyncSubprocess<"pipe", "pipe"> {
  return Bun.spawnSync(
    [
      process.execPath,
      command,
      paths.source,
      paths.destination,
      ...(format === undefined ? [] : [format]),
    ],
    { stdout: "pipe", stderr: "pipe" },
  );
}

test("assembles an always-applied Cursor rule from the common sources", () => {
  const paths = fixture();
  const result = run(paths, "cursor-rule");
  expect(result.exitCode).toBe(0);
  expect(readFileSync(paths.destination, "utf8")).toBe(
    "---\ndescription: Common agent instructions, persona and preferences\nalwaysApply: true\n---\nRules\nSoul\nUser\nVisual\n",
  );
  utimesSync(paths.destination, new Date(0), new Date(0));
  const before = statSync(paths.destination);
  expect(run(paths, "cursor-rule").exitCode).toBe(0);
  expect(statSync(paths.destination).mtimeMs).toBe(before.mtimeMs);
});

test("rejects an unknown output format without changing the destination", () => {
  const paths = fixture();
  mkdirSync(join(paths.root, ".codex"));
  writeFileSync(paths.destination, "keep\n");
  expect(run(paths, "unknown").exitCode).not.toBe(0);
  expect(readFileSync(paths.destination, "utf8")).toBe("keep\n");
});

test("assembles imports and stays silent without rewriting on replay", () => {
  const paths = fixture();
  expect(run(paths).exitCode).toBe(0);
  expect(readFileSync(paths.destination, "utf8")).toBe(
    "Rules\nSoul\nUser\nVisual\n",
  );
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
  expect(readFileSync(paths.destination, "utf8")).toBe(
    "Rules\nSoul\nUser\nVisual\n",
  );
});

test.each(["markdown", "cursor-rule"])(
  "replaces %s output without writing through a symlink",
  (format) => {
    const paths = fixture();
    const external = join(paths.root, "external");
    writeFileSync(external, "preserve\n");
    mkdirSync(join(paths.root, ".codex"));
    symlinkSync(external, paths.destination);
    expect(run(paths, format).exitCode).toBe(0);
    expect(readFileSync(external, "utf8")).toBe("preserve\n");
    expect(
      readFileSync(paths.destination, "utf8").endsWith(
        "Rules\nSoul\nUser\nVisual\n",
      ),
    ).toBeTrue();
  },
);

test.each(["markdown", "cursor-rule"])(
  "preserves %s output when a source cannot be read",
  (format) => {
    const paths = fixture();
    mkdirSync(join(paths.root, ".codex"));
    writeFileSync(paths.destination, "keep\n");
    rmSync(join(paths.source, "USER.md"));
    expect(run(paths, format).exitCode).not.toBe(0);
    expect(readFileSync(paths.destination, "utf8")).toBe("keep\n");
  },
);

test.each(["markdown", "cursor-rule"])(
  "preserves %s output when visual preferences cannot be read",
  (format) => {
    const paths = fixture();
    mkdirSync(join(paths.root, ".codex"));
    writeFileSync(paths.destination, "keep\n");
    rmSync(join(paths.source, "visual-presentation.md"));
    expect(run(paths, format).exitCode).not.toBe(0);
    expect(readFileSync(paths.destination, "utf8")).toBe("keep\n");
  },
);
