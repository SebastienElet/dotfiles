#!/usr/bin/env bun

import { appendFileSync, lstatSync } from "node:fs";

const arguments_ = process.argv.slice(2);
const logPath = process.env.CODEGRAPH_TEST_ARGUMENTS_LOG;
if (logPath !== undefined) {
  appendFileSync(logPath, `${JSON.stringify(arguments_)}\n`);
}

if (arguments_.includes("rev-parse")) {
  finishGit("rev-parse", "true\n");
}

if (arguments_.includes("ls-files")) {
  const files = process.env.CODEGRAPH_TEST_GIT_FILES_JSON;
  finishGit(
    "ls-files",
    files === undefined
      ? (process.env.CODEGRAPH_TEST_GIT_FILES ?? "")
      : `${(JSON.parse(files) as string[]).join("\0")}\0`,
  );
}

const failure = process.env.CODEGRAPH_TEST_TOKEI_FAILURE;
if (failure !== undefined) {
  process.stdout.write('{"partial":true}\n');
  console.error("tokei operational failure");
  process.exit(Number(failure));
}

const streamingIndex = arguments_.indexOf("--streaming");
const inputs = streamingIndex === -1 ? [] : arguments_.slice(0, streamingIndex);
if (arguments_[0] !== undefined && lstatSync(arguments_[0]).isDirectory()) {
  process.stdout.write(process.env.CODEGRAPH_TEST_TOKEI_OUTPUT ?? "");
  process.exit(0);
}

for (const input of inputs) {
  process.stdout.write(record(input, 1));
}

function finishGit(operation: string, output: string): never {
  if (process.env.CODEGRAPH_TEST_GIT_FAILURE === operation) {
    process.stdout.write(operation === "ls-files" ? "partial.tofu\0" : "");
    console.error(`${operation} operational failure`);
    process.exit(7);
  }
  if (
    operation === "rev-parse" &&
    process.env.CODEGRAPH_TEST_GIT_REPOSITORY !== "1"
  ) {
    console.error("fatal: not a git repository (or any parent): .git");
    process.exit(128);
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
