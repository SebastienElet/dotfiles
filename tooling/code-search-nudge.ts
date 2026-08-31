const searchThreshold = 3;

const routedSkill = "code-search";
const searchTools = new Set(["Glob", "Grep"]);
const searchBinaries = new Set(["ack", "ag", "fd", "find", "grep", "rg"]);
const shellSeparators = /\||&&|;/u;
const environmentAssignment = /^[A-Za-z_][A-Za-z0-9_]*=/u;

type NudgeState = Readonly<{
  nudged: boolean;
  searches: number;
  skillInvoked: boolean;
}>;

type HookToolCall = Readonly<{
  toolInput: Readonly<Record<string, unknown>>;
  toolName: string;
}>;

type NudgeTurn = Readonly<{
  additionalContext?: string;
  state: NudgeState;
}>;

type Classification = "ignored" | "search" | "skill-invocation";

const initialNudgeState: NudgeState = {
  nudged: false,
  searches: 0,
  skillInvoked: false,
};

function leadingWord(segment: string): string {
  return (
    segment
      .trim()
      .split(/\s+/u)
      .find((token) => !environmentAssignment.test(token)) ?? ""
  );
}

function isBashSearch(command: unknown): boolean {
  if (typeof command !== "string") {
    return false;
  }
  return command
    .split(shellSeparators)
    .some((segment) => searchBinaries.has(leadingWord(segment)));
}

function classifyToolCall(call: HookToolCall): Classification {
  if (call.toolName === "Skill") {
    return call.toolInput.skill === routedSkill
      ? "skill-invocation"
      : "ignored";
  }
  if (searchTools.has(call.toolName)) {
    return "search";
  }
  if (call.toolName === "Bash" && isBashSearch(call.toolInput.command)) {
    return "search";
  }
  return "ignored";
}

function nudgeMessage(searches: number): string {
  return [
    `${String(searches)} searches in this session without the ${routedSkill} skill.`,
    `Exploratory or structural search routes through ${routedSkill}: invoke it before continuing.`,
    "A one-off lookup of a literal you already know needs no skill.",
  ].join(" ");
}

function nextNudgeTurn(state: NudgeState, call: HookToolCall): NudgeTurn {
  const classification = classifyToolCall(call);
  if (classification === "skill-invocation") {
    return { state: { ...state, skillInvoked: true } };
  }
  if (classification === "ignored") {
    return { state };
  }
  const searches = state.searches + 1;
  if (state.skillInvoked || state.nudged || searches < searchThreshold) {
    return { state: { ...state, searches } };
  }
  return {
    additionalContext: nudgeMessage(searches),
    state: { ...state, nudged: true, searches },
  };
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function parseJson(content: string): unknown {
  try {
    return JSON.parse(content);
  } catch {
    return null;
  }
}

function parseStoredState(content: string): NudgeState {
  const value = parseJson(content);
  if (!isRecord(value)) {
    return initialNudgeState;
  }
  const { nudged, searches, skillInvoked } = value;
  if (
    typeof searches !== "number" ||
    !Number.isInteger(searches) ||
    searches < 0
  ) {
    return initialNudgeState;
  }
  return {
    nudged: nudged === true,
    searches,
    skillInvoked: skillInvoked === true,
  };
}

export {
  type HookToolCall,
  type NudgeState,
  type NudgeTurn,
  classifyToolCall,
  initialNudgeState,
  nextNudgeTurn,
  parseStoredState,
  searchThreshold,
};
