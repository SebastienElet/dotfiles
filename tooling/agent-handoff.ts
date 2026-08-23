import { mkdir, open, readFile, stat } from "node:fs/promises";
import { dirname, join } from "node:path";

type Agent = "Claude Code" | "Codex";

type HookEvent = Readonly<{
  sessionId: string;
  stopHookActive: boolean;
  transcriptPath: string;
}>;

type Usage = Readonly<{
  agent: Agent;
  used: number;
  window?: number;
}>;

class HandoffError extends Error {
  constructor(
    message: string,
    readonly exitCode: number,
  ) {
    super(message);
  }
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function parseHookEvent(input: string): HookEvent {
  let value: unknown;
  try {
    value = JSON.parse(input);
  } catch {
    throw new HandoffError("invalid hook event: expected JSON", 1);
  }
  if (!isRecord(value)) throw new HandoffError("invalid hook event: expected an object", 1);
  if (typeof value.session_id !== "string" || value.session_id.length === 0) {
    throw new HandoffError("missing session_id", 1);
  }
  if (!/^(?!\.{1,2}$)[A-Za-z0-9._-]+$/.test(value.session_id)) {
    throw new HandoffError("invalid session_id", 1);
  }
  if (typeof value.transcript_path !== "string" || value.transcript_path.length === 0) {
    throw new HandoffError("missing transcript_path", 1);
  }
  if (value.stop_hook_active !== undefined && typeof value.stop_hook_active !== "boolean") {
    throw new HandoffError("invalid stop_hook_active", 1);
  }
  return {
    sessionId: value.session_id,
    stopHookActive: value.stop_hook_active ?? false,
    transcriptPath: value.transcript_path,
  };
}

function parseTokenCount(value: unknown, field: string, fallback?: number): number {
  if (value === undefined && fallback !== undefined) return fallback;
  if (typeof value !== "number" || !Number.isSafeInteger(value) || value < 0) {
    throw new HandoffError(`invalid ${field}`, 1);
  }
  return value;
}

function parseClaudeUsage(record: Record<string, unknown>): Usage | undefined {
  if (record.type !== "assistant" || record.isSidechain === true) return undefined;
  if (!isRecord(record.message) || !isRecord(record.message.usage)) return undefined;
  const usage = record.message.usage;
  const input = parseTokenCount(usage.input_tokens, "Claude input_tokens");
  const cacheRead = parseTokenCount(usage.cache_read_input_tokens, "Claude cache_read_input_tokens", 0);
  const cacheCreation = parseTokenCount(
    usage.cache_creation_input_tokens,
    "Claude cache_creation_input_tokens",
    0,
  );
  const used = input + cacheRead + cacheCreation;
  if (!Number.isSafeInteger(used)) throw new HandoffError("invalid Claude token total", 1);
  return { agent: "Claude Code", used };
}

function parseCodexUsage(record: Record<string, unknown>): Usage | undefined {
  if (record.type !== "event_msg" || !isRecord(record.payload)) return undefined;
  if (record.payload.type !== "token_count" || !isRecord(record.payload.info)) return undefined;
  const info = record.payload.info;
  if (!isRecord(info.last_token_usage)) return undefined;
  return {
    agent: "Codex",
    used: parseTokenCount(info.last_token_usage.input_tokens, "Codex input_tokens"),
    window: parseTokenCount(info.model_context_window, "Codex model_context_window"),
  };
}

function findLatestUsage(transcript: string): Usage {
  const lines = transcript.trimEnd().split("\n").slice(-500);
  let latest: Usage | undefined;
  for (const [index, line] of lines.entries()) {
    if (line.trim() === "") continue;
    let record: unknown;
    try {
      record = JSON.parse(line);
    } catch {
      throw new HandoffError(`malformed transcript JSON at retained line ${index + 1}`, 1);
    }
    if (!isRecord(record)) continue;
    latest = parseClaudeUsage(record) ?? parseCodexUsage(record) ?? latest;
  }
  if (latest === undefined) throw new HandoffError("no supported usage record in transcript", 1);
  return latest;
}

function parsePositiveInteger(value: string, name: string): number {
  if (!/^[1-9][0-9]*$/.test(value)) throw new HandoffError(`invalid ${name}`, 1);
  const parsed = Number(value);
  if (!Number.isSafeInteger(parsed)) throw new HandoffError(`invalid ${name}`, 1);
  return parsed;
}

function selectThreshold(usage: Usage, environment: NodeJS.ProcessEnv): number {
  const configured = environment.HANDOFF_TOKEN_THRESHOLD;
  if (configured !== undefined && configured !== "") {
    return parsePositiveInteger(configured, "HANDOFF_TOKEN_THRESHOLD");
  }
  const window = usage.window ?? (() => {
    const value = environment.CLAUDE_CODE_AUTO_COMPACT_WINDOW;
    if (value === undefined || value === "") {
      throw new HandoffError("missing context window", 1);
    }
    return parsePositiveInteger(value, "CLAUDE_CODE_AUTO_COMPACT_WINDOW");
  })();
  return Number((BigInt(window) * 85n) / 100n);
}

function isFileSystemError(error: unknown, code: string): boolean {
  return isRecord(error) && error.code === code;
}

async function sentinelExists(path: string): Promise<boolean> {
  try {
    const metadata = await stat(path);
    if (!metadata.isFile()) throw new HandoffError("invalid handoff sentinel", 3);
    return true;
  } catch (error) {
    if (isFileSystemError(error, "ENOENT")) return false;
    throw new HandoffError("cannot inspect handoff sentinel", 3);
  }
}

async function createSentinel(path: string): Promise<boolean> {
  try {
    await mkdir(dirname(path), { recursive: true });
    const file = await open(path, "wx");
    await file.close();
    return true;
  } catch (error) {
    if (isFileSystemError(error, "EEXIST")) return false;
    throw new HandoffError("cannot create handoff sentinel", 3);
  }
}

async function readTranscript(path: string): Promise<string> {
  try {
    return await readFile(path, "utf8");
  } catch {
    throw new HandoffError("cannot read transcript", 1);
  }
}

function handoffOutput(usage: Usage, threshold: number): string {
  const invocation = usage.agent === "Codex" ? "$handoff" : "/handoff";
  return JSON.stringify(
    {
      decision: "block",
      reason:
        `Context is at ${Math.floor(usage.used / 1000)}k tokens, past the ${Math.floor(threshold / 1000)}k handoff threshold. ` +
        `Start no new work. Use ${invocation} to emit the resume prompt for a fresh session, then stop.`,
    },
    null,
    2,
  );
}

export async function runAgentHandoff(input: string, environment: NodeJS.ProcessEnv): Promise<number> {
  try {
    const event = parseHookEvent(input);
    if (event.stopHookActive) return 0;
    const stateRoot = environment.XDG_STATE_HOME ||
      (environment.HOME === undefined ? undefined : join(environment.HOME, ".local", "state"));
    if (stateRoot === undefined || stateRoot === "") {
      throw new HandoffError("missing HOME and XDG_STATE_HOME", 1);
    }
    const sentinel = join(stateRoot, "dotfiles", "handoff", event.sessionId);
    if (await sentinelExists(sentinel)) return 0;
    const usage = findLatestUsage(await readTranscript(event.transcriptPath));
    const threshold = selectThreshold(usage, environment);
    if (usage.used < threshold || !(await createSentinel(sentinel))) return 0;
    process.stdout.write(`${handoffOutput(usage, threshold)}\n`);
    return 0;
  } catch (error) {
    const failure = error instanceof HandoffError ? error : new HandoffError("unexpected failure", 3);
    process.stderr.write(`agent-handoff: ${failure.message}\n`);
    return failure.exitCode;
  }
}
