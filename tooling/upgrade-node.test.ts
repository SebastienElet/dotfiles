import { afterEach, expect, test } from "bun:test";
import { chmod, readdir, rename, rm } from "node:fs/promises";
import {
  cleanupUpgradeFixtures,
  upgradeFixture,
} from "./upgrade-test-support.ts";
import { join } from "node:path";

afterEach(cleanupUpgradeFixtures);

const directoryMode = 0o755;
const readOnlyMode = 0o555;
const originalPackage = { volta: { node: "24.18.1" } };
const updatedPackage = { volta: { node: "24.19.0" } };

test("reports impossible Node staging and continues independent domains", async () => {
  const fixture = await upgradeFixture();
  await chmod(fixture.repository, readOnlyMode);
  try {
    const result = await fixture.run();
    expect(result.status).toBe(1);
    expect(result.output).toContain("[failed] Node.js pin staging");
    expect(result.output).toContain("[not-attempted] Node.js installation");
    expect(result.calls).not.toContain("volta pin");
    expect(result.calls).toContain("npm update");
  } finally {
    await chmod(fixture.repository, directoryMode);
  }
});

test("reports missing project metadata and removes staging", async () => {
  const fixture = await upgradeFixture();
  await rm(join(fixture.repository, "package.json"));
  const result = await fixture.run();
  expect(result.status).toBe(1);
  expect(result.output).toContain("[failed] Node.js project metadata staging");
  expect(result.calls).not.toContain("volta pin");
  const entries = await readdir(fixture.repository);
  expect(entries.filter((entry) => entry.startsWith(".node-pin."))).toEqual([]);
  expect(result.calls).toContain("npm update");
});

test("invalid Node pin preserves project metadata without attempting installation", async () => {
  const fixture = await upgradeFixture();
  await Bun.write(
    join(fixture.home, "pinned-package.json"),
    '{"volta":{"node":"lts"}}',
  );
  const result = await fixture.run();
  expect(result.status).toBe(1);
  expect(result.output).toContain("[failed] Node.js pin validation");
  expect(result.output).toContain("[not-attempted] Node.js installation");
  expect(result.calls).not.toContain("volta install");
  expect(
    await Bun.file(join(fixture.repository, "package.json")).json(),
  ).toEqual(originalPackage);
});

test("failed installation preserves metadata and can be replayed", async () => {
  const fixture = await upgradeFixture();
  const failed = await fixture.run({ UPGRADE_FAILURE: "volta install" });
  expect(failed.status).toBe(1);
  expect(
    await Bun.file(join(fixture.repository, "package.json")).json(),
  ).toEqual(originalPackage);
  expect(await Bun.file(join(fixture.home, "installed-node")).exists()).toBe(
    false,
  );
  expect(failed.output).toContain("[not-attempted] Node.js pin publication");
  const recovered = await fixture.run();
  expect(recovered.status).toBe(0);
  expect(
    await Bun.file(join(fixture.repository, "package.json")).json(),
  ).toEqual(updatedPackage);
  expect(recovered.calls).not.toContain(" commit");
});

test("failed publication exposes installed Node and replays after the collision is removed", async () => {
  const fixture = await upgradeFixture();
  const failed = await fixture.run({ UPGRADE_NODE_EFFECT: "publication" });
  expect(failed.status).toBe(1);
  expect(failed.output).toContain("[succeeded] Node.js installation");
  expect(failed.output).toContain("[failed] Node.js pin publication");
  expect(await Bun.file(join(fixture.home, "installed-node")).text()).toBe(
    "install node@24.19.0",
  );
  expect(
    await Bun.file(join(fixture.home, "previous-package.json")).json(),
  ).toEqual(originalPackage);
  await rm(join(fixture.repository, "package.json"), { recursive: true });
  await rename(
    join(fixture.home, "previous-package.json"),
    join(fixture.repository, "package.json"),
  );
  const recovered = await fixture.run();
  expect(recovered.status).toBe(0);
  expect(
    await Bun.file(join(fixture.repository, "package.json")).json(),
  ).toEqual(updatedPackage);
  expect(recovered.calls).not.toContain(" commit");
});

test("cleanup failure contributes to the result after successful publication", async () => {
  const fixture = await upgradeFixture();
  try {
    const failed = await fixture.run({ UPGRADE_NODE_EFFECT: "cleanup" });
    expect(failed.status).toBe(1);
    expect(failed.output).toContain("[succeeded] Node.js pin publication");
    expect(failed.output).toContain("[failed] Node.js staging cleanup");
    expect(
      await Bun.file(join(fixture.repository, "package.json")).json(),
    ).toEqual(updatedPackage);
    expect(failed.calls).toContain("npm update");
  } finally {
    const entries = await readdir(fixture.repository);
    for (const stage of entries.filter((entry) =>
      entry.startsWith(".node-pin."),
    )) {
      await chmod(join(fixture.repository, stage, "blocked"), directoryMode);
      await rm(join(fixture.repository, stage), { recursive: true });
    }
  }
  const recovered = await fixture.run();
  expect(recovered.status).toBe(0);
  expect(recovered.calls).not.toContain(" commit");
});
