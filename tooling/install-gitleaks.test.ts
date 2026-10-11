import { copyFile, mkdir, readdir, stat, writeFile } from "node:fs/promises";
import { expect, test } from "bun:test";
import { installRelease, installedVersion } from "./install-gitleaks.ts";
import { createHash } from "node:crypto";
import { join } from "node:path";
import { repository } from "./secret-scan-test-support.ts";

const sha256Length = 64;

async function archive(root: string): Promise<Uint8Array> {
  const binary = Bun.which("gitleaks");
  if (binary === null) {
    throw new Error(
      "The real Gitleaks must be prepared for the installation test.",
    );
  }
  const source = join(root, "release");
  await mkdir(source);
  await copyFile(binary, join(source, "gitleaks"));
  const path = join(root, "release.tar.gz");
  const result = Bun.spawnSync(["tar", "-czf", path, "-C", source, "gitleaks"]);
  if (result.exitCode !== 0) {
    throw new Error("Cannot assemble release fixture.");
  }
  return new Uint8Array(await Bun.file(path).arrayBuffer());
}

test("verified installation executes the real binary and a second preparation stays offline", async () => {
  const root = await repository();
  const bytes = await archive(root);
  const version = installedVersion(Bun.which("gitleaks") ?? "gitleaks");
  process.stderr.write(
    `Qualified Gitleaks ${version} on ${process.platform} ${process.arch}\n`,
  );
  const expected = {
    version,
    linuxX64Sha256: createHash("sha256").update(bytes).digest("hex"),
  };
  const directory = join(root, "installed");
  await installRelease(directory, expected, () => Promise.resolve(bytes));
  const binary = join(directory, "gitleaks");
  expect(installedVersion(binary)).toBe(version);
  const before = await stat(binary);
  await installRelease(directory, expected, () =>
    Promise.reject(new Error("Network must remain unused")),
  );
  const after = await stat(binary);
  expect(after.mtimeMs).toBe(before.mtimeMs);
  expect(await readdir(directory)).toEqual(["gitleaks"]);
});

test("download and integrity failures preserve an existing installation", async () => {
  const root = await repository();
  const directory = join(root, "installed");
  await mkdir(directory);
  const binary = join(directory, "gitleaks");
  await writeFile(binary, "#!/bin/sh\nprintf '0.0.0\\n'\n", { mode: 0o755 });
  const expected = {
    version: "8.30.1",
    linuxX64Sha256: "0".repeat(sha256Length),
  };
  expect(
    installRelease(directory, expected, () =>
      Promise.reject(new Error("download unavailable")),
    ),
  ).rejects.toThrow("download unavailable");
  expect(
    installRelease(directory, expected, () =>
      Promise.resolve(new Uint8Array()),
    ),
  ).rejects.toThrow("integrity failed");
  expect(installedVersion(binary)).toBe("0.0.0");
  expect(await readdir(directory)).toEqual(["gitleaks"]);
});

test("invalid archives and unexpected binary versions fail without publishing a binary", async () => {
  const root = await repository();
  const invalid = new Uint8Array([1]);
  const expected = {
    version: "0.0.0",
    linuxX64Sha256: createHash("sha256").update(invalid).digest("hex"),
  };
  const directory = join(root, "installed");
  expect(
    installRelease(directory, expected, () => Promise.resolve(invalid)),
  ).rejects.toThrow("extraction failed");
  const bytes = await archive(root);
  expect(
    installRelease(
      directory,
      {
        ...expected,
        linuxX64Sha256: createHash("sha256").update(bytes).digest("hex"),
      },
      () => Promise.resolve(bytes),
    ),
  ).rejects.toThrow("version mismatch");
  expect(await readdir(directory)).toEqual([]);
});
