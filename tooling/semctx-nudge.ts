import { dirname, isAbsolute, join } from "node:path";
import { existsSync, statSync } from "node:fs";
import { z } from "zod";

const semctxTools = [
  "semctx_prepare_task",
  "semctx_change_open",
  "semctx_semantic_slice",
  "semctx_verify_change",
  "semctx_change_verify",
  "semctx_inspect",
];

const hostArguments = z.tuple([
  z.literal("--host"),
  z.enum(["claude", "codex"]),
]);

type Host = z.infer<typeof hostArguments>[1];

const sessionStartEvent = z.object({
  cwd: z.string().refine(isAbsolute, "cwd must be absolute"),
});

type SemctxNudgeIo = Readonly<{
  exists: (path: string) => boolean;
  isDirectory: (path: string) => boolean;
}>;

type SemctxNudgeOutcome = Readonly<{ stderr?: string; stdout?: string }>;

const fileSystemIo: SemctxNudgeIo = {
  exists: existsSync,
  isDirectory: (path) =>
    statSync(path, { throwIfNoEntry: false })?.isDirectory() ?? false,
};

function findGitRoot(start: string, io: SemctxNudgeIo): string | undefined {
  const parent = dirname(start);
  if (io.exists(join(start, ".git"))) {
    return start;
  }
  return parent === start ? undefined : findGitRoot(parent, io);
}

function toolLoadingStep(host: Host): string {
  if (host === "codex") {
    return "Discover its MCP tools (mcp__semctx__*) before concluding that the server is unavailable,";
  }
  const query = semctxTools.map((tool) => `mcp__plugin_semctx_semctx__${tool}`);
  return `Load its tools with ToolSearch query "select:${query.join(",")}",`;
}

function reminder(host: Host, gitRoot: string): string {
  return [
    `This Git checkout is semctx-enabled (${gitRoot}/.semctx).`,
    toolLoadingStep(host),
    `pass repositoryRoot "${gitRoot}", and follow the Semctx section of the global instructions:`,
    "change contract before a non-trivial edit, semctx_verify_change then semctx_change_verify",
    "before a commit or push.",
  ].join(" ");
}

function renderOutput(additionalContext: string): string {
  return JSON.stringify({
    hookSpecificOutput: { additionalContext, hookEventName: "SessionStart" },
  });
}

function runSemctxNudge(
  argv: readonly string[],
  input: string,
  io: SemctxNudgeIo = fileSystemIo,
): SemctxNudgeOutcome {
  try {
    const [, host] = hostArguments.parse(argv);
    const { cwd } = sessionStartEvent.parse(JSON.parse(input));
    const gitRoot = findGitRoot(cwd, io);
    if (gitRoot === undefined || !io.isDirectory(join(gitRoot, ".semctx"))) {
      return {};
    }
    return { stdout: renderOutput(reminder(host, gitRoot)) };
  } catch (error) {
    const reason = error instanceof Error ? error.message : String(error);
    return { stderr: `semctx-nudge disabled for this session: ${reason}` };
  }
}

export { type SemctxNudgeIo, type SemctxNudgeOutcome, runSemctxNudge };
