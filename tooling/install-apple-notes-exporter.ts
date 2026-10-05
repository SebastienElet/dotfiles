import { lstat, mkdtemp, rm } from "node:fs/promises";
import type { Stats } from "node:fs";
import { join } from "node:path";
import { z } from "zod";

const executableBits = 0o111;
const argumentOffset = 2;
const applicationName = "Apple Notes Exporter.app";
const bundleSchema = z.object({
  CFBundleIdentifier: z.literal("com.zaremski.AppleNotesExporter"),
  CFBundleExecutable: z.literal("Apple Notes Exporter"),
});
const pinnedBundleSchema = bundleSchema.extend({
  CFBundleShortVersionString: z.literal("2.0"),
  CFBundleVersion: z.literal("2"),
});

type Installation = Readonly<{
  directory: string;
  download: (archive: string) => Promise<void>;
  extract: (archive: string, staging: string) => Promise<void>;
  readBundleInfo: (path: string) => Promise<unknown>;
}>;

async function command(arguments_: readonly string[]): Promise<string> {
  const child = Bun.spawn([...arguments_], {
    stdout: "pipe",
    stderr: "inherit",
  });
  const [output, exitCode] = await Promise.all([
    new Response(child.stdout).text(),
    child.exited,
  ]);
  if (exitCode !== 0) {
    throw new Error(`${arguments_[0]} failed (${exitCode})`);
  }
  return output;
}

async function metadata(path: string): Promise<Stats | null> {
  try {
    return await lstat(path);
  } catch (error) {
    if (z.object({ code: z.literal("ENOENT") }).safeParse(error).success) {
      return null;
    }
    throw error;
  }
}

async function verifyBundle(
  application: string,
  readBundleInfo: Installation["readBundleInfo"],
  validateInfo: (info: unknown) => unknown,
): Promise<void> {
  for (const path of [
    application,
    ...["Contents", "Contents/MacOS", "Contents/SharedSupport"].map((part) =>
      join(application, part),
    ),
  ]) {
    const entry = await metadata(path);
    if (entry === null || !entry.isDirectory() || entry.isSymbolicLink()) {
      throw new Error(`Missing or divergent application directory: ${path}`);
    }
  }
  const infoPath = join(application, "Contents/Info.plist");
  const info = await metadata(infoPath);
  if (info === null || !info.isFile() || info.isSymbolicLink()) {
    throw new Error(`Missing or divergent bundle metadata: ${infoPath}`);
  }
  validateInfo(await readBundleInfo(infoPath));
  for (const part of [
    "MacOS/Apple Notes Exporter",
    "SharedSupport/notes-export-mcp",
  ]) {
    const path = join(application, "Contents", part);
    const entry = await metadata(path);
    if (
      entry === null ||
      !entry.isFile() ||
      entry.isSymbolicLink() ||
      (entry.mode & executableBits) === 0
    ) {
      throw new Error(
        `Missing or non-executable application/MCP binary: ${path}`,
      );
    }
  }
}

async function installAppleNotesExporter(
  installation: Installation,
): Promise<void> {
  const directory = z.string().startsWith("/").parse(installation.directory);
  const destination = join(directory, applicationName);
  const existing = await metadata(destination);
  if (existing !== null) {
    if (!existing.isDirectory() || existing.isSymbolicLink()) {
      throw new Error(`${destination} exists`);
    }
    await verifyBundle(destination, installation.readBundleInfo, (info) =>
      bundleSchema.parse(info),
    );
    return;
  }
  const staging = await mkdtemp(join(directory, ".apple-notes-exporter-"));
  try {
    const archive = join(staging, "AppleNotesExporter.zip");
    const application = join(staging, applicationName);
    await installation.download(archive);
    await installation.extract(archive, staging);
    await verifyBundle(application, installation.readBundleInfo, (info) =>
      pinnedBundleSchema.parse(info),
    );
    await command(["/bin/mv", "-n", application, directory]);
    if (await metadata(application)) {
      throw new Error(`${destination} appeared during installation`);
    }
  } finally {
    await rm(staging, { recursive: true, force: true });
  }
}

async function main(): Promise<void> {
  if (process.platform !== "darwin") {
    throw new Error("Apple Notes Exporter requires macOS");
  }
  const [directory] = z
    .tuple([z.string().startsWith("/")])
    .parse(process.argv.slice(argumentOffset));
  await installAppleNotesExporter({
    directory,
    download: async (archive) => {
      await command([
        "/usr/bin/curl",
        "--fail",
        "--location",
        "--proto",
        "=https",
        "--proto-redir",
        "=https",
        "--connect-timeout",
        "15",
        "--max-time",
        "120",
        "--max-filesize",
        "33554432",
        "https://github.com/kzaremski/apple-notes-exporter/releases/download/v2.0-2/AppleNotesExporter_v2.0-2.zip",
        "--output",
        archive,
      ]);
    },
    extract: async (archive, staging) => {
      await command([
        "/usr/bin/unzip",
        "-q",
        archive,
        `${applicationName}/*`,
        "-d",
        staging,
      ]);
    },
    readBundleInfo: async (path): Promise<unknown> =>
      JSON.parse(
        await command(["/usr/bin/plutil", "-convert", "json", "-o", "-", path]),
      ),
  });
}

if (import.meta.main) {
  try {
    await main();
  } catch (error) {
    process.stderr.write(
      `${error instanceof Error ? error.message : String(error)}\n`,
    );
    process.exitCode = 1;
  }
}

export { installAppleNotesExporter };
