import { lstat, mkdtemp, rm, rmdir, writeFile } from "node:fs/promises";
import type { Stats } from "node:fs";
import { createHash } from "node:crypto";
import { join } from "node:path";
import { z } from "zod";

type Installation = Readonly<{
  directory: string;
  checksum: string;
  download: () => Promise<Uint8Array>;
  extract: (image: string, destination: string) => Promise<void>;
  verify: (application: string) => Promise<void>;
}>;

async function command(arguments_: readonly string[]): Promise<void> {
  const child = Bun.spawn([...arguments_], {
    stdout: "inherit",
    stderr: "inherit",
  });
  const status = await child.exited;
  if (status !== 0) {
    throw new Error(`${arguments_[0]} failed (${status})`);
  }
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

async function installLumen(installation: Installation): Promise<void> {
  const destination = join(installation.directory, "Lumen.app");
  const existing = await metadata(destination);
  if (existing !== null) {
    if (!existing.isDirectory() || existing.isSymbolicLink()) {
      throw new Error(`${destination} exists`);
    }
    await installation.verify(destination);
    return;
  }
  const content = await verifiedDownload(installation);
  const staging = await mkdtemp(join(installation.directory, ".lumen-"));
  try {
    const image = join(staging, "Lumen.dmg");
    const application = join(staging, "Lumen.app");
    await writeFile(image, content);
    await installation.extract(image, application);
    await installation.verify(application);
    await command(["mv", "-n", application, installation.directory]);
    if (await metadata(application)) {
      throw new Error(`${destination} appeared during installation`);
    }
  } finally {
    await rm(staging, { recursive: true, force: true });
  }
}

async function verify(application: string): Promise<void> {
  await command([
    "/usr/bin/codesign",
    "--verify",
    "--deep",
    "--strict",
    '-R=anchor apple generic and certificate leaf[subject.OU] = "448LBGWBYM" and identifier "com.sonpiaz.lumen"',
    application,
  ]);
}

async function extract(image: string, destination: string): Promise<void> {
  const mount = await mkdtemp("/private/tmp/lumen-mount-");
  try {
    await command([
      "/usr/bin/hdiutil",
      "attach",
      "-readonly",
      "-nobrowse",
      "-mountpoint",
      mount,
      image,
    ]);
  } catch (error) {
    await rmdir(mount);
    throw error;
  }
  try {
    await command(["/usr/bin/ditto", join(mount, "Lumen.app"), destination]);
  } finally {
    await command(["/usr/bin/hdiutil", "detach", mount]);
    await rmdir(mount);
  }
}

async function download(): Promise<Uint8Array> {
  const timeoutMilliseconds = 120_000;
  const response = await fetch(
    "https://github.com/sonpiaz/lumen/releases/download/v0.1.0/Lumen-0.1.0.dmg",
    {
      signal: AbortSignal.timeout(timeoutMilliseconds),
    },
  );
  if (!response.ok) {
    throw new Error(`Lumen download failed: HTTP ${response.status}`);
  }
  return new Uint8Array(await response.arrayBuffer());
}

async function main(): Promise<void> {
  if (process.platform !== "darwin" || process.arch !== "arm64") {
    throw new Error("The Lumen 0.1.0 release requires macOS on Apple Silicon");
  }
  const argumentOffset = 2;
  const [directory] = z
    .tuple([z.string().startsWith("/")])
    .parse(process.argv.slice(argumentOffset));
  await installLumen({
    directory,
    checksum:
      "f1aa0d00f2752d3623cbb174299b129e67dfd9a2f3fcdf565c903436ab16a65d",
    download,
    extract,
    verify,
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

async function verifiedDownload(
  installation: Installation,
): Promise<Uint8Array> {
  const content = await installation.download();
  if (
    createHash("sha256").update(content).digest("hex") !== installation.checksum
  ) {
    throw new Error("Lumen SHA-256 mismatch");
  }
  return content;
}

export { installLumen };
