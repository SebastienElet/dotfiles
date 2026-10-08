import { afterEach, expect, test } from "bun:test";
import { mkdtempSync, rmSync } from "node:fs";
import type { IssueFixture } from "./dispatch-test-support.ts";
import { MemoryHerdr } from "./memory-herdr-test-support.ts";
import { dispatchWithReservation } from "./launch.ts";
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
