import { dirname, join } from "node:path";
import { mkdir, writeFile } from "node:fs/promises";
import { gitOutput } from "./secret-scan-git.ts";
import { runScanner } from "./secret-scan-scanner.ts";

async function scanBlobs(
  root: string,
  directory: string,
  source: Readonly<{ paths: readonly string[]; ref: string }>,
): Promise<void> {
  const snapshot = join(directory, "snapshot");
  await mkdir(snapshot);
  for (const path of source.paths) {
    const destination = join(snapshot, path);
    await mkdir(dirname(destination), { recursive: true });
    await writeFile(
      destination,
      gitOutput(root, ["cat-file", "blob", `${source.ref}:${path}`]),
    );
  }
  await runScanner(snapshot, directory, ["dir", "."]);
}

export { scanBlobs };
