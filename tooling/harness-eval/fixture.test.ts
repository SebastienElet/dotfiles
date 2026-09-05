import { afterEach, expect, test } from "bun:test";
import { collectObservations, prepareFixture } from "./fixture.ts";
import { mkdirSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { at } from "./test-support.ts";
import { evaluate } from "./oracle.ts";
import { join } from "node:path";
import { loadCases } from "./sources.ts";

const roots: string[] = [];
afterEach(() => {
  for (const path of roots.splice(0)) {
    rmSync(path, { recursive: true, force: true });
  }
});

test("instrumentation identifies the file actually read from nested working directories and symlinks", () => {
  const knownPathIndex = 2;
  const fixture = prepareFixture(
    process.cwd(),
    at(loadCases(process.cwd()), knownPathIndex),
  );
  roots.push(fixture.root);
  mkdirSync(join(fixture.workspace, "other/src/auth"), { recursive: true });
  writeFileSync(
    join(fixture.workspace, "other/src/auth/session.ts"),
    "unrelated shadow file",
  );
  const invoke = (cwd: string, file: string): "PASS" | "FAIL" => {
    writeFileSync(fixture.observations, "");
    expect(
      Bun.spawnSync(["cat", file], { cwd, env: fixture.env }).exitCode,
    ).toBe(0);
    return evaluate("known-path-v1", collectObservations(fixture.observations));
  };
  expect(invoke(join(fixture.workspace, "other"), "src/auth/session.ts")).toBe(
    "FAIL",
  );
  expect(invoke(join(fixture.workspace, "src/auth"), "session.ts")).toBe(
    "PASS",
  );
  symlinkSync(
    join(fixture.workspace, "src/auth/session.ts"),
    join(fixture.workspace, "target"),
  );
  expect(invoke(fixture.workspace, "target")).toBe("PASS");
});

test("instrumentation excludes arbitrary argument contents from publishable observations", () => {
  const fixture = prepareFixture(
    process.cwd(),
    at(loadCases(process.cwd()), 0),
  );
  roots.push(fixture.root);
  const result = Bun.spawnSync(
    ["colgrep-search", "synthetic-sensitive-marker"],
    { cwd: fixture.workspace, env: fixture.env },
  );
  expect(result.exitCode).toBe(0);
  expect(
    JSON.stringify(collectObservations(fixture.observations)),
  ).not.toContain("synthetic-sensitive-marker");
});
