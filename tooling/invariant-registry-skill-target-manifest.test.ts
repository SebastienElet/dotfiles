import { afterEach, expect, test } from "bun:test";
import {
  cleanupFixtures,
  conditionalRegistryText,
  initializeFixture,
  makeExecutionMarker,
} from "./invariant-registry-skill-target-repository-test-support.ts";
import { createHash } from "node:crypto";
import { inspectCanonicalMakefileDeployments } from "./invariant-registry-skill-target-deployment-manifest.ts";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { validateInvariantRegistryText } from "./invariant-registry-repository-validator.ts";

const deploymentDiagnostic =
  "Declared user-skill consumer has no matching user deployment";
const manifestDiagnostic = "Conditional skill deployment manifest is invalid";
const consumers = ["claude", "codex", "cursor"] as const;
const cursorTarget = "~/.cursor/skills/enforcement-code";
const cursorRouteHeader = `${cursorTarget}: \${DOTFILES_PATH}/harness/skills/enforcement-code FORCE | ~/.cursor/skills`;

const canonicalSnapshot = async (): Promise<
  Readonly<{ lines: readonly string[]; sha256: string }>
> => {
  const bytes = await readFile(resolve(import.meta.dir, "../Makefile"));
  return {
    lines: new TextDecoder("utf-8", { fatal: true }).decode(bytes).split("\n"),
    sha256: createHash("sha256").update(bytes).digest("hex"),
  };
};

const validationError = (root: string): Error | undefined => {
  try {
    validateInvariantRegistryText(conditionalRegistryText(), root);
    return undefined;
  } catch (error) {
    return error instanceof Error ? error : new Error(String(error));
  }
};

afterEach(cleanupFixtures);

test("accepts every exact route and aggregate in the canonical template", async () => {
  expect(
    inspectCanonicalMakefileDeployments(
      await canonicalSnapshot(),
      "enforcement-code",
      consumers,
    ),
  ).toEqual(consumers);
});

test("rejects removed aggregate membership independently of the digest", async () => {
  const snapshot = await canonicalSnapshot();
  const lines = snapshot.lines.map((line) =>
    line.startsWith("cursor: ") ? line.replace(` ${cursorTarget}`, "") : line,
  );
  expect(
    inspectCanonicalMakefileDeployments(
      { lines, sha256: snapshot.sha256 },
      "enforcement-code",
      consumers,
    ),
  ).toBeUndefined();
});

test("rejects a wrong route independently of the digest", async () => {
  const snapshot = await canonicalSnapshot();
  const lines = snapshot.lines.map((line) =>
    line === cursorRouteHeader
      ? line.replace("harness/skills/enforcement-code", "harness/skills")
      : line,
  );
  expect(
    inspectCanonicalMakefileDeployments(
      { lines, sha256: snapshot.sha256 },
      "enforcement-code",
      consumers,
    ),
  ).toBeUndefined();
});

test("accepts the byte-exact canonical Makefile copy", async () => {
  const root = await initializeFixture();
  expect(validationError(root)).toBeUndefined();
  expect(await Bun.file(makeExecutionMarker(root)).exists()).toBe(false);
});

test.each([
  "altered-byte",
  "fake-marker",
  "inactive-route",
  "malicious-target-override",
  "missing-aggregate",
  "missing-route",
  "wrong-route",
] as const)(
  "rejects the %s Makefile mutant without execution",
  async (mutation) => {
    const root = await initializeFixture({ makefileMutation: mutation });
    const error = validationError(root);
    expect(await Bun.file(makeExecutionMarker(root)).exists()).toBe(false);
    expect(error?.message ?? "").toContain(deploymentDiagnostic);
  },
);

test("rejects a missing Arnes consumer mapping", async () => {
  const root = await initializeFixture({
    installedFor: ["claude", "codex"],
  });
  expect(validationError(root)?.message ?? "").toContain(deploymentDiagnostic);
  expect(await Bun.file(makeExecutionMarker(root)).exists()).toBe(false);
});

test("rejects duplicate Arnes installations before inspecting routes", async () => {
  const root = await initializeFixture({
    installedFor: ["claude", "codex", "cursor", "cursor"],
    makefileMutation: "malicious-target-override",
  });
  expect(validationError(root)?.message ?? "").toContain(manifestDiagnostic);
  expect(await Bun.file(makeExecutionMarker(root)).exists()).toBe(false);
});
