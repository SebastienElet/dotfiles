import {
  type Agent,
  runEvaluationProcess,
  runManagedProcess,
  runManagedProcessToFile,
} from "./agent-memory-eval-process.ts";
import { chmod, copyFile, mkdir, rm } from "node:fs/promises";
import { dirname, join } from "node:path";
import { homedir } from "node:os";

const credentialLookupTimeoutMilliseconds = 10_000;
const privateFileMode = 0o600;
const processIdTimeoutMilliseconds = 5000;

async function installCredential(
  agent: Agent,
  home: string,
  environment: Readonly<NodeJS.ProcessEnv>,
): Promise<void> {
  if (agent === "cursor") {
    return;
  }
  const destination = credentialDestination(agent, home);
  await mkdir(dirname(destination), { mode: 0o700, recursive: true });
  await (agent === "codex"
    ? copyCodexCredential(destination)
    : copyClaudeCredential(destination, environment));
  await chmod(destination, privateFileMode);
}

function credentialDestination(
  agent: Exclude<Agent, "cursor">,
  home: string,
): string {
  return agent === "codex"
    ? join(home, ".codex", "auth.json")
    : join(home, ".claude", ".credentials.json");
}

async function copyCodexCredential(destination: string): Promise<void> {
  try {
    await copyFile(join(homedir(), ".codex", "auth.json"), destination);
  } catch {
    throw new Error("Codex authentication unavailable");
  }
}

async function copyClaudeCredential(
  destination: string,
  environment: Readonly<NodeJS.ProcessEnv>,
): Promise<void> {
  const identity = await runEvaluationProcess(
    ["id", "-un"],
    environment,
    processIdTimeoutMilliseconds,
  );
  try {
    await runManagedProcessToFile(
      [
        "/usr/bin/security",
        "find-generic-password",
        "-w",
        "-s",
        "Claude Code-credentials",
        "-a",
        identity.stdout.trim(),
      ],
      process.env,
      destination,
      credentialLookupTimeoutMilliseconds,
    );
  } catch {
    await rm(destination, { force: true });
    throw new Error("Claude authentication unavailable");
  }
}

async function withCursorAuthentication<Result>(
  environment: Readonly<NodeJS.ProcessEnv>,
  operation: (authenticated: Readonly<NodeJS.ProcessEnv>) => Promise<Result>,
): Promise<Result> {
  await assertCursorStatus(process.env);
  const token = await cursorToken();
  const authenticated = { ...environment, CURSOR_AUTH_TOKEN: token };
  await assertCursorStatus(authenticated);
  return operation(authenticated);
}

async function assertCursorStatus(
  environment: Readonly<NodeJS.ProcessEnv>,
): Promise<void> {
  await runEvaluationProcess(
    ["cursor-agent", "status", "--format", "json"],
    environment,
    credentialLookupTimeoutMilliseconds,
  );
}

async function cursorToken(): Promise<string> {
  const credential = await runManagedProcess({
    command: [
      "/usr/bin/security",
      "find-generic-password",
      "-w",
      "-a",
      "cursor-user",
      "-s",
      "cursor-access-token",
    ],
    environment: process.env,
    timeoutMilliseconds: credentialLookupTimeoutMilliseconds,
  });
  const token = credential.stdout.trim();
  if (token === "") {
    throw new Error("Cursor authentication unavailable");
  }
  return token;
}

export { installCredential, withCursorAuthentication };
