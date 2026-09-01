import type { Agent } from "./agent-memory-eval-process.ts";

function memoryCommandOutput(
  agent: Agent,
  stream: string,
  runtime: string,
  subcommand: string,
  expectedCommand = `${runtime} ${subcommand} --format json`,
): string {
  const events = stream
    .split("\n")
    .filter((line) => line.trim() !== "")
    .map(parseEvent);
  const output =
    agent === "codex"
      ? codexOutput(events, expectedCommand)
      : agent === "claude"
        ? claudeOutput(events, expectedCommand)
        : cursorOutput(events, expectedCommand);
  if (output === undefined || output === "") {
    throw new Error(`${agent} did not complete agent-memory ${subcommand}`);
  }
  return output;
}

function memoryCommandObserved(
  agent: Agent,
  stream: string,
  runtime: string,
  subcommand: string,
  expectedCommand?: string,
): boolean {
  const events = stream
    .split("\n")
    .filter((line) => line.trim() !== "")
    .map(parseEvent);
  if (agent === "codex") {
    return events
      .map((event) => field(event, "item"))
      .some(
        (item) =>
          item?.type === "command_execution" &&
          observedCommandMatches(item.command, runtime, subcommand, expectedCommand),
      );
  }
  if (agent === "claude") {
    return events
      .flatMap(messageContent)
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
  events: readonly Readonly<Record<string, unknown>>[],
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
  return typeof item?.aggregated_output === "string" ? item.aggregated_output : undefined;
}

function claudeOutput(
  events: readonly Readonly<Record<string, unknown>>[],
  expectedCommand: string,
): string | undefined {
  const uses = events.flatMap(messageContent).filter(
    (block) =>
      block.type === "tool_use" &&
      block.name === "Bash" &&
      typeof block.id === "string" &&
      commandMatches(field(block, "input")?.command, expectedCommand),
  );
  for (const use of uses) {
    const result = events
      .flatMap(messageContent)
      .find(
        (block) =>
          block.type === "tool_result" &&
          block.tool_use_id === use.id &&
          block.is_error !== true,
      );
    const output = valueText(result?.content);
    if (output !== "") return output;
  }
  return undefined;
}

function cursorOutput(
  events: readonly Readonly<Record<string, unknown>>[],
  expectedCommand: string,
): string | undefined {
  const command = events.find((event) => {
    const shell = field(field(event, "tool_call"), "shellToolCall");
    return typeof event.call_id === "string" && commandMatches(field(shell, "args")?.command, expectedCommand);
  });
  const result = events.find((event) => {
    const shell = field(field(event, "tool_call"), "shellToolCall");
    return event.call_id === command?.call_id && field(field(shell, "result"), "success") !== undefined;
  });
  const success = field(field(field(result, "tool_call"), "shellToolCall"), "result");
  return valueText(field(success, "success")?.stdout);
}

function commandMatches(value: unknown, expectedCommand: string): boolean {
  if (typeof value !== "string") return false;
  const command = value.trim();
  return command === expectedCommand || command === `/bin/zsh -lc "${expectedCommand}"`;
}

function observedCommandMatches(
  value: unknown,
  runtime: string,
  subcommand: string,
  expectedCommand: string | undefined,
): boolean {
  if (expectedCommand !== undefined) return commandMatches(value, expectedCommand);
  if (typeof value !== "string") return false;
  const wrapped = value.trim().match(/^\/bin\/zsh -lc "([\s\S]*)"$/u);
  const command = wrapped?.[1] ?? value.trim();
  if (/^\s*(?:echo|printf)\b/u.test(command)) return false;
  return (
    (command.includes(runtime) || command.includes(`'${runtime}'`)) &&
    command.includes(` ${subcommand} --format json`)
  );
}

function messageContent(event: Readonly<Record<string, unknown>>): Readonly<Record<string, unknown>>[] {
  const content = field(event, "message")?.content;
  return Array.isArray(content) ? content.filter(isRecord) : [];
}

function valueText(value: unknown): string {
  if (typeof value === "string") return value;
  if (Array.isArray(value)) return value.map(valueText).join("\n");
  if (isRecord(value) && typeof value.text === "string") return value.text;
  return "";
}

function memoryCommandErrorCode(stream: string): string | undefined {
  const match = stream.match(/\\?"code\\?":\\?"([a-z_]+)\\?"/gu)?.at(-1);
  return match?.match(/[a-z_]+(?=\\?"$)/u)?.[0];
}

function parseEvent(line: string, index: number): Readonly<Record<string, unknown>> {
  try {
    const parsed: unknown = JSON.parse(line);
    if (!isRecord(parsed)) throw new Error();
    return parsed;
  } catch {
    throw new Error(`malformed action JSONL at line ${index + 1}`);
  }
}

function field(
  value: Readonly<Record<string, unknown>> | undefined,
  name: string,
): Readonly<Record<string, unknown>> | undefined {
  const candidate = value?.[name];
  return isRecord(candidate) ? candidate : undefined;
}

function isRecord(value: unknown): value is Readonly<Record<string, unknown>> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

export { memoryCommandErrorCode, memoryCommandObserved, memoryCommandOutput };
