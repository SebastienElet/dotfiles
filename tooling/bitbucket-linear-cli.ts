import { type Command, createProviders } from "./bitbucket-linear-providers.ts";
import { type Config, configSchema } from "./bitbucket-linear-schema.ts";
import { type Plan, synchronize } from "./bitbucket-linear-run.ts";
import { runBounded } from "./bounded-command.ts";

type Options = Readonly<{ config: string; apply: boolean; json: boolean }>;
const usage =
  "bitbucket-linear-sync --config <file.json> [--apply] [--json]\nInspection is the default; --apply prints the complete plan before mutations.\n";

function options(arguments_: readonly string[]): Options | "help" {
  if (arguments_.length === 1 && arguments_[0] === "--help") {
    return "help";
  }
  const seen = new Set<string>();
  let config: string | undefined = undefined;
  for (let index = 0; index < arguments_.length; index += 1) {
    const argument = arguments_[index];
    if (argument === undefined) {
      throw new Error("Missing argument");
    }
    if (
      seen.has(argument) ||
      !["--config", "--apply", "--json"].includes(argument)
    ) {
      throw new Error("Invalid usage");
    }
    seen.add(argument);
    if (argument === "--config") {
      config = arguments_[(index += 1)];
      if (config === undefined || config === "" || config.startsWith("--")) {
        throw new Error("Missing config path");
      }
    }
  }
  if (config === undefined || config === "") {
    throw new Error("Explicit config required");
  }
  return { config, apply: seen.has("--apply"), json: seen.has("--json") };
}

const command: Command = async (name, arguments_) => {
  const outcome = await runBounded(name, arguments_, {
    cwd: process.cwd(),
    environment: process.env,
    timeoutMilliseconds: 30_000,
  });
  if (outcome.kind === "failed") {
    throw new Error(`${name} unavailable or timed out`);
  }
  if (outcome.status !== 0) {
    throw new Error(`${name} failed (exit ${outcome.status})`);
  }
  try {
    return JSON.parse(outcome.stdout) as unknown;
  } catch {
    throw new Error(`${name} returned invalid JSON`);
  }
};

function publicPlan(plan: Plan): Readonly<Record<string, unknown>> {
  return {
    phase: "plan",
    identity: plan.identity,
    observations: plan.observations,
    issues: plan.issues.map((entry) => ({
      issueId: entry.issue.id,
      identifier: entry.issue.identifier,
      teamId: entry.issue.teamId,
      observedState: entry.issue.state,
      problems: entry.problems,
      pullRequests: entry.pullRequests.map((pr) => ({
        id: pr.id,
        url: pr.url,
        state: pr.state,
        repository: pr.repository,
        sourceRepository: pr.sourceRepository,
        sourceRepositoryId: pr.sourceRepositoryId,
        destinationRepositoryId: pr.destinationRepositoryId,
        authorId: pr.authorId,
      })),
      attachments: entry.attachments,
      completeStateId: entry.completeStateId,
    })),
  };
}

async function output(text: string): Promise<void> {
  await new Promise<void>((resolve, reject) => {
    process.stdout.write(text, (error: Readonly<Error> | null | undefined) => {
      if (error === undefined || error === null) {
        resolve();
        return;
      }
      reject(error);
    });
  });
}

const usageError = 2;
const indentation = 2;
async function execute(config: Config, parsed: Options): Promise<number> {
  try {
    const result = await synchronize(config, createProviders(config, command), {
      apply: parsed.apply,
      announce: async (plan) => {
        const view = publicPlan(plan);
        await output(
          parsed.json
            ? `${JSON.stringify(view)}\n`
            : `Plan\n${JSON.stringify(view, null, indentation)}\n`,
        );
      },
    });
    const text = parsed.json
      ? JSON.stringify({
          phase: "result",
          mode: parsed.apply ? "apply" : "inspect",
          observations: result.observations,
          exitCode: result.exitCode,
        })
      : result.observations
          .map(
            (event) =>
              `${event.status}: ${event.identifier ?? event.repository ?? event.url ?? "inventory"}${event.reason === undefined ? "" : ` (${event.reason})`}`,
          )
          .join("\n");
    await output(`${text}\n`);
    return result.exitCode;
  } catch {
    process.stderr.write(
      "bitbucket-linear-sync: provider/inventory/output failed; inspect authentication, command availability and provider health\n",
    );
    return 1;
  }
}
async function main(arguments_: readonly string[]): Promise<number> {
  try {
    const parsed = options(arguments_);
    if (parsed === "help") {
      await output(usage);
      return 0;
    }
    const config = configSchema.parse(await Bun.file(parsed.config).json());
    return await execute(config, parsed);
  } catch {
    process.stderr.write(
      "bitbucket-linear-sync: usage/configuration invalid; use --help\n",
    );
    return usageError;
  }
}
export { main };
