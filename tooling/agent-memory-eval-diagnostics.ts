import type { Agent, AgentCondition } from "./agent-memory-eval-process.ts";

function diagnosticClass(diagnostic: string): string {
  const normalized = diagnostic.toLowerCase();
  if (/monthly usage limit|higher limits|actionrequirederror/u.test(normalized)) return "usage_limit";
  if (/auth|login|credential|unauthorized/u.test(normalized)) return "authentication_unavailable";
  if (/hook|trust/u.test(normalized)) return "hook_configuration";
  if (/model|provider/u.test(normalized)) return "model_unavailable";
  if (/network|connect|timeout/u.test(normalized)) return "network_unavailable";
  return diagnostic.trim() === "" ? "empty_stderr" : "redacted_process_failure";
}

function classifyAgentFailure(_agent: Agent, stdout: string, stderr: string): string {
  const event = stdout
    .split("\n")
    .filter((line) => line.trim() !== "")
    .map(parseDiagnosticEvent)
    .filter((value): value is Readonly<Record<string, unknown>> => value !== undefined)
    .at(-1);
  const fields = event === undefined ? [] : diagnosticFields(event);
  return diagnosticClass([stderr, ...fields].join("\n"));
}

function conditionedAgentFailure(
  agent: Agent,
  condition: AgentCondition,
  stdout: string,
  stderr: string,
): Error {
  return new Error(`${agent}:${condition}:${classifyAgentFailure(agent, stdout, stderr)}`);
}

function agentStreamFailed(stdout: string): boolean {
  return stdout
    .split("\n")
    .filter((line) => line.trim() !== "")
    .map(parseDiagnosticEvent)
    .filter((value): value is Readonly<Record<string, unknown>> => value !== undefined)
    .some(
      (event) =>
        event.type === "error" ||
        (event.type === "result" &&
          (event.is_error === true || String(event.subtype).startsWith("error"))),
    );
}

function parseDiagnosticEvent(line: string): Readonly<Record<string, unknown>> | undefined {
  try {
    const value: unknown = JSON.parse(line);
    if (!isRecord(value) || !["system", "result", "error"].includes(String(value.type))) {
      return undefined;
    }
    return value;
  } catch {
    return undefined;
  }
}

function diagnosticFields(event: Readonly<Record<string, unknown>>): string[] {
  const direct = [event.subtype, event.class, event.code, event.message, event.result].filter(
    (value): value is string => typeof value === "string",
  );
  const error = isRecord(event.error) ? event.error : undefined;
  return [
    ...direct,
    ...[error?.class, error?.code, error?.message].filter(
      (value): value is string => typeof value === "string",
    ),
  ];
}

function isRecord(value: unknown): value is Readonly<Record<string, unknown>> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

export { agentStreamFailed, classifyAgentFailure, conditionedAgentFailure, diagnosticClass };
