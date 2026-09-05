import { type Executor, type SeriesOptions, runSeries } from "./runner.ts";
import { copyFileSync, existsSync } from "node:fs";
import { dirname, join } from "node:path";
import type { Report } from "./report-schema.ts";
import { capture } from "./process.ts";
import { parseCodexEvents } from "./codex.ts";

const VERSION_TIMEOUT_MS = 5000;
type Authentication = Readonly<{ file?: string; apiKey?: string }>;

function codexArguments(model: string, reasoningEffort: string): string[] {
  return [
    "exec",
    "--json",
    "--ephemeral",
    "--ignore-user-config",
    "--ignore-rules",
    "--skip-git-repo-check",
    "--model",
    model,
    "--sandbox",
    "workspace-write",
    "-c",
    'approval_policy="never"',
    "-c",
    "sandbox_workspace_write.network_access=false",
    "-c",
    'web_search="disabled"',
    "-c",
    `model_reasoning_effort="${reasoningEffort}"`,
    "-c",
    'shell_environment_policy.inherit="all"',
    "-c",
    "shell_environment_policy.ignore_default_excludes=false",
    "-c",
    "shell_environment_policy.experimental_use_profile=false",
    "-",
  ];
}

function codexExecutor(
  command: string,
  options: SeriesOptions,
  authentication: Authentication,
): Executor {
  return async (fixture, prompt) => {
    if (authentication.file !== undefined) {
      copyFileSync(
        authentication.file,
        join(fixture.env.CODEX_HOME, "auth.json"),
      );
    }
    const env = {
      ...fixture.env,
      PATH: `${fixture.env.PATH}:${dirname(command)}`,
      ...(authentication.apiKey === undefined
        ? {}
        : { CODEX_API_KEY: authentication.apiKey }),
    };
    const started = performance.now();
    const result = await capture(
      command,
      codexArguments(options.model, options.controls.reasoningEffort),
      {
        cwd: fixture.workspace,
        env,
        stdin: prompt,
        timeoutSeconds: options.controls.timeoutSeconds,
      },
    );
    const durationMs = Math.round(performance.now() - started);
    if (result.error !== null) {
      return { error: result.error, durationMs, tokens: null, toolCalls: null };
    }
    try {
      return { ...parseCodexEvents(result.output), error: null, durationMs };
    } catch {
      return {
        error: "protocol-invalid",
        durationMs,
        tokens: null,
        toolCalls: null,
      };
    }
  };
}

function authenticationForEval(): Authentication {
  const apiKey = process.env.CODEX_API_KEY;
  if (apiKey !== undefined && apiKey !== "") {
    return { apiKey };
  }
  const authFile = join(
    process.env.CODEX_HOME ?? join(process.env.HOME ?? "", ".codex"),
    "auth.json",
  );
  if (existsSync(authFile)) {
    return { file: authFile };
  }
  throw new Error("Manual eval requires saved Codex auth or CODEX_API_KEY");
}

function runLive(
  repository: string,
  options: Omit<SeriesOptions, "agent" | "agentVersion">,
): Promise<Report> {
  const command = Bun.which("codex");
  if (command === null) {
    throw new Error("Codex CLI is not installed");
  }
  const authentication = authenticationForEval();
  const version = Bun.spawnSync([command, "--version"], {
    timeout: VERSION_TIMEOUT_MS,
  });
  if (version.exitCode !== 0) {
    throw new Error("Cannot identify Codex version");
  }
  const series = {
    ...options,
    agent: "codex",
    agentVersion: version.stdout.toString().trim(),
  };
  return runSeries(
    repository,
    series,
    codexExecutor(command, series, authentication),
  );
}

export { codexArguments, codexExecutor, runLive };
