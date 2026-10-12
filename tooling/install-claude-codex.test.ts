import { afterEach, expect, test } from "bun:test";
import {
  clearGateFixtures,
  executable,
  gateFixture,
  runGate,
} from "./gate-test-support.ts";
import { readFileSync, statSync, writeFileSync } from "node:fs";
import { join } from "node:path";

afterEach(clearGateFixtures);
const failureExitCode = 31;

const installed = {
  id: "codex@openai-codex",
  scope: "user",
  enabled: true,
};

function fixture(list: string): ReturnType<typeof gateFixture> {
  const context = gateFixture();
  writeFileSync(join(context.root, "list.json"), list);
  writeFileSync(
    join(context.root, "after-install.json"),
    JSON.stringify([installed]),
  );
  writeFileSync(join(context.root, "failure"), "");
  executable(
    context.bin,
    "claude",
    `exec "${process.execPath}" "${join(import.meta.dir, "install-claude-codex-test-provider.ts")}" "$@"`,
  );
  return context;
}

test("preserves an installed user companion silently on replay", () => {
  const context = fixture(JSON.stringify([installed]));
  const path = join(context.root, "list.json");
  const before = statSync(path);
  const result = runGate("install-claude-codex.ts", context);
  expect(result.exitCode).toBe(0);
  expect(result.stdout.toString()).toBe("");
  expect(result.stderr.toString()).toBe("");
  expect(readFileSync(path, "utf8")).toBe(JSON.stringify([installed]));
  expect(statSync(path).mtimeMs).toBe(before.mtimeMs);
});

test.each(["[]", JSON.stringify([{ ...installed, scope: "project" }])])(
  "installs the missing user companion and permits silent replay",
  (list) => {
    const context = fixture(list);
    const first = runGate("install-claude-codex.ts", context);
    expect(first.exitCode).toBe(0);
    expect(first.stdout.toString()).toContain("Plugin installed");
    const repeat = runGate("install-claude-codex.ts", context);
    expect(repeat.exitCode).toBe(0);
    expect(repeat.stdout.toString()).toBe("");
    expect(repeat.stderr.toString()).toBe("");
  },
);

test.each(["list", "install"])("propagates Claude %s failure", (operation) => {
  const context = fixture("[]");
  writeFileSync(join(context.root, "failure"), operation);
  const result = runGate("install-claude-codex.ts", context);
  expect(result.exitCode).toBe(failureExitCode);
  expect(result.stderr.toString()).toContain("Claude command rejected");
});

test.each(["", "null", '{"plugins":[]}', '[{"id":"codex@openai-codex"}]'])(
  "rejects malformed plugin inventory without overwriting it: %s",
  (list) => {
    const context = fixture(list);
    expect(runGate("install-claude-codex.ts", context).exitCode).not.toBe(0);
    expect(readFileSync(join(context.root, "list.json"), "utf8")).toBe(list);
  },
);

test("preserves an explicitly disabled user companion and reports failure", () => {
  const list = JSON.stringify([{ ...installed, enabled: false }]);
  const context = fixture(list);
  const result = runGate("install-claude-codex.ts", context);
  expect(result.exitCode).not.toBe(0);
  expect(result.stderr.toString()).toContain("disabled");
  expect(readFileSync(join(context.root, "list.json"), "utf8")).toBe(list);
});

test.each(["[]", JSON.stringify([{ ...installed, enabled: false }]), "null"])(
  "rejects installation without an enabled user companion: %s",
  (list) => {
    const context = fixture("[]");
    writeFileSync(join(context.root, "after-install.json"), list);
    expect(runGate("install-claude-codex.ts", context).exitCode).not.toBe(0);
  },
);
