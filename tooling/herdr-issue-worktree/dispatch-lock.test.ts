import { afterEach, expect, test } from "bun:test";
import { existsSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { withDispatchLock } from "./dispatch-lock.ts";

const roots: string[] = [];

afterEach(() => {
  for (const root of roots.splice(0)) {
    rmSync(root, { force: true, recursive: true });
  }
});

test("refuses a competing dispatch until the first critical section finishes", async () => {
  const root = mkdtempSync(join(tmpdir(), "herdr-dispatch-lock-"));
  roots.push(root);
  const completion = Promise.withResolvers<null>();
  const started = Promise.withResolvers<null>();
  let calls = 0;
  const first = withDispatchLock(root, "fixture", async () => {
    calls += 1;
    started.resolve(null);
    await completion.promise;
  });
  await started.promise;
  try {
    const competitor = withDispatchLock(root, "fixture", () => {
      calls += 1;
      return Promise.resolve();
    });
    const failure: unknown = await competitor.catch((error: unknown) => error);
    expect(
      failure instanceof Error &&
        failure.message.includes("Existing issue dispatch lock"),
    ).toBe(true);
    expect(calls).toBe(1);
  } finally {
    completion.resolve(null);
    await first;
  }
  await withDispatchLock(root, "fixture", () => {
    calls += 1;
    return Promise.resolve();
  });
  const firstAndFollowingCallCount = 2;
  expect(calls).toBe(firstAndFollowingCallCount);
});

test("retains an abandoned lock for inspection instead of guessing its owner died", async () => {
  const root = mkdtempSync(join(tmpdir(), "herdr-dispatch-lock-"));
  roots.push(root);
  const lock = join(root, "fixture.lock");
  writeFileSync(lock, "");
  const action = withDispatchLock(root, "fixture", () => Promise.resolve());
  const failure: unknown = await action.catch((error: unknown) => error);
  expect(failure instanceof Error && failure.message.includes(lock)).toBe(true);
  expect(existsSync(lock)).toBe(true);
});

test("retains its reservation after a failure that could follow a partial native mutation", async () => {
  const root = mkdtempSync(join(tmpdir(), "herdr-dispatch-lock-"));
  roots.push(root);
  const action = withDispatchLock(root, "fixture", () =>
    Promise.reject(new Error("Native failure")),
  );
  const failure: unknown = await action.catch((error: unknown) => error);
  expect(
    failure instanceof Error && failure.message.includes("Native failure"),
  ).toBe(true);
  expect(existsSync(join(root, "fixture.lock"))).toBe(true);
});
