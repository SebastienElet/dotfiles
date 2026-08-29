import { expect, test } from "bun:test";
import {
  createLinkedWorktreeFixture,
  readInvocations,
  runEntryPoint,
} from "./colgrep-worktree-test-support.ts";

test("searches only after proving the active linked-worktree index", () => {
  const fixture = createLinkedWorktreeFixture();
  const result = runEntryPoint(
    fixture.linkedRoot,
    "authentication boundary",
    fixture.environment,
  );

  expect(result.exitCode).toBe(0);
  expect(JSON.parse(result.stdout)).toEqual([fixture.activeResult]);
  expect(readInvocations(fixture)).toEqual([
    ["init", "-y", fixture.linkedRoot],
    ["status", fixture.linkedRoot],
    [
      "search",
      "--json",
      "--no-update",
      "authentication boundary",
      fixture.linkedRoot,
    ],
  ]);
});
