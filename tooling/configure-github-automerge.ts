import type { GithubApi, GithubRequest } from "./github-automerge-plan.ts";
import { planGithubAutomerge } from "./github-automerge-plan.ts";
import { z } from "zod";

const argumentOffset = 2;
const ghTimeoutMilliseconds = 30_000;
const invocationSchema = z.tuple([z.enum(["--dry-run", "--apply", "--help"])]);
const usage =
  "Usage: bun tooling/configure-github-automerge.ts --dry-run | --apply | --help";

async function ghApi(request: GithubRequest): Promise<unknown> {
  const commandArguments = [
    "api",
    "--hostname",
    "github.com",
    "--method",
    request.method,
    request.endpoint,
  ];
  if (request.paginate === true) {
    commandArguments.push("--paginate", "--slurp");
  }
  if (request.body !== undefined) {
    commandArguments.push("--input", "-");
  }
  const child = Bun.spawn(["gh", ...commandArguments], {
    stdin:
      request.body === undefined
        ? "ignore"
        : new Blob([JSON.stringify(request.body)]),
    stdout: "pipe",
    stderr: "pipe",
    timeout: ghTimeoutMilliseconds,
  });
  const [status, output, errors] = await Promise.all([
    child.exited,
    new Response(child.stdout).text(),
    new Response(child.stderr).text(),
  ]);
  if (status !== 0 || child.signalCode !== null) {
    throw new Error(
      `gh ${request.method} ${request.endpoint} failed (${child.signalCode ?? status}): ${errors.trim()}`,
    );
  }
  return JSON.parse(output);
}

async function configureGithubAutomerge(
  arguments_: readonly string[],
  api: GithubApi,
  report: (message: string) => void,
): Promise<void> {
  const invocation = invocationSchema.safeParse(arguments_);
  if (!invocation.success) {
    throw new Error(usage);
  }
  const [mode] = invocation.data;
  if (mode === "--help") {
    report(usage);
    return;
  }
  const plans = await planGithubAutomerge(api);
  let changeCount = 0;
  for (const plan of plans) {
    report(`${plan.repository} (${plan.branch}): ${plan.contexts.join(", ")}`);
    for (const change of plan.changes) {
      report(
        `gh api --hostname github.com --method ${change.method} ${change.endpoint} --input -\n${JSON.stringify(change.body)}`,
      );
      if (mode === "--apply") {
        await api(change);
      }
      changeCount += 1;
    }
  }
  report(
    `${mode === "--dry-run" ? "Dry run" : "Applied"}: ${plans.length} repositories, ${changeCount} changes${mode === "--dry-run" ? "; no writes" : ""}.`,
  );
}

if (import.meta.main) {
  try {
    await configureGithubAutomerge(
      Bun.argv.slice(argumentOffset),
      ghApi,
      (message) => {
        process.stdout.write(`${message}\n`);
      },
    );
  } catch (error) {
    process.stderr.write(
      `${error instanceof Error ? error.message : String(error)}\n`,
    );
    process.exitCode = 1;
  }
}

export { configureGithubAutomerge };
