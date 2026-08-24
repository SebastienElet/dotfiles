#!/usr/bin/env bun

import { realpathSync } from "node:fs";
import { runFreshnessProbe } from "./codegraph/mcp-probe.ts";

const repositoryArgument = process.argv[2];
const binary = process.env.CODEGRAPH_BIN ?? Bun.which("codegraph");
if (
  repositoryArgument === undefined ||
  binary === null ||
  binary === undefined
) {
  process.stderr.write("usage: codegraph-mcp-probe REPOSITORY\n");
  process.exit(64);
}

try {
  const repository = realpathSync(repositoryArgument);
  if (process.env.CODEGRAPH_PROBE_PAUSE_MS !== undefined) {
    const pause = Number(process.env.CODEGRAPH_PROBE_PAUSE_MS);
    if (!Number.isSafeInteger(pause) || pause < 0 || pause > 10_000) {
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
  process.stderr.write(
    `${error instanceof Error ? error.message : String(error)}\n`,
  );
  process.exit(1);
}
