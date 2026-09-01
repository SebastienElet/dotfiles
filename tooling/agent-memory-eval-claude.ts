import type { Agent } from "./agent-memory-eval-process.ts";

function normalizeAgentVersion(agent: Agent, output: string): string {
  const version = output.trim();
  if (agent !== "claude") return version;
  return /^(\d+\.\d+\.\d+)(?: \(Claude Code\))?$/u.exec(version)?.[1] ?? version;
}

function claudeVersion(events: readonly Readonly<Record<string, unknown>>[]): unknown {
  const init = events.find((event) => event.type === "system" && event.subtype === "init");
  return init?.claude_code_version;
}

function claudeHookContext(
  events: readonly Readonly<Record<string, unknown>>[],
  source: string,
): string {
  const started = events.find(
    (event) =>
      event.type === "system" &&
      event.subtype === "hook_started" &&
      event.hook_event === "UserPromptSubmit" &&
      typeof event.hook_id === "string",
  );
  const response = events.find(
    (event) =>
      event.type === "system" &&
      event.subtype === "hook_response" &&
      event.hook_event === "UserPromptSubmit" &&
      event.hook_id === started?.hook_id &&
      (event.exit_code === 0 || event.outcome === "success"),
  );
  const text = [response?.output, response?.stdout].map(hookOutputText).join("\n");
  return text.includes(source) && text.includes("verdict_age_milliseconds") ? text : "";
}

function hookOutputText(value: unknown): string {
  if (typeof value === "string") return value;
  if (value === undefined) return "";
  try {
    return JSON.stringify(value);
  } catch {
    return "";
  }
}

export { claudeHookContext, claudeVersion, normalizeAgentVersion };
