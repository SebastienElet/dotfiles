import {
  createCheckoutFixture,
  readInvocations,
  runEntryPoint,
} from "./colgrep-search-test-support.ts";
import { expect, test } from "bun:test";

test("searches only after proving the active checkout index", () => {
  const fixture = createCheckoutFixture();
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
