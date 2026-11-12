import { afterEach, expect, test } from "bun:test";
import {
  lstatSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  utimesSync,
  writeFileSync,
} from "node:fs";
import { deployAgentMeasureHooks } from "./agent-measure-hooks.ts";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { z } from "zod";

const fixtures: string[] = [];

afterEach(() => {
  for (const directory of fixtures.splice(0)) {
    rmSync(directory, { force: true, recursive: true });
  }
});

function fixtureHome(): string {
  const directory = mkdtempSync(join(tmpdir(), "measure-hooks-"));
  fixtures.push(directory);
  return directory;
}

function writeSettings(home: string, relative: string, value: unknown): void {
  const destination = join(home, relative);
  mkdirSync(join(destination, ".."), { recursive: true });
  writeFileSync(destination, JSON.stringify(value));
}

function readSettings(home: string, relative: string): unknown {
  return JSON.parse(readFileSync(join(home, relative), "utf8"));
}

test("adds measurement while preserving personal settings and unrelated hooks", () => {
  const home = fixtureHome();
  const measurementCommand: unknown = expect.stringContaining(
    `'${home}/.local/bin/agent-measure' hook --agent claude-code`,
  );
  const notification = { type: "command", command: "notify-user" };
  writeSettings(home, ".claude/settings.json", {
    model: "personal-model",
    permissions: { allow: ["Read"] },
    hooks: {
      PreToolUse: [{ matcher: "Read", hooks: [notification] }],
    },
  });

  deployAgentMeasureHooks(home);

  expect(readSettings(home, ".claude/settings.json")).toMatchObject({
    model: "personal-model",
    permissions: { allow: ["Read"] },
    hooks: {
      PreToolUse: [
        { matcher: "Read", hooks: [notification] },
        {
          hooks: [
            {
              type: "command",
              command: measurementCommand,
            },
          ],
        },
      ],
    },
  });
});

test("replaces duplicate measurement entries and preserves mixed hook groups", () => {
  const home = fixtureHome();
  const command = `'${home}/.local/bin/agent-measure' hook --agent claude-code`;
  const measurementCommand: unknown = expect.stringContaining(command);
  const notification = { type: "command", command: "notify-user" };
  writeSettings(home, ".claude/settings.json", {
    hooks: {
      PreToolUse: [
        {
          matcher: "Read",
          hooks: [notification, { type: "command", command }],
        },
        { hooks: [{ type: "command", command }] },
      ],
      Notification: [{ hooks: [{ type: "command", command }] }],
    },
  });

  deployAgentMeasureHooks(home);

  expect(readSettings(home, ".claude/settings.json")).toMatchObject({
    hooks: {
      PreToolUse: [
        { matcher: "Read", hooks: [notification] },
        {
          hooks: [{ type: "command", command: measurementCommand }],
        },
      ],
    },
  });
  expect(readSettings(home, ".claude/settings.json")).not.toHaveProperty(
    "hooks.Notification",
  );
});

test("replays without rewriting or adding another collector", () => {
  const home = fixtureHome();
  deployAgentMeasureHooks(home);
  const path = join(home, ".claude/settings.json");
  utimesSync(path, new Date(0), new Date(0));
  const before = readFileSync(path, "utf8");
  const metadata = lstatSync(path);

  deployAgentMeasureHooks(home);

  expect(readFileSync(path, "utf8")).toBe(before);
  expect(lstatSync(path).mtimeMs).toBe(metadata.mtimeMs);
  expect(lstatSync(path).ino).toBe(metadata.ino);
});

test("keeps external formatting when the measurement hooks are already current", () => {
  const home = fixtureHome();
  deployAgentMeasureHooks(home);
  const path = join(home, ".claude/settings.json");
  const settings = readSettings(home, ".claude/settings.json");
  const compact = JSON.stringify(settings);
  writeFileSync(path, compact);
  const metadata = lstatSync(path);

  deployAgentMeasureHooks(home);

  expect(readFileSync(path, "utf8")).toBe(compact);
  expect(lstatSync(path).ino).toBe(metadata.ino);
});

test("preserves Cursor settings, third-party commands and unknown events", () => {
  const home = fixtureHome();
  const foreignHook = { command: "printf agent-measure", timeout: 30 };
  writeSettings(home, ".cursor/hooks.json", {
    version: 1,
    personal: { enabled: true },
    hooks: {
      preToolUse: [foreignHook],
      futureEvent: [{ command: "custom-hook" }],
    },
  });

  deployAgentMeasureHooks(home);

  expect(readSettings(home, ".cursor/hooks.json")).toMatchObject({
    version: 1,
    personal: { enabled: true },
    hooks: {
      preToolUse: [
        foreignHook,
        { command: `'${home}/.local/bin/agent-measure' hook --agent cursor` },
      ],
      futureEvent: [{ command: "custom-hook" }],
    },
  });
});

test("preserves special JSON property names without changing prototypes", () => {
  const home = fixtureHome();
  const command = `'${home}/.local/bin/agent-measure' hook --agent claude-code`;
  const group: unknown = Object.fromEntries([
    ["__proto__", { keep: "group" }],
    ["hooks", [{ command: "notify-user" }, { type: "command", command }]],
  ]);
  const hooks: unknown = Object.fromEntries([
    ["__proto__", [{ command: "custom-hook" }]],
    ["PreToolUse", [group]],
  ]);
  const settings: unknown = Object.fromEntries([
    ["__proto__", { keep: "root" }],
    ["hooks", hooks],
  ]);
  writeSettings(home, ".claude/settings.json", settings);

  deployAgentMeasureHooks(home);

  const result = readSettings(home, ".claude/settings.json");
  expect(result).toHaveProperty(["__proto__", "keep"], "root");
  expect(result).toHaveProperty(
    ["hooks", "__proto__"],
    [{ command: "custom-hook" }],
  );
  expect(result).toHaveProperty(
    ["hooks", "PreToolUse", "0", "__proto__", "keep"],
    "group",
  );
});

test("runs the deployed command when the home path contains quotes and spaces", () => {
  const home = join(fixtureHome(), "home ' and spaces");
  const executable = join(home, ".local/bin/agent-measure");
  const argumentsPath = join(home, "observed-arguments");
  const executableMode = 0o755;
  mkdirSync(join(executable, ".."), { recursive: true });
  writeFileSync(
    executable,
    '#!/bin/sh\nprintf "%s\\n" "$@" > "$MEASURE_TEST_ARGUMENTS"\n',
    { mode: executableMode },
  );
  deployAgentMeasureHooks(home);
  const commandSchema = z.object({ command: z.string() });
  const groupSchema = z.object({ hooks: z.array(commandSchema) });
  const eventsSchema = z.object({ PreToolUse: z.array(groupSchema) });
  const settings = z
    .object({ hooks: eventsSchema })
    .parse(readSettings(home, ".claude/settings.json"));
  const command = settings.hooks.PreToolUse[0]?.hooks[0]?.command;
  expect(command).toBeDefined();
  if (command === undefined) {
    throw new Error("Measurement command is missing");
  }

  const result = Bun.spawnSync(["/bin/sh", "-c", command], {
    env: {
      ...process.env,
      CURSOR_VERSION: "",
      MEASURE_TEST_ARGUMENTS: argumentsPath,
    },
  });

  expect(result.exitCode).toBe(0);
  expect(readFileSync(argumentsPath, "utf8")).toBe(
    "hook\n--agent\nclaude-code\n",
  );
});
