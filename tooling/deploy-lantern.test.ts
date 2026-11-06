import { afterEach, expect, test } from "bun:test";
import {
  cleanupDeploymentFixtures,
  createDeploymentFixture,
  createHarnessDeploymentFixture,
  expectSuccess,
  fileIdentity,
  pathExists,
} from "./deployment-test-support.ts";
import { dirname, join } from "node:path";
import {
  mkdirSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import {
  prepareLanternFixture,
  runLantern,
} from "./deploy-lantern-test-support.ts";
import { runDeploymentMoon } from "./deployment-moon-runner.ts";

afterEach(cleanupDeploymentFixtures);

const shaLength = 40;

test("Moon deploys the full pinned Lantern prompt followed by preferences and replays without rewriting", () => {
  const fixture = prepareLanternFixture(
    createHarnessDeploymentFixture("lantern"),
  );
  expectSuccess(runDeploymentMoon(fixture, ["harness:lantern"]));
  expect(readFileSync(fixture.destination, "utf8")).toBe(
    "# Upstream\nKeep every upstream instruction.\n\n\n# Preferences\nUse dedicated worktrees.\n",
  );
  const before = fileIdentity(fixture.generated);
  expectSuccess(runDeploymentMoon(fixture, ["harness:lantern"]));
  expect(fileIdentity(fixture.generated)).toEqual(before);
});

test.each([
  "malformed",
  "uninstalled",
  "revision",
  "edited",
  "unreadable",
  "command",
] as const)(
  "refuses %s upstream evidence and preserves an existing generated prompt",
  (failure) => {
    const fixture = prepareLanternFixture(
      createDeploymentFixture("lantern-refusal"),
    );
    expectSuccess(runLantern(fixture));
    const before = fileIdentity(fixture.generated);
    if (failure === "malformed") {
      writeFileSync(fixture.registration, "invalid");
    }
    if (failure === "uninstalled") {
      writeFileSync(fixture.registration, '{"result":{"plugins":[]}}');
    }
    if (failure === "revision") {
      writeFileSync(
        join(fixture.repository, "harness/lantern-revision"),
        "0".repeat(shaLength),
      );
    }
    if (failure === "edited") {
      writeFileSync(join(fixture.plugin, "prompt.md"), "changed upstream\n");
    }
    if (failure === "unreadable") {
      rmSync(join(fixture.plugin, "prompt.md"));
    }
    if (failure === "command") {
      writeFileSync(
        join(fixture.bin, "herdr"),
        `#!${process.execPath}\nprocess.exit(1);\n`,
      );
    }
    expect(runLantern(fixture).exitCode).not.toBe(0);
    expect(fileIdentity(fixture.generated)).toEqual(before);
  },
);

test.each(["file", "link", "directory"] as const)(
  "preserves a conflicting %s destination",
  (kind) => {
    const fixture = prepareLanternFixture(
      createDeploymentFixture("lantern-collision"),
    );
    expectSuccess(runLantern(fixture));
    const generatedBefore = fileIdentity(fixture.generated);
    rmSync(fixture.destination);
    writeFileSync(
      join(fixture.repository, "harness/lantern.md"),
      "New preferences\n",
    );
    const other = join(fixture.root, "personal.md");
    writeFileSync(other, "personal configuration\n");
    if (kind === "file") {
      writeFileSync(fixture.destination, "personal configuration\n");
    }
    if (kind === "link") {
      symlinkSync(other, fixture.destination);
    }
    if (kind === "directory") {
      mkdirSync(fixture.destination);
    }
    expect(runLantern(fixture).exitCode).not.toBe(0);
    expect(fileIdentity(fixture.generated)).toEqual(generatedBefore);
    expect(readFileSync(other, "utf8")).toBe("personal configuration\n");
    if (kind !== "directory") {
      expect(readFileSync(fixture.destination, "utf8")).toBe(
        "personal configuration\n",
      );
    }
  },
);

test("refuses redirected generated paths without writing outside the repository", () => {
  const fixture = prepareLanternFixture(
    createDeploymentFixture("lantern-path"),
  );
  const outside = join(fixture.root, "outside");
  mkdirSync(outside);
  symlinkSync(outside, join(fixture.repository, "harness/generated"));
  expect(runLantern(fixture).exitCode).not.toBe(0);
  expect(pathExists(join(outside, "lantern"))).toBe(false);
  expect(pathExists(fixture.destination)).toBe(false);
});

test("regenerates the composition on preference changes without touching private notes", () => {
  const fixture = prepareLanternFixture(
    createDeploymentFixture("lantern-update"),
  );
  const notes = join(fixture.home, ".local/state/lantern/preferences.md");
  mkdirSync(dirname(notes), { recursive: true });
  writeFileSync(notes, "private decision\n");
  expectSuccess(runLantern(fixture));
  writeFileSync(
    join(fixture.repository, "harness/lantern.md"),
    "# Updated preferences\n",
  );
  expectSuccess(runLantern(fixture));
  expect(readFileSync(fixture.destination, "utf8")).toBe(
    "# Upstream\nKeep every upstream instruction.\n\n\n# Updated preferences\n",
  );
  expect(readFileSync(notes, "utf8")).toBe("private decision\n");
});
