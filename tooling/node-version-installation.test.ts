import { afterEach, describe, expect, test } from "bun:test";
import {
  cleanupNodeVersionFixtures,
  runMakeNode,
  runUpgrade,
} from "./node-version-installation-test-support";

afterEach(cleanupNodeVersionFixtures);

describe("Makefile Node installation", () => {
  test("passes the exact project pin to Volta", async () => {
    const result = await runMakeNode({ volta: { node: "24.18.1" } });

    expect(result.status).toBe(0);
    expect(result.calls).toBe("install node@24.18.1\n");
  });

  test("fails closed when the project pin is absent", async () => {
    const result = await runMakeNode({});

    expect(result.status).not.toBe(0);
    expect(result.calls).toBe("");
    expect(result.output).toContain("Cannot read an exact Node pin");
  });
});

describe("Node upgrade", () => {
  test("pins the current LTS before installing the resolved exact version", async () => {
    const result = await runUpgrade({
      pinnedPackage: { volta: { node: "24.19.0" } },
    });

    expect(result.status).toBe(0);
    expect(result.calls).toBe("pin node@lts\ninstall node@24.19.0\n");
  });

  test("fails closed when Volta produces an invalid pin", async () => {
    const result = await runUpgrade({
      pinnedPackage: { volta: { node: "lts" } },
    });

    expect(result.status).toBe(1);
    expect(result.calls).toBe("pin node@lts\n");
    expect(result.output).toContain("invalid Node.js project pin");
  });

  test("fails closed when the pin validator cannot run", async () => {
    const result = await runUpgrade({
      includeBun: false,
      pinnedPackage: { volta: { node: "24.19.0" } },
    });

    expect(result.status).toBe(1);
    expect(result.calls).toBe("");
    expect(result.output).toContain(
      "bun not found, unable to validate the Node.js pin",
    );
  });

  test("fails closed when the pin validator dependencies are missing", async () => {
    const result = await runUpgrade({
      includeDependencies: false,
      pinnedPackage: { volta: { node: "24.19.0" } },
    });

    expect(result.status).toBe(1);
    expect(result.calls).toBe("");
    expect(result.output).toContain(
      "Node.js pin validator dependencies missing",
    );
  });
});

describe("Node upgrade command failures", () => {
  test("fails closed when Volta cannot update the project pin", async () => {
    const result = await runUpgrade({
      failVoltaCommand: "pin",
      pinnedPackage: { volta: { node: "24.19.0" } },
    });

    expect(result.status).toBe(1);
    expect(result.calls).toBe("pin node@lts\n");
    expect(result.output).toContain("unable to update the Node.js project pin");
  });

  test("fails closed when Volta cannot install the exact project pin", async () => {
    const result = await runUpgrade({
      failVoltaCommand: "install",
      pinnedPackage: { volta: { node: "24.19.0" } },
    });

    expect(result.status).toBe(1);
    expect(result.calls).toBe("pin node@lts\ninstall node@24.19.0\n");
    expect(result.output).toContain(
      "unable to install the pinned Node.js version",
    );
  });

  test("fails closed when Volta is missing", async () => {
    const result = await runUpgrade({
      includeVolta: false,
      pinnedPackage: { volta: { node: "24.19.0" } },
    });

    expect(result.status).toBe(1);
    expect(result.calls).toBe("");
    expect(result.output).toContain(
      "volta not found, unable to upgrade Node.js",
    );
  });
});
