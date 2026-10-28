import { afterEach, expect, test } from "bun:test";
import {
  clearGateFixtures,
  executable,
  gateFixture,
  runGate,
} from "./gate-test-support.ts";
import {
  mkdirSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";

afterEach(clearGateFixtures);
const failureStatus = 31;

function fixture(): ReturnType<typeof gateFixture> {
  const result = gateFixture();
  for (const location of [".local/bin", ".volta/bin"]) {
    mkdirSync(join(result.home, location), { recursive: true });
  }
  for (const command of ["agent-handoff", "arnes", "claude"]) {
    executable(join(result.home, ".local/bin"), command, "true");
  }
  for (const command of ["codex", "node", "pnpm"]) {
    executable(join(result.home, ".volta/bin"), command, "true");
  }
  executable(
    result.bin,
    "brew",
    String.raw`if [ "$1" = --prefix ]; then printf "%s\n" "$GATE_FIXTURE"; fi`,
  );
  executable(result.bin, "bun", "true");
  executable(result.bin, "tar", 'printf "%s" snapshot');
  executable(
    result.bin,
    "moon",
    'if read -r answer; then exit 32; fi\nprintf "%s\\n" "$*" >> "$GATE_FIXTURE/moon-calls"',
  );
  return result;
}

test("closes stdin across the public profile lifecycle", () => {
  const context = fixture();
  const result = Bun.spawnSync(
    [process.execPath, join(import.meta.dir, "smoke-minimal.ts")],
    {
      cwd: context.root,
      env: {
        ...process.env,
        HOME: context.home,
        PATH: context.bin,
        GATE_FIXTURE: context.root,
      },
      stdin: Buffer.from("unexpected input\n"),
    },
  );
  expect(result.exitCode).toBe(0);
});

test.each(["moon", "brew", "bun", "tar"])("refuses %s failure", (command) => {
  const context = fixture();
  executable(
    context.bin,
    command,
    String.raw`printf "%s\n" command-stdout; printf "%s\n" rejected >&2; exit 31`,
  );
  const result = runGate("smoke-minimal.ts", context);
  expect(result.exitCode).toBe(failureStatus);
  expect(result.stderr.toString()).toContain("rejected");
  expect(result.stderr.toString()).toContain("Command failed:");
  expect(result.stderr.toString()).toContain(`"${command}"`);
  expect(result.stderr.toString()).toContain("stdout:\ncommand-stdout");
  expect(result.stderr.toString()).toContain("stderr:\nrejected");
});

function changingSnapshotFixture(change: string): ReturnType<typeof fixture> {
  const context = fixture();
  mkdirSync(join(context.home, ".claude"));
  executable(
    context.bin,
    "tar",
    'if [ "$1" = --no-recursion ]; then exec /usr/bin/tar "$@"; fi\nif [ -e "$GATE_FIXTURE/replayed" ]; then printf changed; else printf initial; fi',
  );
  executable(
    context.bin,
    "moon",
    `set -e\nif [ -e "$GATE_FIXTURE/ran" ]; then\n${change}\n: > "$GATE_FIXTURE/replayed"\nfi\n: > "$GATE_FIXTURE/ran"`,
  );
  return context;
}

test.each([
  {
    kind: "added",
    change: 'printf changed-content > "$HOME/.claude/observed"',
  },
  { kind: "removed", change: '/bin/rm "$HOME/.claude/observed"' },
  {
    kind: "modified",
    change: 'printf changed-content > "$HOME/.claude/observed"',
  },
  { kind: "modified", change: '/bin/chmod 600 "$HOME/.claude/observed"' },
  {
    kind: "modified",
    change: '/bin/ln -sf changed-target "$HOME/.claude/observed"',
    link: true,
  },
] as const)(
  "reports a $kind snapshot path after $change",
  ({
    kind,
    change,
    link,
  }: Readonly<{ kind: string; change: string; link?: boolean }>) => {
    const context = changingSnapshotFixture(change);
    const path = join(context.home, ".claude/observed");
    if (link === true) {
      symlinkSync("original-target", path);
    } else if (kind !== "added") {
      writeFileSync(path, "original-content", { mode: 0o644 });
    }
    const result = runGate("smoke-minimal.ts", context);
    expect(result.exitCode).not.toBe(0);
    expect(result.stderr.toString()).toContain(`${kind}: ".claude/observed"`);
    expect(result.stderr.toString()).toContain("Installed artifacts changed");
    expect(result.stderr.toString()).not.toContain("changed-content");
  },
);

test.each(["stdout", "stderr"])(
  "refuses repeat installation output on %s",
  (stream) => {
    const context = fixture();
    executable(
      context.bin,
      "moon",
      `if [ -e "$GATE_FIXTURE/ran" ]; then echo changed ${stream === "stderr" ? ">&2" : ""}; fi\n: > "$GATE_FIXTURE/ran"`,
    );
    expect(runGate("smoke-minimal.ts", context).exitCode).not.toBe(0);
  },
);

test("refuses a changed artifact snapshot", () => {
  const context = fixture();
  executable(
    context.bin,
    "tar",
    'if [ -e "$GATE_FIXTURE/snapshot" ]; then echo changed; else echo first; fi\n: > "$GATE_FIXTURE/snapshot"',
  );
  const result = runGate("smoke-minimal.ts", context);
  expect(result.exitCode).not.toBe(0);
  expect(result.stderr.toString()).toContain("artifacts changed");
});

test("refuses a missing installed executable", () => {
  const context = fixture();
  rmSync(join(context.home, ".local/bin/arnes"));
  expect(runGate("smoke-minimal.ts", context).exitCode).not.toBe(0);
});

test("propagates a failure of the repeated installation", () => {
  const context = fixture();
  executable(
    context.bin,
    "moon",
    'if [ -e "$GATE_FIXTURE/ran" ]; then echo rejected >&2; exit 31; fi\n: > "$GATE_FIXTURE/ran"',
  );
  const result = runGate("smoke-minimal.ts", context);
  expect(result.exitCode).toBe(failureStatus);
  expect(result.stderr.toString()).toContain("rejected");
});

test("refreshes the Homebrew lock from the runner catalog before installing", () => {
  const context = fixture();
  executable(
    context.bin,
    "bun",
    String.raw`printf "bun %s\n" "$*" >> "$GATE_FIXTURE/moon-calls"`,
  );
  expect(runGate("smoke-minimal.ts", context).exitCode).toBe(0);
  const calls = readFileSync(join(context.root, "moon-calls"), "utf8").split(
    "\n",
  );
  expect(calls[0]).toMatch(
    /^bun .*tooling\/homebrew-lock\.ts refresh .*homebrew\.lock\.json$/u,
  );
  expect(calls[1]).toBe("exec --quiet --ignore-ci-checks repository:install");
});
