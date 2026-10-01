#!/usr/bin/env bun
import { appendFileSync } from "node:fs";
import { basename } from "node:path";
import { z } from "zod";

const failureStatus = 9;
const environmentSchema = z.object({
  FAKE_TRACE: z.string().min(1),
  FAKE_FAIL_ON: z.string().min(1).optional(),
  FAKE_APPEND_TO: z.string().min(1).optional(),
});
const [, executable = "", ...commandArguments] = process.argv;
const environment = environmentSchema.parse(process.env);

appendFileSync(
  environment.FAKE_TRACE,
  `${JSON.stringify({ command: basename(executable), arguments: commandArguments })}\n`,
);
if (environment.FAKE_APPEND_TO !== undefined) {
  appendFileSync(environment.FAKE_APPEND_TO, "mutated by the fake command\n");
}
if (
  environment.FAKE_FAIL_ON !== undefined &&
  commandArguments.join(" ").includes(environment.FAKE_FAIL_ON)
) {
  process.exitCode = failureStatus;
}
