import type { Agent, ProcessOutput } from "./agent-memory-eval-process.ts";
import {
  parseAgentSession,
  parseEvaluationTrace,
} from "./agent-memory-eval-contract.ts";
import { join } from "node:path";

function requireRuntimeTrace(
  ...[
    agent,
    trace,
    condition,
    expectedExitClass = "success",
    expectedCommand = "hook",
  ]: readonly [Agent, string | undefined, string, string?, string?]
): void {
  try {
    parseEvaluationTrace(
      agent,
      trace ?? "",
      expectedExitClass,
      expectedCommand,
    );
  } catch (error) {
    const detail =
      error instanceof Error ? error.message : "invalid runtime trace";
    throw new Error(`${condition}: ${detail}`, { cause: error });
  }
}

function relevantTrace(
  ...[agent, output, nonce, runtime, store, version]: readonly [
    Agent,
    ProcessOutput,
    string,
    string,
    string,
    string,
  ]
): ReturnType<typeof parseAgentSession> {
  try {
    return parseAgentSession(
      agent,
      output.stdout,
      {
        cache: join(store, "oracle-cache.json"),
        nonce,
        runtime,
        source: "proof.txt",
        version,
      },
      {
        ...(output.cacheAbsentBefore === undefined
          ? {}
          : { cacheAbsentBefore: output.cacheAbsentBefore }),
        ...(output.cacheCompletedBeforeModel === undefined
          ? {}
          : { cacheCompletedBeforeModel: output.cacheCompletedBeforeModel }),
        ...(output.cachePath === undefined
          ? {}
          : { cachePath: output.cachePath }),
        ...(output.runtimeTrace === undefined
          ? {}
          : { runtimeTrace: output.runtimeTrace }),
        ...(output.traceCompletedBeforeModel === undefined
          ? {}
          : { traceCompletedBeforeModel: output.traceCompletedBeforeModel }),
        version,
      },
    );
  } catch (error) {
    const detail =
      error instanceof Error ? error.message : "invalid relevant session";
    throw new Error(`relevant: ${detail}`, { cause: error });
  }
}

export { relevantTrace, requireRuntimeTrace };
