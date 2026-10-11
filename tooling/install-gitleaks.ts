import { chmod, mkdir, mkdtemp, rename, rm, writeFile } from "node:fs/promises";
import { join, resolve } from "node:path";
import { createHash } from "node:crypto";
import release from "./gitleaks-release.json";
import { z } from "zod";

const downloadTimeoutMilliseconds = 60_000;
const executableMode = 0o755;
const releaseSchema = z.object({
  version: z.string().regex(/^\d+\.\d+\.\d+$/u),
  linuxX64Sha256: z.string().regex(/^[a-f0-9]{64}$/u),
});
const manifest = releaseSchema.parse(release);
const destination = resolve(import.meta.dir, "../.moon/cache/gitleaks");
const invocation = z.union([z.tuple([]), z.tuple([z.literal("--check")])]);
const argumentOffset = 2;

function installedVersion(binary: string): string {
  const result = Bun.spawnSync([binary, "version"]);
  if (result.exitCode !== 0) {
    throw new Error("Gitleaks installation: version probe failed.");
  }
  return result.stdout.toString().trim();
}

async function extractRelease(
  temporary: string,
  version: string,
): Promise<string> {
  const path = join(temporary, "release.tar.gz");
  const extraction = Bun.spawnSync([
    "tar",
    "-xzf",
    path,
    "-C",
    temporary,
    "gitleaks",
  ]);
  if (extraction.exitCode !== 0) {
    throw new Error("Gitleaks extraction failed.");
  }
  const extracted = join(temporary, "gitleaks");
  await chmod(extracted, executableMode);
  if (installedVersion(extracted) !== version) {
    throw new Error("Gitleaks release version mismatch.");
  }
  return extracted;
}

async function installRelease(
  directory: string,
  expected: Readonly<z.infer<typeof releaseSchema>>,
  download: () => Promise<Uint8Array>,
): Promise<void> {
  const binary = join(directory, "gitleaks");
  if (
    (await Bun.file(binary).exists()) &&
    installedVersion(binary) === expected.version
  ) {
    return;
  }
  await mkdir(directory, { recursive: true });
  const temporary = await mkdtemp(join(directory, ".install-"));
  try {
    const archive = await download();
    if (
      createHash("sha256").update(archive).digest("hex") !==
      expected.linuxX64Sha256
    ) {
      throw new Error("Gitleaks download integrity failed.");
    }
    await writeFile(join(temporary, "release.tar.gz"), archive);
    await rename(await extractRelease(temporary, expected.version), binary);
  } finally {
    await rm(temporary, { recursive: true, force: true });
  }
}

async function downloadRelease(): Promise<Uint8Array> {
  const url = `https://github.com/gitleaks/gitleaks/releases/download/v${manifest.version}/gitleaks_${manifest.version}_linux_x64.tar.gz`;
  const response = await fetch(url, {
    signal: AbortSignal.timeout(downloadTimeoutMilliseconds),
  });
  if (!response.ok) {
    throw new Error(`Gitleaks download failed (${response.status}).`);
  }
  return new Uint8Array(await response.arrayBuffer());
}

if (import.meta.main) {
  try {
    const args = invocation.parse(Bun.argv.slice(argumentOffset));
    if (process.platform !== "linux" || process.arch !== "x64") {
      throw new Error("Gitleaks release installation supports Linux x64 only.");
    }
    if (args.length > 0) {
      if (
        installedVersion(join(destination, "gitleaks")) !== manifest.version
      ) {
        throw new Error("Gitleaks needs preparation.");
      }
    } else {
      await installRelease(destination, manifest, downloadRelease);
    }
  } catch (error) {
    process.stderr.write(
      `${error instanceof Error && !(error instanceof z.ZodError) ? error.message : "Gitleaks installation failed."}\n`,
    );
    process.exitCode = 1;
  }
}

export { installRelease, installedVersion };
