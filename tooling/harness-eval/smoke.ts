import type { Oracle } from "./contracts.ts";
import type { Report } from "./report-schema.ts";
import { loadCases } from "./sources.ts";
import { runSeries } from "./runner.ts";

const SMOKE_TIMEOUT_SECONDS = 120;
const commandsByOracle: Readonly<
  Record<Oracle, readonly (readonly string[])[]>
> = {
  "structural-v1": [
    ["cat", ".agents/skills/code-search/SKILL.md"],
    ["colgrep-search", "dependencies"],
  ],
  "literal-v1": [["rg", "FEATURE_FLAG_DISABLED"]],
  "known-path-v1": [["cat", "src/auth/session.ts"]],
};

function runSmoke(repository: string): Promise<Report> {
  const cases = loadCases(repository);
  return runSeries(
    repository,
    {
      agent: "fixture-smoke",
      agentVersion: "1",
      model: "none",
      only: cases.map((entry) => entry.definition.id),
      runs: 1,
      controls: {
        sandbox: "workspace-write",
        network: false,
        tools: "shell-with-synthetic-cat-rg-fd-colgrep-v1",
        timeoutSeconds: SMOKE_TIMEOUT_SECONDS,
        reasoningEffort: "low",
        tokenBudget: null,
      },
    },
    (fixture, prompt) => {
      const entry = cases.find((item) => item.prompt === prompt);
      if (entry === undefined) {
        throw new Error("Unknown smoke case");
      }
      const commands = commandsByOracle[entry.definition.oracle];
      for (const command of commands) {
        const result = Bun.spawnSync([...command], {
          cwd: fixture.workspace,
          env: fixture.env,
        });
        if (result.exitCode !== 0) {
          throw new Error(`Fixture smoke failed: ${command[0]}`);
        }
      }
      return {
        error: null,
        tokens: null,
        toolCalls: commands.length,
        durationMs: null,
      };
    },
  );
}

export { runSmoke };
