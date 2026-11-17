#!/usr/bin/env bun

import { createConnection } from "node:net";
import process from "node:process";
import { z } from "zod";

const argumentOffset = 2;
const exitCode = 17;
const forwardedArguments = process.argv.slice(argumentOffset);
const commandSchema = z.object({ command: z.string() });
const groupSchema = z.object({ hooks: z.array(commandSchema) });
const settingsSchema = z.object({
  hooks: z.object({ Stop: z.array(groupSchema) }),
});
const nativeInput = process.env.CODEX_PROFILE_TEST_NATIVE;
const rawRelayInput = process.env.CODEX_PROFILE_TEST_RELAY_INPUT;
const lifetime = z
  .enum(["wait", "resist"])
  .optional()
  .parse(process.env.CODEX_PROFILE_TEST_LIFETIME);
if (nativeInput !== undefined) {
  const hook = forwardedArguments.find((value) =>
    value.startsWith("hooks.Stop="),
  );
  const settings = settingsSchema.parse(Bun.TOML.parse(hook ?? ""));
  const command = settings.hooks.Stop[0]?.hooks[0]?.command;
  if (command === undefined) {
    throw new Error("Missing native hook");
  }
  const result = Bun.spawnSync(["/bin/sh", "-c", command], {
    stdin: Buffer.from(nativeInput),
  });
  if (result.exitCode !== 0) {
    throw new Error("Native hook failed");
  }
}
if (rawRelayInput !== undefined) {
  const path = z.string().parse(process.env.DOTFILES_CODEX_PROFILE_RELAY);
  await new Promise<void>((resolve, reject) => {
    const socket = createConnection(path);
    socket.on("error", reject);
    socket.on("connect", () => {
      socket.end(rawRelayInput);
    });
    socket.on("close", () => {
      resolve();
    });
  });
}
if (lifetime !== undefined) {
  process.on("SIGINT", () => {
    process.exit(exitCode);
  });
  process.on("SIGTERM", () => {
    if (lifetime === "wait") {
      process.exit(exitCode);
    }
    process.stdout.write("resisted termination\n");
  });
}
process.stdout.write(
  `${JSON.stringify({
    arguments: forwardedArguments,
    relay: process.env.DOTFILES_CODEX_PROFILE_RELAY,
    pid: process.pid,
    term: process.env.TERM,
  })}\n`,
);
if (lifetime !== undefined) {
  const lifetimeMilliseconds = 10_000;
  await Bun.sleep(lifetimeMilliseconds);
}
process.exitCode = exitCode;
