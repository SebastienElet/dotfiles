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
].map((tool) => `mcp__plugin_semctx_semctx__${tool}`);

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

function reminder(gitRoot: string): string {
  return [
    `This Git checkout is semctx-enabled (${gitRoot}/.semctx).`,
    `Load its tools with ToolSearch query "select:${semctxTools.join(",")}",`,
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
  input: string,
  io: SemctxNudgeIo = fileSystemIo,
): SemctxNudgeOutcome {
  try {
    const { cwd } = sessionStartEvent.parse(JSON.parse(input));
    const gitRoot = findGitRoot(cwd, io);
    if (gitRoot === undefined || !io.isDirectory(join(gitRoot, ".semctx"))) {
      return {};
    }
    return { stdout: renderOutput(reminder(gitRoot)) };
  } catch (error) {
    const reason = error instanceof Error ? error.message : String(error);
    return { stderr: `semctx-nudge disabled for this session: ${reason}` };
  }
}

export { type SemctxNudgeIo, type SemctxNudgeOutcome, runSemctxNudge };
