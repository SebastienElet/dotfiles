import { afterEach, expect, test } from "bun:test";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { deployAgentMeasureHooks } from "./agent-measure-hooks.ts";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { z } from "zod";

const fixtures: string[] = [];
const executableMode = 0o755;
const commandSchema = z.object({ command: z.string() });
const groupSchema = z.object({ hooks: z.array(commandSchema) });
const claudeSchema = z.object({
  hooks: z.object({ PreToolUse: z.array(groupSchema) }),
});
const cursorSchema = z.object({
  hooks: z.object({ preToolUse: z.array(commandSchema) }),
});

afterEach(() => {
  for (const home of fixtures.splice(0)) {
    rmSync(home, { recursive: true, force: true });
  }
});

function collectorHome(): string {
  const home = mkdtempSync(join(tmpdir(), "measure-context-"));
  fixtures.push(home);
  mkdirSync(join(home, ".local/bin"), { recursive: true });
  writeFileSync(
    join(home, ".local/bin/agent-measure"),
    '#!/bin/sh\nprintf "%s\\n" "$@" > "$MEASURE_TEST_ARGUMENTS"\n',
    { mode: executableMode },
  );
  deployAgentMeasureHooks(home);
  return home;
}

function cursorInvocation(
  home: string,
  command: string,
): Bun.SyncSubprocess<"pipe", "pipe"> {
  return Bun.spawnSync(["/bin/sh", "-c", command], {
    env: {
      ...process.env,
      CURSOR_VERSION: "fixture-cursor",
      MEASURE_TEST_ARGUMENTS: join(home, "arguments"),
    },
  });
}

test("does not collect a Cursor-imported hook as a Claude run", () => {
  const home = collectorHome();
  const settings = claudeSchema.parse(
    JSON.parse(readFileSync(join(home, ".claude/settings.json"), "utf8")),
  );
  const command = settings.hooks.PreToolUse[0]?.hooks[0]?.command;
  if (command === undefined) {
    throw new Error("Claude command is missing");
  }

  const result = cursorInvocation(home, command);

  expect(result.exitCode).toBe(0);
  expect(existsSync(join(home, "arguments"))).toBe(false);
});

test("keeps the native Cursor collector active in Cursor", () => {
  const home = collectorHome();
  const settings = cursorSchema.parse(
    JSON.parse(readFileSync(join(home, ".cursor/hooks.json"), "utf8")),
  );
  const command = settings.hooks.preToolUse[0]?.command;
  if (command === undefined) {
    throw new Error("Cursor command is missing");
  }

  const result = cursorInvocation(home, command);

  expect(result.exitCode).toBe(0);
  expect(readFileSync(join(home, "arguments"), "utf8")).toBe(
    "hook\n--agent\ncursor\n",
  );
});
