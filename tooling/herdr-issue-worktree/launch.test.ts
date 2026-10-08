import { afterEach, expect, test } from "bun:test";
import { mkdtempSync, readdirSync, rmSync } from "node:fs";
import type { IssueFixture } from "./dispatch-test-support.ts";
import { MemoryHerdr } from "./memory-herdr-test-support.ts";
import { dispatchWithReservation } from "./launch.ts";
import { inspectRepository } from "./repository.ts";
import { issueFixture } from "./dispatch-test-support.ts";
import { join } from "node:path";
import { tmpdir } from "node:os";

const fixtures: IssueFixture[] = [];
const directories: string[] = [];
const sourceAndPreparationPaneCount = 2;
afterEach(() => {
  for (const fixture of fixtures.splice(0)) {
    fixture.dispose();
  }
  for (const directory of directories.splice(0)) {
    rmSync(directory, { recursive: true, force: true });
  }
});

test("reuses working preparation through a retained uncertain-start reservation", async () => {
  const fixture = issueFixture();
  fixtures.push(fixture);
  const directory = mkdtempSync(join(tmpdir(), "herdr-issue-reservation-"));
  directories.push(directory);
  const herdr = new MemoryHerdr(fixture);
  herdr.startupUncertain = true;
  const failure = await dispatchWithReservation(
    fixture.selection,
    herdr,
    directory,
  ).catch((error: unknown) => error);
  if (!(failure instanceof Error)) {
    throw new Error("Expected an uncertain startup error");
  }
  expect(failure.message).toContain("uncertain");
  expect(
    await dispatchWithReservation(fixture.selection, herdr, directory),
  ).toMatchObject({ kind: "reused", paneId: "preparation:p1" });
  expect(herdr.initialPrompts).toHaveLength(1);
  expect(herdr.state.panes).toHaveLength(sourceAndPreparationPaneCount);
});

for (const gitArguments of [
  [
    "-c",
    "user.name=Fixture",
    "-c",
    "user.email=fixture@example.invalid",
    "commit",
    "-q",
    "--allow-empty",
    "-m",
    "advance",
  ],
  ["checkout", "-q", "-b", "new-source"],
  ["checkout", "-q", "--detach"],
]) {
  test("releases a definitely untouched preflight reservation before a refreshed click", async () => {
    const fixture = issueFixture();
    fixtures.push(fixture);
    const directory = mkdtempSync(join(tmpdir(), "herdr-issue-preflight-"));
    directories.push(directory);
    const herdr = new MemoryHerdr(fixture);
    const changed = Bun.spawnSync([
      "git",
      "-C",
      fixture.selection.repository.root,
      ...gitArguments,
    ]);
    expect(changed.exitCode).toBe(0);
    const rejected = await dispatchWithReservation(
      fixture.selection,
      herdr,
      directory,
    ).catch((error: unknown) => error);
    expect(rejected).toMatchObject({ kind: "rejected" });
    expect(herdr.state.panes).toHaveLength(1);
    expect(herdr.initialPrompts).toHaveLength(0);
    expect(readdirSync(directory)).toEqual([]);
    if (gitArguments.includes("--detach")) {
      expect(
        Bun.spawnSync([
          "git",
          "-C",
          fixture.selection.repository.root,
          "checkout",
          "-q",
          "main",
        ]).exitCode,
      ).toBe(0);
    }
    const refreshed = {
      ...fixture.selection,
      repository: inspectRepository(fixture.selection.repository.root),
    };
    expect(
      await dispatchWithReservation(refreshed, herdr, directory),
    ).toMatchObject({ kind: "started" });
    expect(herdr.initialPrompts).toHaveLength(1);
  });
}
