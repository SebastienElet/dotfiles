import { dirname, join } from "node:path";
import {
  format as formatWithPrettier,
  getFileInfo,
  resolveConfig,
} from "prettier";
import type { EditedFile } from "./format-edited-file-location.ts";
import { format as formatWithOxfmt } from "oxfmt";
import oxfmtConfig from "../oxfmt.config.ts";

type Environment = Readonly<Record<string, string | undefined>>;

type FormatContext = Readonly<{
  environment: Environment;
  file: EditedFile;
  timeoutMilliseconds: number;
}>;

type Transformation =
  | Readonly<{ code: string; kind: "formatted" }>
  | Readonly<{ kind: "ignored" }>
  | Readonly<{ kind: "failed"; reason: string }>;

type Formatter = Readonly<{
  name: string;
  transform: (
    source: string,
    context: FormatContext,
  ) => Promise<Transformation>;
}>;

type PipeCommand = Readonly<{ arguments: readonly string[]; name: string }>;

function failureMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

async function formatTypeScript(
  source: string,
  context: FormatContext,
): Promise<Transformation> {
  const result = await formatWithOxfmt(
    context.file.relativePath,
    source,
    oxfmtConfig,
  );
  if (result.errors.length > 0) {
    const messages: string[] = [];
    for (const error of result.errors) {
      messages.push(error.message);
    }
    return { kind: "failed", reason: `oxfmt failed: ${messages.join("; ")}` };
  }
  return { code: result.code, kind: "formatted" };
}

async function formatWithPrettierConfiguration(
  source: string,
  context: FormatContext,
): Promise<Transformation> {
  const { absolutePath, root } = context.file;
  try {
    const info = await getFileInfo(absolutePath, {
      ignorePath: [join(root, ".gitignore"), join(root, ".prettierignore")],
    });
    if (info.ignored) {
      return { kind: "ignored" };
    }
    const config = await resolveConfig(absolutePath);
    const code = await formatWithPrettier(source, {
      ...config,
      filepath: absolutePath,
    });
    return { code, kind: "formatted" };
  } catch (error) {
    return {
      kind: "failed",
      reason: `prettier failed: ${failureMessage(error)}`,
    };
  }
}

async function pipeThrough(
  command: PipeCommand,
  source: string,
  context: FormatContext,
): Promise<Transformation> {
  const executable = Bun.which(command.name, {
    PATH: context.environment.PATH ?? "",
  });
  if (executable === null) {
    return { kind: "failed", reason: `${command.name} is not installed` };
  }
  const child = Bun.spawn([executable, ...command.arguments], {
    cwd: dirname(context.file.absolutePath),
    env: { ...context.environment },
    stderr: "pipe",
    stdin: new TextEncoder().encode(source),
    stdout: "pipe",
    timeout: context.timeoutMilliseconds,
  });
  const [status, stdout, stderr] = await Promise.all([
    child.exited,
    new Response(child.stdout).text(),
    new Response(child.stderr).text(),
  ]);
  if (status === 0) {
    return { code: stdout, kind: "formatted" };
  }
  if (child.signalCode !== null) {
    return {
      kind: "failed",
      reason: `${command.name} was terminated by ${child.signalCode}`,
    };
  }
  const detail = stderr.length > 0 ? stderr : `exit status ${status}`;
  return { kind: "failed", reason: `${command.name} failed: ${detail}` };
}

const formatters = [
  {
    formatter: { name: "oxfmt", transform: formatTypeScript },
    pattern: /\.(?:ts|tsx|mts|cts)$/u,
  },
  {
    formatter: { name: "prettier", transform: formatWithPrettierConfiguration },
    pattern: /\.(?:yml|yaml|md|json)$/u,
  },
  {
    formatter: {
      name: "fish_indent",
      transform: (
        source: string,
        context: FormatContext,
      ): Promise<Transformation> =>
        pipeThrough({ arguments: [], name: "fish_indent" }, source, context),
    },
    pattern: /^home\/\.config\/fish\/(?:.+\/)?[^/]+\.fish$/u,
  },
  {
    formatter: {
      name: "rustfmt",
      transform: (
        source: string,
        context: FormatContext,
      ): Promise<Transformation> =>
        pipeThrough(
          { arguments: ["--emit", "stdout"], name: "rustfmt" },
          source,
          context,
        ),
    },
    pattern: /\.rs$/u,
  },
] as const satisfies readonly Readonly<{
  formatter: Formatter;
  pattern: RegExp;
}>[];

function formatterFor(relativePath: string): Formatter | undefined {
  for (const { formatter, pattern } of formatters) {
    if (pattern.test(relativePath)) {
      return formatter;
    }
  }
  return undefined;
}

export { formatterFor };
export type { Environment, FormatContext, Formatter, Transformation };
