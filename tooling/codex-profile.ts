import { profileMessage, profileSchema } from "./codex-profile-observation.ts";
import { launchProfile } from "./codex-profile-launch.ts";
import { relayProfileMessage } from "./codex-profile-relay.ts";
import { z } from "zod";

const argumentOffset = 2;
const maximumInputBytes = 1_048_576;

async function nativePayload(): Promise<unknown> {
  try {
    const chunks: Uint8Array[] = [];
    let size = 0;
    for await (const chunk of Bun.stdin.stream()) {
      size += chunk.byteLength;
      if (size > maximumInputBytes) {
        return null;
      }
      chunks.push(chunk);
    }
    return JSON.parse(Buffer.concat(chunks).toString("utf8"));
  } catch {
    return null;
  }
}

async function codexProfile(arguments_: readonly string[]): Promise<void> {
  if (arguments_[0] === "--help") {
    process.stdout.write(
      "Usage: codex-profile MODEL EFFORT [CODEX_ARGUMENTS...]\n",
    );
    return;
  }
  if (arguments_[0] === "hook") {
    const [, model, effort] = z
      .tuple([z.literal("hook"), z.string(), z.string()])
      .parse(arguments_);
    const profile = profileSchema.parse({ model, effort });
    const systemMessage = profileMessage(profile, await nativePayload());
    const relay = z
      .string()
      .min(1)
      .optional()
      .parse(process.env.DOTFILES_CODEX_PROFILE_RELAY);
    if (
      relay !== undefined &&
      !(await relayProfileMessage(relay, systemMessage))
    ) {
      const unavailable = "codex-profile: inconnu (relais local indisponible)";
      process.stdout.write(JSON.stringify({ systemMessage: unavailable }));
      process.stderr.write(`${unavailable}\n`);
      process.exitCode = 1;
      return;
    }
    process.stdout.write(JSON.stringify({ systemMessage }));
    return;
  }
  try {
    process.exitCode = await launchProfile(arguments_, import.meta.filename);
  } catch {
    process.stderr.write(
      "codex-profile: launch failed; require Codex and MODEL EFFORT [CODEX_ARGUMENTS...]\n",
    );
    process.exitCode = 1;
  }
}

if (import.meta.main) {
  await codexProfile(process.argv.slice(argumentOffset));
}

export { codexProfile };
