import { BindingConflictError, patchBinding } from "./binding-patch.ts";
import { expect, test } from "bun:test";
import type { BindingRequest } from "./binding-patch.ts";
import type { Bindings } from "./binding-store.ts";

const identity = {
  issue_agent: "codex",
  issue_name: "fixture",
  issue_phase: "starting",
  issue_role: "work",
  issue_checkout: "/work",
  issue_repo: "/source/.git",
  issue_workspace: "w2",
  issue_url: "https://github.com/fixture/project/issues/17",
};

test("rereads and preserves an unrelated concurrent binding before retrying", async () => {
  let state: Bindings = {};
  let writes = 0;
  const unrelated = {
    ...identity,
    issue_url: "https://github.com/fixture/project/issues/18",
  };
  const request = (data: BindingRequest): Promise<Bindings> => {
    if (data.operation === "read") {
      return Promise.resolve(structuredClone(state));
    }
    writes += 1;
    if (writes === 1) {
      state = { other: unrelated };
      return Promise.reject(new BindingConflictError("stale"));
    }
    expect(data.expected).toEqual(state);
    state = { ...state, [data.pane_id]: data.tokens };
    return Promise.resolve(state);
  };
  await patchBinding("w2:p1", identity, request);
  expect(state).toEqual({ other: unrelated, "w2:p1": identity });
});

test("rereads after a definitely unapplied busy-writer conflict", async () => {
  let writes = 0;
  const request = (data: BindingRequest): Promise<Bindings> => {
    if (data.operation === "read") {
      return Promise.resolve({});
    }
    writes += 1;
    return writes === 1
      ? Promise.reject(new BindingConflictError("busy"))
      : Promise.resolve({ [data.pane_id]: data.tokens });
  };
  await patchBinding("w2:p1", identity, request);
  const expectedWrites = 2;
  expect(writes).toBe(expectedWrites);
});

const conflictingBindings: readonly Bindings[] = [
  { other: identity },
  { "w2:p1": { ...identity, issue_session: "replacement" } },
];
for (const changed of conflictingBindings) {
  test("refuses a changed issue owner or target-pane binding after contention", async () => {
    let state: Bindings = {};
    let writes = 0;
    const request = (data: BindingRequest): Promise<Bindings> => {
      if (data.operation === "read") {
        return Promise.resolve(structuredClone(state));
      }
      writes += 1;
      state = changed;
      return Promise.reject(new BindingConflictError("stale"));
    };
    const result = await patchBinding("w2:p1", identity, request).catch(
      (error: unknown) => error,
    );
    expect(result).toBeInstanceOf(Error);
    expect(writes).toBe(1);
    expect(state).toEqual(changed);
  });
}

test("bounds persistent contention without replacing state", async () => {
  let writes = 0;
  const request = (data: BindingRequest): Promise<Bindings> => {
    if (data.operation === "read") {
      return Promise.resolve({});
    }
    writes += 1;
    return Promise.reject(new BindingConflictError("busy"));
  };
  const result = await patchBinding("w2:p1", identity, request).catch(
    (error: unknown) => error,
  );
  expect(result).toMatchObject({ message: "busy" });
  const maximumWrites = 3;
  expect(writes).toBe(maximumWrites);
});

test("does not retry an uncertain write failure", async () => {
  let writes = 0;
  const request = (data: BindingRequest): Promise<Bindings> => {
    if (data.operation === "read") {
      return Promise.resolve({});
    }
    writes += 1;
    return Promise.reject(new Error("unknown write outcome"));
  };
  const result = await patchBinding("w2:p1", identity, request).catch(
    (error: unknown) => error,
  );
  expect(result).toMatchObject({ message: "unknown write outcome" });
  expect(writes).toBe(1);
});

test("does not mutate again when rereading after contention fails", async () => {
  let reads = 0;
  let writes = 0;
  const request = (data: BindingRequest): Promise<Bindings> => {
    if (data.operation === "read") {
      reads += 1;
      return reads === 1
        ? Promise.resolve({})
        : Promise.reject(new Error("read failed"));
    }
    writes += 1;
    return Promise.reject(new BindingConflictError("busy"));
  };
  const result = await patchBinding("w2:p1", identity, request).catch(
    (error: unknown) => error,
  );
  expect(result).toMatchObject({ message: "read failed" });
  expect(writes).toBe(1);
});
