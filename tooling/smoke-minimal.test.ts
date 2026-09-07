import { afterEach, expect, test } from "bun:test";
import {
  clearGateFixtures,
  executable,
  gateFixture,
  runGate,
} from "./gate-test-support.ts";
import { mkdirSync, readFileSync } from "node:fs";
import { join } from "node:path";

afterEach(clearGateFixtures);
const failureStatus = 31;

function fixture(): ReturnType<typeof gateFixture> {
  const result = gateFixture();
  for (const location of [".local/bin", ".volta/bin"]) {
    mkdirSync(join(result.home, location), { recursive: true });
  }
  for (const command of [
    "agent-handoff",
    "agent-memory",
    "arnes",
    "claude",
    "colgrep-search",
  ]) {
    executable(join(result.home, ".local/bin"), command, "true");
  }
  for (const command of ["codex", "node", "pnpm"]) {
    executable(join(result.home, ".volta/bin"), command, "true");
  }
  executable(result.bin, "colgrep", "true");
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

test("runs the public install twice with closed stdin", () => {
  const context = fixture();
  const result = runGate("smoke-minimal.ts", context);
  expect(result.exitCode).toBe(0);
  expect(readFileSync(join(context.root, "moon-calls"), "utf8")).toBe(
    "exec --quiet --ignore-ci-checks repository:install\nexec --quiet --ignore-ci-checks repository:install\n",
  );
});

test.each(["moon", "brew", "bun", "tar"])("refuses %s failure", (command) => {
  const context = fixture();
  executable(
    context.bin,
    command,
    String.raw`printf "%s\n" rejected >&2; exit 31`,
  );
  const result = runGate("smoke-minimal.ts", context);
  expect(result.exitCode).toBe(failureStatus);
  expect(result.stderr.toString()).toContain("rejected");
});

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
  executable(
    context.bin,
    "brew",
    'if [ "$1" = --prefix ]; then echo /missing; fi',
  );
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
