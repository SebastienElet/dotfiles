import { appendFileSync, readFileSync, realpathSync } from "node:fs";
import { relative, resolve } from "node:path";

const ARGUMENT_OFFSET = 2;

function invoke(
  tool: string,
  args: readonly string[],
): { output: string; exitCode: number } {
  if (tool === "cat") {
    try {
      return {
        output: args
          .map((path) => readFileSync(resolve(path), "utf8"))
          .join(""),
        exitCode: 0,
      };
    } catch {
      return { output: "", exitCode: 1 };
    }
  }
  if (tool === "rg" && args.includes("FEATURE_FLAG_DISABLED")) {
    return {
      output: "src/flags.ts:1:export const FEATURE_FLAG_DISABLED = false;\n",
      exitCode: 0,
    };
  }
  if (tool === "fd" || (tool === "rg" && args.includes("--files"))) {
    return {
      output:
        "packages/app/package.json\npackages/auth/package.json\nsrc/auth/session.ts\nsrc/flags.ts\n",
      exitCode: 0,
    };
  }
  if (
    tool === "colgrep-search" &&
    args.some((argument) => !argument.startsWith("-"))
  ) {
    return {
      output: `${JSON.stringify({
        results: [
          {
            path: "packages/app/package.json",
            content: '{"name":"app","dependencies":{"auth":"workspace:*"}}',
          },
          { path: "packages/auth/package.json", content: '{"name":"auth"}' },
        ],
      })}\n`,
      exitCode: 0,
    };
  }
  return { output: "Unsupported synthetic tool invocation\n", exitCode: 64 };
}

export function invokeShim(tool: string): void {
  const args = process.argv.slice(ARGUMENT_OFFSET);
  const workspace = process.env.HARNESS_EVAL_WORKSPACE;
  const log = process.env.HARNESS_EVAL_OBSERVATIONS;
  if (workspace === undefined || log === undefined) {
    throw new Error("Missing fixture instrumentation");
  }
  const normalized = args.map((argument) => {
    if (tool !== "cat") {
      return argument;
    }
    try {
      return relative(realpathSync(workspace), realpathSync(resolve(argument)));
    } catch {
      return "<unresolved>";
    }
  });
  const result = invoke(tool, args);
  const publicArguments = normalized.map((argument) =>
    [
      ".agents/skills/code-search/SKILL.md",
      "src/auth/session.ts",
      "FEATURE_FLAG_DISABLED",
      "--files",
    ].includes(argument)
      ? argument
      : "<other>",
  );
  appendFileSync(
    log,
    `${JSON.stringify({
      tool,
      args: publicArguments,
      exitCode: result.exitCode,
    })}\n`,
  );
  process.stdout.write(result.output);
  process.exitCode = result.exitCode;
}
