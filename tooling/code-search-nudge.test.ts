import {
  type HookToolCall,
  initialNudgeState,
  nextNudgeTurn,
  parseStoredState,
  searchThreshold,
} from "./code-search-nudge.ts";
import { expect, test } from "bun:test";

const belowThreshold = searchThreshold - 1;
const persistentSearches = 12;

function replay(
  calls: readonly HookToolCall[],
): readonly (string | undefined)[] {
  let state = initialNudgeState;
  const emitted: (string | undefined)[] = [];
  for (const call of calls) {
    const turn = nextNudgeTurn(state, call);
    ({ state } = turn);
    emitted.push(turn.additionalContext);
  }
  return emitted;
}

function grep(times: number): readonly HookToolCall[] {
  return Array.from({ length: times }, () => ({
    toolInput: { pattern: "whatever" },
    toolName: "Grep",
  }));
}

function bash(commands: readonly string[]): readonly HookToolCall[] {
  return commands.map((command) => ({
    toolInput: { command },
    toolName: "Bash",
  }));
}

function repeated(command: string, times: number): readonly string[] {
  return Array.from({ length: times }, () => command);
}

test("stays silent below the threshold so a one-off lookup is never nudged", () => {
  const emitted = replay(grep(belowThreshold));

  expect(emitted.every((entry) => entry === undefined)).toBe(true);
});

test("nudges once the exploratory threshold is reached", () => {
  const emitted = replay(grep(searchThreshold));

  expect(emitted.at(-1)).toContain("code-search");
});

test("nudges only once so repeated searches never turn into alert fatigue", () => {
  const emitted = replay(grep(persistentSearches));

  expect(emitted.filter((entry) => entry !== undefined)).toHaveLength(1);
});

test("counts ripgrep and fd reached through Bash, not only Grep and Glob", () => {
  const emitted = replay(
    bash([...repeated("rg pattern", belowThreshold), "fd -e ts"]),
  );

  expect(emitted.at(-1)).toContain("code-search");
});

test("counts a search hidden behind a compound Bash command", () => {
  const emitted = replay(
    bash(repeated("cd /tmp && rg pattern", searchThreshold)),
  );

  expect(emitted.at(-1)).toContain("code-search");
});

test("counts a search behind an environment assignment prefix", () => {
  const emitted = replay(
    bash(repeated("RIPGREP_CONFIG_PATH=/dev/null rg pattern", searchThreshold)),
  );

  expect(emitted.at(-1)).toContain("code-search");
});

test("ignores Bash commands that are not searches", () => {
  const emitted = replay(
    bash(repeated("git log --oneline", persistentSearches)),
  );

  expect(emitted.every((entry) => entry === undefined)).toBe(true);
});

test("disarms permanently once the skill is invoked", () => {
  const emitted = replay([
    ...grep(belowThreshold),
    { toolInput: { skill: "code-search" }, toolName: "Skill" },
    ...grep(persistentSearches),
  ]);

  expect(emitted.every((entry) => entry === undefined)).toBe(true);
});

test("another skill does not disarm the nudge", () => {
  const emitted = replay([
    { toolInput: { skill: "testing" }, toolName: "Skill" },
    ...grep(searchThreshold),
  ]);

  expect(emitted.at(-1)).toContain("code-search");
});

test("treats unreadable or absent stored state as a fresh session", () => {
  expect(parseStoredState("")).toEqual(initialNudgeState);
  expect(parseStoredState("not json")).toEqual(initialNudgeState);
  expect(parseStoredState('{"searches":"many"}')).toEqual(initialNudgeState);
});

test("restores a stored session so the count survives across hook processes", () => {
  const stored = parseStoredState(JSON.stringify({ searches: belowThreshold }));
  const turn = nextNudgeTurn(stored, {
    toolInput: { pattern: "whatever" },
    toolName: "Grep",
  });

  expect(turn.additionalContext).toContain("code-search");
});

test("names the skill and the count so the nudge is actionable", () => {
  const emitted = replay(grep(searchThreshold));
  const message = emitted.at(-1) ?? "";

  expect(message).toContain("code-search");
  expect(message).toContain(String(searchThreshold));
});
