import { closeSync, constants, fstatSync, openSync, readSync } from "node:fs";
import { isAbsolute } from "node:path";
import { z } from "zod";

const modelSchema = z.string().regex(/^[a-zA-Z0-9][a-zA-Z0-9._:/-]{0,127}$/u);
const effortSchema = z.string().regex(/^[a-z][a-z0-9_-]{0,31}$/u);
const profileSchema = z.object({ model: modelSchema, effort: effortSchema });
type Profile = z.infer<typeof profileSchema>;
const maximumTurnIdLength = 128;
const hookSchema = z.object({
  hook_event_name: z.enum(["SessionStart", "PreToolUse", "Stop"]),
  model: modelSchema.nullish(),
  turn_id: z.string().min(1).max(maximumTurnIdLength).nullish(),
  transcript_path: z.string().min(1).refine(isAbsolute).nullish(),
  agent_id: z.string().nullish(),
  agent_type: z.string().nullish(),
});
type Hook = z.infer<typeof hookSchema>;
const recordSchema = z.object({ type: z.string(), payload: z.unknown() });
const contextSchema = z.object({
  turn_id: z.string().min(1).max(maximumTurnIdLength),
  model: modelSchema,
  effort: effortSchema.nullish(),
});
type TurnContext = z.infer<typeof contextSchema>;
const maximumTranscriptBytes = 1_048_576;

function transcriptTail(path: string): string {
  const descriptor = openSync(
    path,
    constants.O_RDONLY | constants.O_NONBLOCK | constants.O_NOFOLLOW,
  );
  try {
    const metadata = fstatSync(descriptor);
    if (!metadata.isFile() || !Number.isSafeInteger(metadata.size)) {
      throw new Error("Unreadable transcript");
    }
    const size = Math.min(metadata.size, maximumTranscriptBytes);
    const offset = metadata.size - size;
    const buffer = Buffer.alloc(size);
    const count = readSync(descriptor, buffer, 0, size, offset);
    const start = offset === 0 ? 0 : buffer.indexOf("\n") + 1;
    return new TextDecoder("utf-8", { fatal: true }).decode(
      buffer.subarray(start, count),
    );
  } finally {
    closeSync(descriptor);
  }
}

function latestContext(contents: string): TurnContext | null {
  if (!contents.endsWith("\n")) {
    return null;
  }
  let latest: TurnContext | null = null;
  for (const line of contents.trimEnd().split("\n")) {
    const record = recordSchema.parse(JSON.parse(line));
    if (record.type === "turn_context") {
      latest = contextSchema.parse(record.payload);
    }
  }
  return latest;
}

function observedEffort(hook: Hook): string | null {
  const transcriptPath = hook.transcript_path ?? null;
  const turnId = hook.turn_id ?? null;
  if (
    hook.hook_event_name === "SessionStart" ||
    transcriptPath === null ||
    turnId === null
  ) {
    return null;
  }
  try {
    const context = latestContext(transcriptTail(transcriptPath));
    return context?.turn_id === turnId && context.model === hook.model
      ? (context.effort ?? null)
      : null;
  } catch {
    return null;
  }
}

function profileMessage(expected: Profile, input: unknown): string {
  const parsed = hookSchema.safeParse(input);
  if (!parsed.success) {
    return "codex-profile: inconnu (entrée native illisible)";
  }
  const hook = parsed.data;
  if ((hook.agent_id ?? hook.agent_type ?? null) !== null) {
    return "codex-profile: inconnu (profil spécialisé, hors comparaison racine)";
  }
  const model = hook.model ?? null;
  const effort = observedEffort(hook);
  const different =
    (model !== null && model !== expected.model) ||
    (effort !== null && effort !== expected.effort);
  const completeness =
    model !== null && effort !== null ? "conforme" : "inconnu";
  const state = different ? "AVERTISSEMENT — différent" : completeness;
  return `codex-profile: ${state}; demandé ${expected.model}/${expected.effort}; observé ${model ?? "inconnu"}/${effort ?? "inconnu"}`;
}

export { profileMessage, profileSchema };
