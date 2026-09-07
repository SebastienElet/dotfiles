import { dirname, join } from "node:path";
import {
  lstatSync,
  mkdirSync,
  readFileSync,
  renameSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { z } from "zod";

const argumentsSchema = z.tuple([z.string().min(1), z.string().min(1)]);
const argumentOffset = 2;
const failure = 1;

function isCurrent(destination: string, expected: string): boolean {
  try {
    return (
      lstatSync(destination).isFile() &&
      readFileSync(destination, "utf8") === expected
    );
  } catch (error) {
    if (error instanceof Error && "code" in error && error.code === "ENOENT") {
      return false;
    }
    throw error;
  }
}

function assembleInstructions(source: string, destination: string): void {
  const expected =
    readFileSync(join(source, "AGENTS.md"), "utf8").replaceAll(
      /^@.*\n/gmu,
      "",
    ) +
    readFileSync(join(source, "SOUL.md"), "utf8") +
    readFileSync(join(source, "USER.md"), "utf8");
  if (isCurrent(destination, expected)) {
    return;
  }
  mkdirSync(dirname(destination), { recursive: true });
  const temporary = `${destination}.expected.${process.pid}`;
  try {
    writeFileSync(temporary, expected, { flag: "wx" });
    renameSync(temporary, destination);
  } finally {
    rmSync(temporary, { force: true });
  }
}

if (import.meta.main) {
  try {
    const [source, destination] = argumentsSchema.parse(
      process.argv.slice(argumentOffset),
    );
    assembleInstructions(source, destination);
  } catch (error) {
    process.stderr.write(
      `${error instanceof Error ? error.message : String(error)}\n`,
    );
    process.exitCode = failure;
  }
}
