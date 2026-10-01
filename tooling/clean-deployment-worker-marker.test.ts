import { afterEach, expect, test } from "bun:test";
import {
  cleanupDeploymentFixtures,
  createDeploymentFixture,
} from "./deployment-test-support.ts";
import {
  closeSync,
  mkdirSync,
  openSync,
  readFileSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { publishWorkerMarker } from "./clean-deployment-worker-marker.ts";

afterEach(cleanupDeploymentFixtures);

test("publishes a complete replacement without exposing truncated content to an existing reader", () => {
  const context = createDeploymentFixture("worker-marker");
  const path = join(context.root, "marker.json");
  const before = JSON.stringify({ stopped: false });
  writeFileSync(path, before);
  const reader = openSync(path, "r");
  try {
    publishWorkerMarker(path, { stopped: true });
    expect(readFileSync(reader, "utf8")).toBe(before);
    expect(JSON.parse(readFileSync(path, "utf8"))).toEqual({ stopped: true });
  } finally {
    closeSync(reader);
  }
});

test("preserves a destination that refuses publication", () => {
  const context = createDeploymentFixture("worker-marker-refused");
  const path = join(context.root, "marker");
  mkdirSync(path);
  writeFileSync(join(path, "preserved"), "keep");
  expect(() => {
    publishWorkerMarker(path, { stopped: true });
  }).toThrow();
  expect(readFileSync(join(path, "preserved"), "utf8")).toBe("keep");
});
