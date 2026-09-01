import { join } from "node:path";

import {
  parseAgentSession,
  parseEvaluationTrace,
} from "./agent-memory-eval-contract.ts";
import type { Agent, ProcessOutput } from "./agent-memory-eval-process.ts";

function requireRuntimeTrace(
  agent: Agent,
  trace: string | undefined,
  condition: string,
  expectedExitClass = "success",
  expectedCommand = "hook",
): void {
  try {
    parseEvaluationTrace(agent, trace ?? "", expectedExitClass, expectedCommand);
  } catch (error) {
    const detail = error instanceof Error ? error.message : "invalid runtime trace";
    throw new Error(`${condition}: ${detail}`);
  }
}

function relevantTrace(
  agent: Agent,
  output: ProcessOutput,
  nonce: string,
  runtime: string,
  store: string,
  version: string,
) {
  try {
    return parseAgentSession(
      agent,
      output.stdout,
      { cache: join(store, "oracle-cache.json"), nonce, runtime, source: "proof.txt", version },
      {
        ...(output.cacheAbsentBefore === undefined
          ? {}
          : { cacheAbsentBefore: output.cacheAbsentBefore }),
        ...(output.cacheCompletedBeforeModel === undefined
          ? {}
          : { cacheCompletedBeforeModel: output.cacheCompletedBeforeModel }),
        ...(output.cachePath === undefined ? {} : { cachePath: output.cachePath }),
        ...(output.runtimeTrace === undefined ? {} : { runtimeTrace: output.runtimeTrace }),
        ...(output.traceCompletedBeforeModel === undefined
          ? {}
          : { traceCompletedBeforeModel: output.traceCompletedBeforeModel }),
        version,
      },
    );
  } catch (error) {
    const detail = error instanceof Error ? error.message : "invalid relevant session";
    throw new Error(`relevant: ${detail}`);
  }
}

export { relevantTrace, requireRuntimeTrace };
