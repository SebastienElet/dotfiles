import { isAbsolute, join } from "node:path";
import { lstatSync, mkdtempSync, writeFileSync } from "node:fs";
import { Buffer } from "node:buffer";
import { z } from "zod";

const initialArgumentLimit = 768;

function initialPrompt(directory: string, prompt: string): string {
  z.string().refine(isAbsolute).parse(directory);
  z.string().min(1).parse(prompt);
  promptReference(join(directory, "prompt-000000", "request.json"));
  if (!lstatSync(directory).isDirectory()) {
    throw new Error("Initial prompt directory must be a real directory");
  }
  const requestDirectory = mkdtempSync(join(directory, "prompt-"));
  const path = join(requestDirectory, "request.json");
  writeFileSync(path, JSON.stringify(prompt), { flag: "wx", mode: 0o600 });
  return promptReference(path);
}

function promptReference(path: string): string {
  const data = Buffer.from(JSON.stringify(path), "utf8").toString("base64");
  const instruction = `Read the file whose absolute path is encoded as base64 UTF-8 JSON below; parse that file as a JSON string and follow the complete user request: ${data}`;
  if (Buffer.byteLength(instruction, "utf8") > initialArgumentLimit) {
    throw new Error("Initial prompt reference exceeds native startup limit");
  }
  return instruction;
}

if (import.meta.main) {
  try {
    const directory = z.string().parse(process.argv[2]);
    process.stdout.write(initialPrompt(directory, await Bun.stdin.text()));
  } catch (error) {
    process.stderr.write(
      `${error instanceof Error ? error.message : String(error)}\n`,
    );
    process.exitCode = 1;
  }
}

export { initialPrompt };
