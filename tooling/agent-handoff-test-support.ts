import { join } from "node:path";

const entryPoint = join(import.meta.dir, "agent-handoff");
const inheritedEnvironment = Object.fromEntries(
  Object.entries(process.env).filter(
    (entry): entry is [string, string] => entry[1] !== undefined,
  ),
);

export type HookResult = Readonly<{
  exitCode: number;
  stderr: string;
  stdout: string;
}>;

export function claudeUsage(used: number, sidechain = false): string {
  return JSON.stringify({
    type: "assistant",
    isSidechain: sidechain,
    message: {
      usage: {
        input_tokens: 2,
        cache_read_input_tokens: used - 2,
        cache_creation_input_tokens: 0,
      },
    },
  });
}

export function codexUsage(used: number, window = 100_000): string {
  return JSON.stringify({
    type: "event_msg",
    payload: {
      type: "token_count",
      info: {
        last_token_usage: { input_tokens: used },
        model_context_window: window,
        total_token_usage: { input_tokens: 999_999 },
      },
    },
  });
}

export function event(
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

export async function runEntryPoint(
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
