import { isAbsolute, resolve } from "node:path";
import { z } from "zod";

const absolutePathSchema = z
  .string()
  .min(1)
  .refine(isAbsolute, "file path must be absolute");

const claudeFileToolNameSchema = z.enum(["Edit", "Write", "MultiEdit"]);

const claudeFileToolSchema = z.object({
  tool_input: z.object({ file_path: absolutePathSchema }),
  tool_name: claudeFileToolNameSchema,
});

const codexPatchSchema = z.object({
  cwd: absolutePathSchema,
  tool_input: z.object({ command: z.string() }),
  tool_name: z.literal("apply_patch"),
});

const hookInputSchema = z.object({
  hook_event_name: z.literal("PostToolUse"),
  tool_name: z.string(),
});

const editedPatchHeader =
  /^\*\*\* (?<action>Add File|Update File|Move to): (?<path>.+)$/u;

type PatchHeader = Readonly<{ action: string; path: string }>;

type EditedPaths =
  | Readonly<{ paths: readonly string[]; success: true }>
  | Readonly<{ reason: string; success: false }>;

function parseJson(stdin: string): unknown {
  try {
    return JSON.parse(stdin);
  } catch {
    return undefined;
  }
}

function patchedPaths(cwd: string, patch: string): readonly string[] {
  const headers = patch.split("\n").flatMap((line): readonly PatchHeader[] => {
    const groups = editedPatchHeader.exec(line.trimEnd())?.groups;
    return groups?.action === undefined || groups.path === undefined
      ? []
      : [
          {
            action: groups.action,
            path: resolve(cwd, groups.path.trim()),
          },
        ];
  });
  return headers.flatMap((header, index) => {
    if (header.action === "Move to") {
      return [];
    }
    const next = headers[index + 1];
    return [next?.action === "Move to" ? next.path : header.path];
  });
}

function editedPaths(stdin: string): EditedPaths {
  const value = parseJson(stdin);
  if (value === undefined) {
    return { reason: "input is not JSON", success: false };
  }
  const hookInput = hookInputSchema.safeParse(value);
  if (!hookInput.success) {
    return { reason: z.prettifyError(hookInput.error), success: false };
  }
  if (hookInput.data.tool_name === "apply_patch") {
    const patch = codexPatchSchema.safeParse(value);
    return patch.success
      ? {
          paths: patchedPaths(patch.data.cwd, patch.data.tool_input.command),
          success: true,
        }
      : { reason: z.prettifyError(patch.error), success: false };
  }
  if (!claudeFileToolNameSchema.safeParse(hookInput.data.tool_name).success) {
    return { paths: [], success: true };
  }
  const fileTool = claudeFileToolSchema.safeParse(value);
  return fileTool.success
    ? { paths: [fileTool.data.tool_input.file_path], success: true }
    : { reason: z.prettifyError(fileTool.error), success: false };
}

export { editedPaths };
