import { dirname, join } from "node:path";
import {
  ensureDirectories,
  installedFiles,
  metadata,
  preserveExisting,
  regularText,
} from "./agent-reach-files.ts";
import {
  mkdir,
  mkdtemp,
  realpath,
  rename,
  rm,
  rmdir,
  writeFile,
} from "node:fs/promises";

const directoryMode = 0o700;
const fileMode = 0o600;
const cursorManifest = ".cursor-plugin/plugin.json";

async function publishProjection(
  home: string,
  destination: string,
  files: readonly (readonly [string, string])[],
): Promise<void> {
  const staging = await mkdtemp(
    join(dirname(dirname(destination)), ".agent-reach-"),
  );
  try {
    for (const [path, content] of files) {
      const target = join(staging, path);
      await ensureDirectories(home, dirname(target));
      await writeFile(target, content, { flag: "wx", mode: fileMode });
    }
    await mkdir(destination, { mode: directoryMode });
    try {
      await rename(staging, destination);
    } catch (error) {
      await rmdir(destination);
      throw error;
    }
  } finally {
    await rm(staging, { recursive: true, force: true });
  }
}

async function projectAgentReachCursor(
  homeArgument: string,
  marketplace: string,
): Promise<string> {
  const home = await realpath(homeArgument);
  const source = await realpath(join(marketplace, "plugins/agent-reach"));
  const files: (readonly [string, string])[] = [];
  for (const path of await installedFiles(source)) {
    files.push([path, await regularText(source, path)]);
  }
  files.push([
    cursorManifest,
    await regularText(source, ".claude-plugin/plugin.json"),
  ]);
  const destination = join(home, ".cursor/plugins/local/agent-reach");
  await ensureDirectories(home, dirname(destination));
  const existing = await metadata(destination);
  if (existing !== null) {
    if (!existing.isDirectory() || existing.isSymbolicLink()) {
      throw new Error(`divergent Cursor plugin: ${destination}`);
    }
    await preserveExisting(destination, files);
    return destination;
  }
  await publishProjection(home, destination, files);
  return destination;
}

export { projectAgentReachCursor };
