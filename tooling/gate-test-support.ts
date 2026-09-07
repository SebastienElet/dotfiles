import {
  chmodSync,
  mkdirSync,
  mkdtempSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";

const executableMode = 0o755;
const roots: string[] = [];

type GateFixture = Readonly<{ root: string; bin: string; home: string }>;

function gateFixture(): GateFixture {
  const root = mkdtempSync(join(tmpdir(), "gate-"));
  const bin = join(root, "bin");
  const home = join(root, "home");
  mkdirSync(bin);
  mkdirSync(home);
  roots.push(root);
  return { root, bin, home };
}

function executable(bin: string, name: string, body: string): void {
  const path = join(bin, name);
  writeFileSync(path, `#!/bin/sh\n${body}\n`);
  chmodSync(path, executableMode);
}

function clearGateFixtures(): void {
  for (const root of roots.splice(0)) {
    rmSync(root, { recursive: true, force: true });
  }
}

function runGate(
  entrypoint: string,
  fixture: GateFixture,
  arguments_: readonly string[] = [],
): Bun.SyncSubprocess<"pipe", "pipe"> {
  return Bun.spawnSync(
    [process.execPath, join(import.meta.dir, entrypoint), ...arguments_],
    {
      cwd: fixture.root,
      env: {
        ...process.env,
        HOME: fixture.home,
        PATH: fixture.bin,
        GATE_FIXTURE: fixture.root,
      },
    },
  );
}

export { gateFixture, executable, clearGateFixtures, runGate };
