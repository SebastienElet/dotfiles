import { checkCommand, reportCheckFailure } from "./check-command.ts";
import { getFileInfo } from "prettier";
import { skillMarkdownPaths } from "./skill-markdown-paths.ts";
import { z } from "zod";

const argumentOffset = 2;

async function main(): Promise<void> {
  const argumentsToPrettier = z
    .array(z.string().min(1))
    .min(1)
    .parse(process.argv.slice(argumentOffset));
  const skills = skillMarkdownPaths();
  for (const path of skills) {
    const info = await getFileInfo(path, {
      ignorePath: [".gitignore", ".prettierignore"],
    });
    if (info.ignored) {
      throw new Error(`Prettier excludes indexed skill Markdown: ${path}`);
    }
  }
  const result = checkCommand([
    "bun",
    "run",
    "prettier",
    ...argumentsToPrettier,
    ...skills,
  ]);
  process.stdout.write(result.stdout);
  process.stderr.write(result.stderr);
}

try {
  await main();
} catch (error) {
  reportCheckFailure(error);
}
