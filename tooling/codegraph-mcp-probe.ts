#!/usr/bin/env bun

import { realpathSync } from "node:fs";
import { runFreshnessProbe } from "./codegraph/mcp-probe.ts";

const binary = process.env.CODEGRAPH_BIN ?? Bun.which("codegraph");
const firstArgumentIndex = 2;
const maximumPauseMilliseconds = 10_000;
const [repositoryArgument] = process.argv.slice(firstArgumentIndex);
const usageExitCode = 64;
if (
  process.argv.length <= firstArgumentIndex ||
  repositoryArgument === "" ||
  binary === null ||
  !binary
) {
  process.stderr.write("usage: codegraph-mcp-probe REPOSITORY\n");
  process.exit(usageExitCode);
}

try {
  const repository = realpathSync(repositoryArgument ?? "");
  if ("CODEGRAPH_PROBE_PAUSE_MS" in process.env) {
    const pause = Number(process.env.CODEGRAPH_PROBE_PAUSE_MS);
    if (
      !Number.isSafeInteger(pause) ||
      pause < Number.MIN_SAFE_INTEGER + Math.abs(Number.MIN_SAFE_INTEGER) ||
      pause > maximumPauseMilliseconds
    ) {
      throw new Error(
        `invalid probe pause: ${process.env.CODEGRAPH_PROBE_PAUSE_MS}`,
      );
    }
    await Bun.sleep(pause);
  }
  process.stdout.write(
    `${JSON.stringify(await runFreshnessProbe(repository, [binary]))}\n`,
  );
} catch (error) {
  let message = String(error);
  if (error instanceof Error) {
    ({ message } = error);
  }
  process.stderr.write(`${message}\n`);
  process.exitCode = 1;
}
