#!/usr/bin/env bun

import { appendFileSync, lstatSync } from "node:fs";
import { z } from "zod";

const filesSchema = z.array(z.string());
const commandArgumentOffset = 2;
const invalidByte = 0xff;
const lineFeedByte = 0x0a;
const gitFailureExitCode = 7;
const gitNotRepositoryExitCode = 128;

const providerArguments = process.argv.slice(commandArgumentOffset);
const logPath = process.env.CODEGRAPH_TEST_ARGUMENTS_LOG;
if (logPath !== undefined) {
  appendFileSync(logPath, `${JSON.stringify(providerArguments)}\n`);
}

if (providerArguments.includes("rev-parse")) {
  finishGit("rev-parse", "true\n");
}

if (providerArguments.includes("ls-files")) {
  if (process.env.CODEGRAPH_TEST_GIT_INVALID_UTF8 === "1") {
    process.stdout.write(new Uint8Array([invalidByte, 0]));
    process.exit(0);
  }
  const files = process.env.CODEGRAPH_TEST_GIT_FILES_JSON;
  finishGit(
    "ls-files",
    files === undefined
      ? (process.env.CODEGRAPH_TEST_GIT_FILES ?? "")
      : `${filesSchema.parse(JSON.parse(files)).join("\0")}\0`,
  );
}

const failure = process.env.CODEGRAPH_TEST_TOKEI_FAILURE;
if (failure !== undefined) {
  process.stdout.write('{"partial":true}\n');
  process.stderr.write("tokei operational failure\n");
  process.exit(Number(failure));
}

const streamingIndex = providerArguments.indexOf("--streaming");
const inputs =
  streamingIndex === -1 ? [] : providerArguments.slice(0, streamingIndex);
if (
  providerArguments[0] !== undefined &&
  lstatSync(providerArguments[0]).isDirectory()
) {
  if (process.env.CODEGRAPH_TEST_TOKEI_INVALID_UTF8 === "1") {
    process.stdout.write(new Uint8Array([invalidByte, lineFeedByte]));
    process.exit(0);
  }
  process.stdout.write(process.env.CODEGRAPH_TEST_TOKEI_OUTPUT ?? "");
  process.exit(0);
}

for (const input of inputs) {
  process.stdout.write(record(input, 1));
}

function finishGit(operation: string, output: string): never {
  if (process.env.CODEGRAPH_TEST_GIT_FAILURE === operation) {
    process.stdout.write(operation === "ls-files" ? "partial.tofu\0" : "");
    process.stderr.write(`${operation} operational failure\n`);
    process.exit(gitFailureExitCode);
  }
  if (
    operation === "rev-parse" &&
    process.env.CODEGRAPH_TEST_GIT_REPOSITORY !== "1"
  ) {
    process.stderr.write("fatal: not a git repository (or any parent): .git\n");
    process.exit(gitNotRepositoryExitCode);
  }
  process.stdout.write(output);
  process.exit(0);
}

function record(name: string, code: number): string {
  return `${JSON.stringify({
    language: "TypeScript",
    stats: { name, stats: { code } },
  })}\n`;
}
