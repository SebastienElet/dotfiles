import { modelText } from "./agent-memory-eval-contract.ts";
import { readFile } from "node:fs/promises";

async function readTrace(path: string): Promise<string> {
  try {
    return await readFile(path, "utf8");
  } catch {
    return "";
  }
}

async function traceHasCompletion(path: string): Promise<boolean> {
  const trace = await readTrace(path);
  return trace.split("\n").some((line) => line.includes('"event":"completed"'));
}

function nonceModelLine(
  environment: Readonly<NodeJS.ProcessEnv>,
  line: string,
  nonce: string,
): boolean {
  const agent = environment.AGENT_MEMORY_EVAL_AGENT;
  if (agent !== "codex" && agent !== "claude" && agent !== "cursor") {
    return false;
  }
  try {
    const event: unknown = JSON.parse(line);
    return isRecord(event) && modelText(agent, event).includes(nonce);
  } catch {
    return false;
  }
}

function isRecord(value: unknown): value is Readonly<Record<string, unknown>> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

export { nonceModelLine, readTrace, traceHasCompletion };
