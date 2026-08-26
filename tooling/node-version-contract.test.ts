import { afterEach, describe, expect, test } from "bun:test";
import { chmod, mkdtemp, rm } from "node:fs/promises";
import {
  nodeInstallSpec,
  readNodeVersion,
  verifyNodeRuntime,
} from "./node-version-contract";
import { fileURLToPath } from "node:url";
import { join } from "node:path";
import { tmpdir } from "node:os";

const executableMode = 0o755;
const temporaryDirectories: string[] = [];

async function createTemporaryDirectory(): Promise<string> {
  const directory = await mkdtemp(join(tmpdir(), "node-version-contract-"));
  temporaryDirectories.push(directory);
  return directory;
}

async function writePackageJson(value: unknown): Promise<string> {
  const directory = await createTemporaryDirectory();
  const packageJsonPath = join(directory, "package.json");
  await Bun.write(packageJsonPath, JSON.stringify(value));
  return packageJsonPath;
}

afterEach(async () => {
  await Promise.all(
    temporaryDirectories
      .splice(0)
      .map((directory) => rm(directory, { recursive: true })),
  );
});

describe("Node project pin", () => {
  test("reads an exact version and builds the Volta install argument", async () => {
    const packageJsonPath = await writePackageJson({
      volta: { node: "24.19.0" },
    });
    const nodeVersion = await readNodeVersion(packageJsonPath);

    expect(nodeVersion).toBe("24.19.0");
    expect(nodeInstallSpec(nodeVersion)).toBe("node@24.19.0");
  });

  test.each([
    ["absent", {}],
    ["moving LTS alias", { volta: { node: "lts" } }],
    ["partial version", { volta: { node: "24" } }],
    ["invalid exact version", { volta: { node: "024.19.0" } }],
  ])("rejects a pin that is %s", async (_name, packageJson: unknown) => {
    const packageJsonPath = await writePackageJson(packageJson);

    expect(readNodeVersion(packageJsonPath)).rejects.toThrow(
      "Cannot read an exact Node pin",
    );
  });

  test("rejects malformed JSON", async () => {
    const directory = await createTemporaryDirectory();
    const packageJsonPath = join(directory, "package.json");
    await Bun.write(packageJsonPath, "{");

    expect(readNodeVersion(packageJsonPath)).rejects.toThrow(
      "Cannot read an exact Node pin",
    );
  });

  test("rejects an unreadable package path", async () => {
    const directory = await createTemporaryDirectory();

    expect(readNodeVersion(join(directory, "missing.json"))).rejects.toThrow(
      "Cannot read an exact Node pin",
    );
  });
});

describe("Node runtime", () => {
  test("matches the repository pin through the active Volta shim", async () => {
    const nodeVersion = await readNodeVersion(
      fileURLToPath(new URL("../package.json", import.meta.url)),
    );

    expect(verifyNodeRuntime(nodeVersion)).resolves.toBeUndefined();
  });

  test("rejects a runtime version that diverges from the pin", async () => {
    const directory = await createTemporaryDirectory();
    const fakeNode = join(directory, "node");
    await Bun.write(fakeNode, "#!/bin/sh\nprintf '%s\\n' v24.18.1\n");
    await chmod(fakeNode, executableMode);
    expect(verifyNodeRuntime("24.19.0", fakeNode)).rejects.toThrow(
      "Node runtime 24.18.1 does not match project pin 24.19.0",
    );
  });

  test("rejects an empty runtime version", async () => {
    const directory = await createTemporaryDirectory();
    const fakeNode = join(directory, "node");
    await Bun.write(fakeNode, "#!/bin/sh\nexit 0\n");
    await chmod(fakeNode, executableMode);
    expect(verifyNodeRuntime("24.19.0", fakeNode)).rejects.toThrow();
  });

  test("keeps a failed runtime probe visible", async () => {
    const directory = await createTemporaryDirectory();
    const fakeNode = join(directory, "node");
    await Bun.write(
      fakeNode,
      "#!/bin/sh\nprintf '%s\\n' unavailable >&2\nexit 23\n",
    );
    await chmod(fakeNode, executableMode);
    expect(verifyNodeRuntime("24.19.0", fakeNode)).rejects.toThrow(
      "node --version failed with status 23: unavailable",
    );
  });
});
