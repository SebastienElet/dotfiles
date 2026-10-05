import { deployLink } from "./deploy-link.ts";
import { join } from "node:path";
import { z } from "zod";

const argumentsSchema = z.tuple([]);
const environmentSchema = z.object({ HOME: z.string().min(1) });
const argumentOffset = 2;

if (import.meta.main) {
  try {
    argumentsSchema.parse(process.argv.slice(argumentOffset));
    const { HOME: home } = environmentSchema.parse(process.env);
    deployLink(
      join(import.meta.dir, "scrapling-mcp"),
      join(home, ".local", "bin", "scrapling_mcp"),
    );
  } catch (error) {
    process.stderr.write(
      `Error: ${error instanceof Error ? error.message : String(error)}\n`,
    );
    process.exitCode = 1;
  }
}
