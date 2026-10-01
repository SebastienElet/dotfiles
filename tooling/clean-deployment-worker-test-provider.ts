#!/usr/bin/env bun
import { basename, join } from "node:path";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { z } from "zod";

const argumentOffset = 2;
const nativeFailureStatus = 9;
const blockedCommandMilliseconds = 20_000;
const environment = z
  .object({
    CLEAN_WORKER_TEST_STATE: z.string().min(1),
    HOME: z.string().min(1),
  })
  .parse(process.env);
const stateSchema = z.looseObject({
  definition: z.unknown().nullable(),
  failure: z
    .enum(["inspect", "bootout", "convert", "retained", "invalid", "timeout"])
    .optional(),
  trace: z.array(
    z.looseObject({ command: z.string(), configPresent: z.boolean() }),
  ),
});
const state = stateSchema.parse(
  JSON.parse(readFileSync(environment.CLEAN_WORKER_TEST_STATE, "utf8")),
);
const command = basename(process.argv[1] ?? "");
const args = process.argv.slice(argumentOffset);
state.trace.push({
  command,
  configPresent: existsSync(join(environment.HOME, ".remem/config.toml")),
});
writeFileSync(environment.CLEAN_WORKER_TEST_STATE, JSON.stringify(state));
if (command === "osascript" && state.failure === "timeout") {
  await Bun.sleep(blockedCommandMilliseconds);
}
if (
  (command === "osascript" && state.failure === "inspect") ||
  (command === "launchctl" && state.failure === "bootout") ||
  (command === "plutil" && state.failure === "convert")
) {
  process.stderr.write("native test failure\n");
  process.exit(nativeFailureStatus);
}
if (command === "plutil") {
  process.stdout.write(readFileSync(args.at(-1) ?? "", "utf8"));
} else if (command === "osascript") {
  process.stdout.write(
    state.failure === "invalid"
      ? '{"state":"unknown"}'
      : JSON.stringify(
          state.definition === null
            ? { state: "absent" }
            : { state: "loaded", definition: state.definition },
        ),
  );
} else if (command === "launchctl" && args[0] === "bootout") {
  if (state.failure !== "retained") {
    state.definition = null;
    writeFileSync(environment.CLEAN_WORKER_TEST_STATE, JSON.stringify(state));
  }
} else {
  throw new Error("unexpected native command");
}
