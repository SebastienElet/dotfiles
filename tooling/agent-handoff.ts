import { type Usage, findLatestUsage } from "./agent-handoff-transcript.ts";
import { dirname, join } from "node:path";
import { mkdir, open, readFile, stat } from "node:fs/promises";
import { HandoffError } from "./agent-handoff-error.ts";

type HookEvent = Readonly<{
  sessionId: string;
  stopHookActive: boolean;
  transcriptPath: string;
}>;

const unexpectedFailureExitCode = 3;
const tokensPerThousand = 1000;
const jsonIndentSpaces = 2;

function parseHookEventValue(input: string): unknown {
  try {
    return JSON.parse(input);
  } catch {
    throw new HandoffError("invalid hook event: expected JSON", 1);
  }
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function parseHookEvent(input: string): HookEvent {
  const value = parseHookEventValue(input);
  if (!isRecord(value)) {
    throw new HandoffError("invalid hook event: expected an object", 1);
  }
  const claudeEvent = value.hook_event_name;
  const codexEvent = value.event;
  if (claudeEvent === undefined && codexEvent === undefined) {
    throw new HandoffError("missing Stop event", 1);
  }
  if (
    (claudeEvent !== undefined && claudeEvent !== "Stop") ||
    (codexEvent !== undefined && codexEvent !== "Stop")
  ) {
    throw new HandoffError("unsupported hook event", 1);
  }
  if (typeof value.session_id !== "string" || value.session_id.length === 0) {
    throw new HandoffError("missing session_id", 1);
  }
  if (!/^(?!\.{1,2}$)[A-Za-z0-9._-]+$/u.test(value.session_id)) {
    throw new HandoffError("invalid session_id", 1);
  }
  if (
    typeof value.transcript_path !== "string" ||
    value.transcript_path.length === 0
  ) {
    throw new HandoffError("missing transcript_path", 1);
  }
  if (
    value.stop_hook_active !== undefined &&
    typeof value.stop_hook_active !== "boolean"
  ) {
    throw new HandoffError("invalid stop_hook_active", 1);
  }
  return {
    sessionId: value.session_id,
    stopHookActive: value.stop_hook_active ?? false,
    transcriptPath: value.transcript_path,
  };
}

function parsePositiveInteger(value: string, name: string): number {
  if (!/^[1-9][0-9]*$/u.test(value)) {
    throw new HandoffError(`invalid ${name}`, 1);
  }
  const parsed = Number(value);
  if (!Number.isSafeInteger(parsed)) {
    throw new HandoffError(`invalid ${name}`, 1);
  }
  return parsed;
}

function selectThreshold(
  usage: Usage,
  environment: Readonly<NodeJS.ProcessEnv>,
): number {
  const configured = environment.HANDOFF_TOKEN_THRESHOLD;
  if (configured !== undefined && configured !== "") {
    return parsePositiveInteger(configured, "HANDOFF_TOKEN_THRESHOLD");
  }
  const window =
    usage.window ??
    ((): number => {
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
    if (!metadata.isFile()) {
      throw new HandoffError(
        "invalid handoff sentinel",
        unexpectedFailureExitCode,
      );
    }
    return true;
  } catch (error) {
    if (isFileSystemError(error, "ENOENT")) {
      return false;
    }
    throw new HandoffError(
      "cannot inspect handoff sentinel",
      unexpectedFailureExitCode,
    );
  }
}

async function createSentinel(path: string): Promise<boolean> {
  try {
    await mkdir(dirname(path), { recursive: true });
    const file = await open(path, "wx");
    await file.close();
    return true;
  } catch (error) {
    if (isFileSystemError(error, "EEXIST")) {
      return false;
    }
    throw new HandoffError(
      "cannot create handoff sentinel",
      unexpectedFailureExitCode,
    );
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
        `Context is at ${Math.floor(usage.used / tokensPerThousand)}k tokens, past the ${Math.floor(threshold / tokensPerThousand)}k handoff threshold. ` +
        `Start no new work. Use ${invocation} to emit the resume prompt for a fresh session, then stop.`,
    },
    null,
    jsonIndentSpaces,
  );
}

function stateRootFromEnvironment(
  environment: Readonly<NodeJS.ProcessEnv>,
): string | undefined {
  const xdgStateHome = environment.XDG_STATE_HOME;
  if (xdgStateHome !== undefined && xdgStateHome !== "") {
    return xdgStateHome;
  }
  if (environment.HOME === undefined) {
    return undefined;
  }
  return join(environment.HOME, ".local", "state");
}

export async function runAgentHandoff(
  input: string,
  environment: Readonly<NodeJS.ProcessEnv>,
): Promise<number> {
  try {
    const event = parseHookEvent(input);
    if (event.stopHookActive) {
      return 0;
    }
    const stateRoot = stateRootFromEnvironment(environment);
    if (stateRoot === undefined || stateRoot === "") {
      throw new HandoffError("missing HOME and XDG_STATE_HOME", 1);
    }
    const sentinel = join(stateRoot, "dotfiles", "handoff", event.sessionId);
    if (await sentinelExists(sentinel)) {
      return 0;
    }
    const usage = findLatestUsage(await readTranscript(event.transcriptPath));
    const threshold = selectThreshold(usage, environment);
    if (usage.used < threshold || !(await createSentinel(sentinel))) {
      return 0;
    }
    process.stdout.write(`${handoffOutput(usage, threshold)}\n`);
    return 0;
  } catch (error) {
    const failure =
      error instanceof HandoffError
        ? error
        : new HandoffError("unexpected failure", unexpectedFailureExitCode);
    process.stderr.write(`agent-handoff: ${failure.message}\n`);
    return failure.exitCode;
  }
}
