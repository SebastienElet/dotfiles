import { join } from "node:path";

const entryPoint = join(import.meta.dir, "agent-handoff");
const assistantMessageTokenCount = 2;
const defaultContextWindow = 100_000;
const inheritedEnvironment = Object.fromEntries(
  Object.entries(process.env).filter(
    (entry: readonly [string, string | undefined]): entry is [string, string] =>
      entry[1] !== undefined,
  ),
);

type HookResult = Readonly<{
  exitCode: number;
  stderr: string;
  stdout: string;
}>;

function claudeUsage(used: number, sidechain = false): string {
  return JSON.stringify({
    isSidechain: sidechain,
    message: {
      usage: {
        cache_creation_input_tokens: 0,
        cache_read_input_tokens: used - assistantMessageTokenCount,
        input_tokens: assistantMessageTokenCount,
      },
    },
    type: "assistant",
  });
}

function codexUsage(used: number, window = defaultContextWindow): string {
  return JSON.stringify({
    payload: {
      info: {
        last_token_usage: { input_tokens: used },
        model_context_window: window,
        total_token_usage: { input_tokens: 999_999 },
      },
      type: "token_count",
    },
    type: "event_msg",
  });
}

function claudeEvent(
  transcriptPath: string,
  sessionId: string,
  stopHookActive = false,
): string {
  return JSON.stringify({
    hook_event_name: "Stop",
    session_id: sessionId,
    stop_hook_active: stopHookActive,
    transcript_path: transcriptPath,
  });
}

function codexEvent(transcriptPath: string, sessionId: string): string {
  return JSON.stringify({
    event: "Stop",
    session_id: sessionId,
    transcript_path: transcriptPath,
  });
}

async function runEntryPoint(
  testRoot: string,
  input: string,
  environment: Readonly<Record<string, string>> = {},
): Promise<HookResult> {
  const env: Record<string, string> = {
    ...inheritedEnvironment,
    CLAUDE_CODE_AUTO_COMPACT_WINDOW: "100000",
    HOME: testRoot,
    XDG_STATE_HOME: join(testRoot, "state"),
    ...environment,
  };
  delete env.HANDOFF_TOKEN_THRESHOLD;
  Object.assign(env, environment);

  const process = Bun.spawn([entryPoint], {
    env,
    stderr: "pipe",
    stdin: new Blob([input]),
    stdout: "pipe",
  });
  const [exitCode, stderr, stdout] = await Promise.all([
    process.exited,
    new Response(process.stderr).text(),
    new Response(process.stdout).text(),
  ]);

  return { exitCode, stderr, stdout };
}

export { claudeEvent, claudeUsage, codexEvent, codexUsage, runEntryPoint };
export type { HookResult };
