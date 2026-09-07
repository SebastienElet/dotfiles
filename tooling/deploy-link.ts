import {
  lstatSync,
  mkdirSync,
  readlinkSync,
  statSync,
  symlinkSync,
} from "node:fs";
import { dirname } from "node:path";
import { z } from "zod";

const argumentsSchema = z.tuple([z.string().min(1), z.string().min(1)]);
const argumentOffset = 2;

function deployLink(source: string, destination: string): void {
  statSync(source);
  const metadata = lstatSync(destination, { throwIfNoEntry: false });
  if (
    metadata?.isSymbolicLink() === true &&
    readlinkSync(destination) === source
  ) {
    return;
  }
  if (metadata !== undefined) {
    throw new Error(
      `${destination} exists and is not the expected symbolic link`,
    );
  }
  mkdirSync(dirname(destination), { recursive: true });
  symlinkSync(source, destination);
}

if (import.meta.main) {
  try {
    const [source, destination] = argumentsSchema.parse(
      process.argv.slice(argumentOffset),
    );
    deployLink(source, destination);
  } catch (error) {
    process.stderr.write(
      `Error: ${error instanceof Error ? error.message : String(error)}\n`,
    );
    process.exitCode = 1;
  }
}

export { deployLink };
