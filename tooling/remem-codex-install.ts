import { mcpServerArguments, run } from "./remem-process.ts";
import { readFileSync, renameSync, writeFileSync } from "node:fs";
import { isDeepStrictEqual } from "node:util";
import { join } from "node:path";
import { z } from "zod";

const argumentsSchema = z.tuple([z.string().min(1), z.string().min(1)]);
const toolsSchema = z.looseObject({
  mcp_servers: z.looseObject({
    remem: z.looseObject({ tools: z.record(z.string(), z.unknown()) }),
  }),
});
const argumentOffset = 2;
const rememToolsHeader =
  /^\s*\[\[?mcp_servers\.remem\.tools(?:\.[^\]]*)?\]\]?\s*$/u;
const anyHeader = /^\s*\[/u;

function withoutRememTools(configuration: string): string {
  let inRememTools = false;
  return configuration
    .split("\n")
    .filter((line) => {
      if (rememToolsHeader.test(line)) {
        inRememTools = true;
      } else if (anyHeader.test(line)) {
        inRememTools = false;
      }
      return !inRememTools;
    })
    .join("\n");
}

function parsedTools(configuration: string): Record<string, unknown> {
  return toolsSchema.parse(Bun.TOML.parse(configuration)).mcp_servers.remem
    .tools;
}

function install(workspaceRoot: string, home: string): void {
  const codex = join(home, ".volta", "bin", "codex");
  const toolsSource = readFileSync(
    join(workspaceRoot, "home", ".codex", "remem-mcp-tools.toml"),
    "utf8",
  );
  run([codex, "features", "disable", "memories"], { discardStdout: true });
  run([codex, "mcp", "add", "remem", ...mcpServerArguments], {
    discardStdout: true,
  });
  const path = join(home, ".codex", "config.toml");
  const updated = `${withoutRememTools(readFileSync(path, "utf8")).trimEnd()}\n\n${toolsSource}`;
  if (
    !isDeepStrictEqual(
      parsedTools(updated),
      parsedTools(`[mcp_servers.remem]\n\n${toolsSource}`),
    )
  ) {
    throw new Error(
      `${path} would not carry the tracked remem tool permissions`,
    );
  }
  const temporary = `${path}.${process.pid}.tmp`;
  writeFileSync(temporary, updated);
  renameSync(temporary, path);
  run([codex, "mcp", "get", "remem"], { discardStdout: true });
}

if (import.meta.main) {
  try {
    const [workspaceRoot, home] = argumentsSchema.parse(
      process.argv.slice(argumentOffset),
    );
    install(workspaceRoot, home);
  } catch (error) {
    process.stderr.write(
      `Error: ${error instanceof Error ? error.message : String(error)}\n`,
    );
    process.exitCode = 1;
  }
}
