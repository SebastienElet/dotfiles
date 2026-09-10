import { copyFile, mkdtemp, rename, rm } from "node:fs/promises";
import { nodeInstallSpec, readNodeVersion } from "./node-version-contract.ts";
import type { UpgradeRunner } from "./upgrade-runner.ts";
import { join } from "node:path";

async function upgradeNode(
  runner: UpgradeRunner,
  repository: string,
): Promise<void> {
  if (Bun.which("volta") === null) {
    await runner.attempt("Node.js upgrade", () => {
      throw new Error("volta not found, unable to upgrade Node.js");
    });
    skipNodeInstallation(runner, "Volta unavailable");
    return;
  }
  const stage = await runner.attempt("Node.js pin staging", () =>
    mkdtemp(join(repository, ".node-pin.")),
  );
  if (stage.state !== "succeeded") {
    skipNodeInstallation(runner, "unable to stage the Node.js project pin");
    return;
  }
  try {
    await installStagedNode(runner, repository, stage.value);
  } finally {
    await runner.attempt("Node.js staging cleanup", () =>
      rm(stage.value, { recursive: true }),
    );
  }
}

async function installStagedNode(
  runner: UpgradeRunner,
  repository: string,
  stage: string,
): Promise<void> {
  const stagedPackage = join(stage, "package.json");
  const copied = await runner.attempt("Node.js project metadata staging", () =>
    copyFile(join(repository, "package.json"), stagedPackage),
  );
  if (copied.state !== "succeeded") {
    skipNodeInstallation(
      runner,
      "unable to stage the Node.js project metadata",
    );
    return;
  }
  const pin = await runner.command(
    "Node.js project pin",
    ["volta", "pin", "node@lts"],
    { cwd: stage, required: true },
  );
  if (pin.state !== "succeeded") {
    skipNodeInstallation(runner, "unable to update the Node.js project pin");
    return;
  }
  const spec = await runner.attempt("Node.js pin validation", async () =>
    nodeInstallSpec(await readNodeVersion(stagedPackage)),
  );
  if (spec.state !== "succeeded") {
    skipNodeInstallation(runner, "invalid Node.js project pin");
    return;
  }
  const installed = await runner.command(
    "Node.js installation",
    ["volta", "install", spec.value],
    { required: true },
  );
  if (installed.state !== "succeeded") {
    runner.skip(
      "Node.js pin publication",
      "unable to install the pinned Node.js version",
    );
    return;
  }
  await runner.attempt("Node.js pin publication", () =>
    rename(stagedPackage, join(repository, "package.json")),
  );
}

function skipNodeInstallation(runner: UpgradeRunner, reason: string): void {
  runner.skip("Node.js installation", reason);
  runner.skip("Node.js pin publication", reason);
}

export { upgradeNode };
