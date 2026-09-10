import { afterEach, expect, test } from "bun:test";
import {
  clearGateFixtures,
  executable,
  gateFixture,
  runGate,
} from "./gate-test-support.ts";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

afterEach(clearGateFixtures);
const failureStatus = 31;

function fixture(): ReturnType<typeof gateFixture> {
  const result = gateFixture();
  writeFileSync(join(result.root, "home/cspell.json"), "{}\n");
  executable(
    result.bin,
    "git",
    String.raw`printf "harness/skills/example/SKILL.md\0"`,
  );
  executable(result.bin, "bun", 'shift 4; exec cspell "$@"');
  executable(
    result.bin,
    "moon",
    'printf "%s" "$HOME" > "$GATE_FIXTURE/deployed-home"',
  );
  executable(
    result.bin,
    "cspell",
    String.raw`if [ "$1" = trace ]; then printf "%s/.config/cspell/user.txt\n" "$HOME"; fi`,
  );
  return result;
}

test("checks the deployed user dictionary and removes its temporary home", () => {
  const context = fixture();
  const result = runGate("check-cspell.ts", context, ["home/cspell.json"]);
  expect(result.exitCode).toBe(0);
  expect(existsSync(join(context.root, "deployed-home"))).toBe(true);
  expect(
    existsSync(readFileSync(join(context.root, "deployed-home"), "utf8")),
  ).toBe(false);
});

test("refuses a missing user dictionary in CSpell trace", () => {
  const context = fixture();
  executable(context.bin, "cspell", String.raw`printf "%s\n" builtin`);
  const result = runGate("check-cspell.ts", context, ["home/cspell.json"]);
  expect(result.exitCode).not.toBe(0);
  expect(result.stderr.toString()).toContain("user dictionary");
});

test.each(["moon", "cspell"])("propagates %s failure", (command) => {
  const context = fixture();
  executable(
    context.bin,
    command,
    String.raw`printf "%s\n" rejected >&2; exit 31`,
  );
  const result = runGate("check-cspell.ts", context, ["home/cspell.json"]);
  expect(result.exitCode).toBe(failureStatus);
  expect(result.stderr.toString()).toContain("rejected");
});

test("refuses an empty lint selection", () => {
  expect(runGate("check-cspell.ts", fixture()).exitCode).not.toBe(0);
});

test("propagates a lint failure after resolving the dictionary", () => {
  const context = fixture();
  executable(
    context.bin,
    "cspell",
    String.raw`if [ "$1" = trace ]; then printf "%s/.config/cspell/user.txt\n" "$HOME"; else echo rejected >&2; exit 31; fi`,
  );
  const result = runGate("check-cspell.ts", context, ["home/cspell.json"]);
  expect(result.exitCode).toBe(failureStatus);
  expect(result.stderr.toString()).toContain("rejected");
});

test.each(["exit 17", "exit 0", String.raw`printf 'invalid'`])(
  "refuses unavailable or malformed Git discovery: %s",
  (body) => {
    const context = fixture();
    executable(context.bin, "git", body);
    const result = runGate("check-cspell.ts", context, ["home/cspell.json"]);
    expect(result.exitCode).not.toBe(0);
    expect(result.stderr.toString()).toContain(
      body === "exit 17" ? "git" : "Git",
    );
  },
);
