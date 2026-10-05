import { dirname, join, relative, sep } from "node:path";
import { ensureDirectories, metadata } from "./agent-reach-files.ts";
import {
  link,
  mkdtemp,
  readFile,
  realpath,
  rm,
  writeFile,
} from "node:fs/promises";
import { z } from "zod";

const copySchema = z
  .object({
    home: z.string().min(1),
    source: z.string().min(1),
    destination: z.string().min(1),
    executable: z.boolean().optional(),
  })
  .readonly();
type CopyOptions = z.infer<typeof copySchema>;
const regularMode = 0o600;
const executableMode = 0o700;

async function copyAgentReachFile(input: CopyOptions): Promise<void> {
  const options = copySchema.parse(input);
  const home = await realpath(options.home);
  const path = relative(home, options.destination);
  if (path === ".." || path.startsWith(`..${sep}`)) {
    throw new Error("Destination outside HOME");
  }
  const content = await readFile(options.source);
  await ensureDirectories(home, dirname(options.destination));
  const existing = await metadata(options.destination);
  if (existing !== null) {
    const existingContent =
      existing.isFile() && !existing.isSymbolicLink()
        ? await readFile(options.destination)
        : null;
    const matches =
      existing.isFile() &&
      !existing.isSymbolicLink() &&
      existing.nlink === 1 &&
      existingContent !== null &&
      existingContent.equals(content);
    if (!matches) {
      throw new Error(`divergent configuration: ${options.destination}`);
    }
    return;
  }
  const staging = await mkdtemp(
    join(dirname(options.destination), ".agent-reach-"),
  );
  try {
    const file = join(staging, "content");
    await writeFile(file, content, {
      mode: options.executable === true ? executableMode : regularMode,
    });
    await link(file, options.destination);
  } finally {
    await rm(staging, { recursive: true, force: true });
  }
}

export { copyAgentReachFile };
