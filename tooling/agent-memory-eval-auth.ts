import type { Agent } from "./agent-memory-eval-process.ts";
import { installAdapter as installRuntimeAdapter } from "./agent-memory-eval-auth-adapter.ts";
import { installCredential as installRuntimeCredential } from "./agent-memory-eval-auth-credentials.ts";

async function prepareAgent(
  ...[agent, home, runtime, environment]: readonly [
    Agent,
    string,
    string,
    Readonly<NodeJS.ProcessEnv>,
  ]
): Promise<void> {
  await installRuntimeCredential(agent, home, environment);
  await installRuntimeAdapter(agent, home, runtime);
}

export {
  buildAgentCommand,
  claudeTemporaryDirectory,
} from "./agent-memory-eval-auth-command.ts";
export { installAdapter } from "./agent-memory-eval-auth-adapter.ts";
export {
  installCredential,
  withCursorAuthentication,
} from "./agent-memory-eval-auth-credentials.ts";
export { prepareAgent };
