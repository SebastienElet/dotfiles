import type { Agent } from "./agent-memory-eval-process.ts";

type Event = Readonly<Record<string, unknown>>;
type CommandArguments = readonly [Agent, string, string, string, string?];
type ObservedCommandArguments = readonly [
  unknown,
  string,
  string,
  string | undefined,
];

function memoryCommandOutput(
  ...[
    agent,
    stream,
    runtime,
    subcommand,
    expectedCommand = `${runtime} ${subcommand} --format json`,
  ]: CommandArguments
): string {
  const output = outputForAgent(agent, parseEvents(stream), expectedCommand);
  if (output === undefined || output === "") {
    throw new Error(`${agent} did not complete agent-memory ${subcommand}`);
  }
  return output;
}

function memoryCommandObserved(
  ...[agent, stream, runtime, subcommand, expectedCommand]: CommandArguments
): boolean {
  const events = parseEvents(stream);
  if (agent === "codex") {
    return codexCommandObserved(events, runtime, subcommand, expectedCommand);
  }
  if (agent === "claude") {
    return claudeCommandObserved(events, runtime, subcommand, expectedCommand);
  }
  return cursorCommandObserved(events, runtime, subcommand, expectedCommand);
}

function outputForAgent(
  agent: Agent,
  events: readonly Event[],
  expectedCommand: string,
): string | undefined {
  if (agent === "codex") {
    return codexOutput(events, expectedCommand);
  }
  if (agent === "claude") {
    return claudeOutput(events, expectedCommand);
  }
  return cursorOutput(events, expectedCommand);
}

function codexCommandObserved(
  ...[events, runtime, subcommand, expectedCommand]: readonly [
    readonly Event[],
    string,
    string,
    string | undefined,
  ]
): boolean {
  return events
    .map((event) => field(event, "item"))
    .some(
      (item) =>
        item?.type === "command_execution" &&
        observedCommandMatches(
          item.command,
          runtime,
          subcommand,
          expectedCommand,
        ),
    );
}

function claudeCommandObserved(
  ...[events, runtime, subcommand, expectedCommand]: readonly [
    readonly Event[],
    string,
    string,
    string | undefined,
  ]
): boolean {
  return events
    .flatMap((event) => messageContent(event))
    .some(
      (block) =>
        block.type === "tool_use" &&
        block.name === "Bash" &&
        observedCommandMatches(
          field(block, "input")?.command,
          runtime,
          subcommand,
          expectedCommand,
        ),
    );
}

function cursorCommandObserved(
  ...[events, runtime, subcommand, expectedCommand]: readonly [
    readonly Event[],
    string,
    string,
    string | undefined,
  ]
): boolean {
  return events.some((event) =>
    observedCommandMatches(
      field(field(field(event, "tool_call"), "shellToolCall"), "args")?.command,
      runtime,
      subcommand,
      expectedCommand,
    ),
  );
}

function codexOutput(
  events: readonly Event[],
  expectedCommand: string,
): string | undefined {
  const item = events
    .map((event) => field(event, "item"))
    .find(
      (candidate) =>
        candidate?.type === "command_execution" &&
        candidate.status === "completed" &&
        candidate.exit_code === 0 &&
        commandMatches(candidate.command, expectedCommand),
    );
  return typeof item?.aggregated_output === "string"
    ? item.aggregated_output
    : undefined;
}

function claudeOutput(
  events: readonly Event[],
  expectedCommand: string,
): string | undefined {
  const uses = events
    .flatMap((event) => messageContent(event))
    .filter(
      (block) =>
        block.type === "tool_use" &&
        block.name === "Bash" &&
        typeof block.id === "string" &&
        commandMatches(field(block, "input")?.command, expectedCommand),
    );
  for (const use of uses) {
    const result = events
      .flatMap((event) => messageContent(event))
      .find(
        (block) =>
          block.type === "tool_result" &&
          block.tool_use_id === use.id &&
          block.is_error !== true,
      );
    const output = valueText(result?.content);
    if (output !== "") {
      return output;
    }
  }
  return undefined;
}

function cursorOutput(
  events: readonly Event[],
  expectedCommand: string,
): string | undefined {
  const command = events.find((event) => {
    const shell = field(field(event, "tool_call"), "shellToolCall");
    return (
      typeof event.call_id === "string" &&
      commandMatches(field(shell, "args")?.command, expectedCommand)
    );
  });
  const result = events.find((event) => {
    const shell = field(field(event, "tool_call"), "shellToolCall");
    return (
      event.call_id === command?.call_id &&
      field(field(shell, "result"), "success") !== undefined
    );
  });
  const shell = field(field(result, "tool_call"), "shellToolCall");
  return valueText(field(field(shell, "result"), "success")?.stdout);
}

function commandMatches(value: unknown, expectedCommand: string): boolean {
  if (typeof value !== "string") {
    return false;
  }
  const command = value.trim();
  return (
    command === expectedCommand ||
    command === `/bin/zsh -lc "${expectedCommand}"`
  );
}

function observedCommandMatches(
  ...[value, runtime, subcommand, expectedCommand]: ObservedCommandArguments
): boolean {
  if (expectedCommand !== undefined) {
    return commandMatches(value, expectedCommand);
  }
  if (typeof value !== "string") {
    return false;
  }
  const wrapped = /^\/bin\/zsh -lc "(?<command>[\s\S]*)"$/u.exec(value.trim());
  const command = wrapped?.groups?.command ?? value.trim();
  if (/^\s*(?:echo|printf)\b/u.test(command)) {
    return false;
  }
  return (
    (command.includes(runtime) || command.includes(`'${runtime}'`)) &&
    command.includes(` ${subcommand} --format json`)
  );
}

function parseEvents(stream: string): readonly Event[] {
  return stream
    .split("\n")
    .filter((line) => line.trim() !== "")
    .map((line, index) => parseEvent(line, index));
}

function messageContent(event: Event): readonly Event[] {
  const content = field(event, "message")?.content;
  return Array.isArray(content) ? content.filter((item) => isRecord(item)) : [];
}

function valueText(value: unknown): string {
  if (typeof value === "string") {
    return value;
  }
  if (Array.isArray(value)) {
    return value.map((item) => valueText(item)).join("\n");
  }
  if (isRecord(value) && typeof value.text === "string") {
    return value.text;
  }
  return "";
}

function memoryCommandErrorCode(stream: string): string | undefined {
  let code: string | undefined = undefined;
  for (const match of stream.matchAll(
    /\\?"code\\?":\\?"(?<code>[a-z_]+)\\?"/gu,
  )) {
    code = match.groups?.code;
  }
  return code;
}

function parseEvent(line: string, index: number): Event {
  try {
    const parsed: unknown = JSON.parse(line);
    if (!isRecord(parsed)) {
      throw new Error("action event must be an object");
    }
    return parsed;
  } catch {
    throw new Error(`malformed action JSONL at line ${index + 1}`);
  }
}

function field(value: Event | undefined, name: string): Event | undefined {
  const candidate = value?.[name];
  return isRecord(candidate) ? candidate : undefined;
}

function isRecord(value: unknown): value is Event {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

export { memoryCommandErrorCode, memoryCommandObserved, memoryCommandOutput };
