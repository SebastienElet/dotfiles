#!/usr/bin/env bun

const commandArgumentOffset = 2;
const commandArguments = process.argv.slice(commandArgumentOffset);
const mode = process.env.COLGREP_TEST_GIT_MODE ?? "healthy";
const projectRoot = requiredEnvironment("COLGREP_TEST_PROJECT_ROOT");

if (commandArguments.includes("--show-toplevel")) {
  if (mode === "empty-root") {
    process.stdout.write("");
  } else if (mode === "multiple-root") {
    process.stdout.write(`${projectRoot}\n${projectRoot}\n`);
  } else {
    process.stdout.write(`${projectRoot}\n`);
  }
} else if (commandArguments.includes("--show-superproject-working-tree")) {
  process.stdout.write(mode === "superproject" ? `${projectRoot}\n` : "");
} else {
  process.stderr.write(
    `unexpected Git invocation: ${commandArguments.join(" ")}\n`,
  );
  process.exitCode = 64;
}

function requiredEnvironment(name: string): string {
  const value = process.env[name];
  if (value === undefined || value === "") {
    throw new Error(`${name} is required`);
  }
  return value;
}

export { requiredEnvironment };
