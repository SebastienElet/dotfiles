import { afterEach, expect, test } from "bun:test";
import {
  clearGateFixtures,
  executable,
  gateFixture,
  runGate,
} from "./gate-test-support.ts";
import { mkdirSync, rmSync } from "node:fs";
import type { ProfileOperations } from "./smoke-minimal.ts";
import { join } from "node:path";
import { smokeMinimalProfile } from "./smoke-minimal.ts";

afterEach(clearGateFixtures);
const failureStatus = 31;

function fixture(): ReturnType<typeof gateFixture> {
  const result = gateFixture();
  for (const location of [".local/bin", ".volta/bin"]) {
    mkdirSync(join(result.home, location), { recursive: true });
  }
  for (const command of ["agent-handoff", "agent-memory", "arnes", "claude"]) {
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

function profile(
  failure?: "clean" | "restore",
): Readonly<{ operations: ProfileOperations; restored: () => boolean }> {
  let installed = false;
  let cleaned = false;
  let restored = false;
  const output = { stdout: "", stderr: "" };
  return {
    operations: {
      install: () => {
        if (cleaned && !installed) {
          if (failure === "restore") {
            throw new Error("restoration failed");
          }
          restored = true;
        }
        installed = true;
        return output;
      },
      clean: () => {
        if (!installed) {
          throw new Error("cleanup requires an installed profile");
        }
        if (failure === "clean") {
          throw new Error("cleanup failed");
        }
        installed = false;
        cleaned = true;
        return output;
      },
      verify: () => {
        if (!installed) {
          throw new Error("profile is absent");
        }
      },
      snapshot: () => {
        if (!installed) {
          throw new Error("profile is absent");
        }
        return "installed identity";
      },
    },
    restored: () => restored,
  };
}

test("restores the profile after successful cleanup and verifies its replay", () => {
  const context = profile();
  smokeMinimalProfile(context.operations);
  expect(context.restored()).toBeTrue();
});

test.each(["clean", "restore"] as const)(
  "propagates %s failure without reporting restoration",
  (failure) => {
    const context = profile(failure);
    expect(() => {
      smokeMinimalProfile(context.operations);
    }).toThrow();
    expect(context.restored()).toBeFalse();
  },
);
