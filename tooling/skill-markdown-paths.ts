import { checkCommand } from "./check-command.ts";
import { z } from "zod";

const pathsSchema = z.array(z.string().min(1).endsWith(".md")).min(1);

function skillMarkdownPaths(): readonly string[] {
  const result = checkCommand([
    "git",
    "ls-files",
    "-z",
    "--",
    "harness/skills/**/*.md",
    ".agents/skills/**/*.md",
  ]);
  const output = new TextDecoder("utf-8", { fatal: true }).decode(
    result.stdout,
  );
  if (!output.endsWith("\0")) {
    throw new Error("Empty or malformed Git skill Markdown selection");
  }
  return pathsSchema.parse(output.slice(0, -1).split("\0"));
}

function requiredFiles(patterns: readonly string[]): readonly string[] {
  return patterns.flatMap((pattern) => {
    const paths = [...new Bun.Glob(pattern).scanSync({ dot: true })];
    if (paths.length === 0) {
      throw new Error(`No files found for required selection: ${pattern}`);
    }
    return paths;
  });
}

export { requiredFiles, skillMarkdownPaths };
