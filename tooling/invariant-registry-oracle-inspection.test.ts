import {
  type FileSnapshot,
  type OracleInspectionProbes,
  inspectOracleWithProbes,
} from "./invariant-registry-oracle-inspection.ts";
import { expect, test } from "bun:test";

const request = {
  root: "/repo",
  path: "/repo/tooling/oracle.test.ts",
  invocation: ["bun", "test", "tooling/oracle.test.ts"],
} as const;
const descriptor = 7;

const snapshot = (
  inode: bigint,
): FileSnapshot & Readonly<{ device: bigint; inode: bigint }> => ({
  device: 1n,
  inode,
  kind: "regular-file",
});

const probes = (
  overrides: Partial<OracleInspectionProbes> = {},
): OracleInspectionProbes => ({
  close: (candidate) => {
    if (candidate !== descriptor) {
      throw new Error("unexpected descriptor");
    }
  },
  fstat: () => snapshot(1n),
  gitIndexMode: () => "100644",
  lstat: () => snapshot(1n),
  openNoFollow: () => descriptor,
  realpath: (path) => path,
  ...overrides,
});

test.each(["120000", "100664", "invalid"] as const)(
  "rejects non-regular Git index mode %s",
  (mode) => {
    const inspection = inspectOracleWithProbes(
      request,
      probes({ gitIndexMode: () => mode }),
    );

    expect(inspection.tracked).toBe(false);
  },
);

test("rejects disagreement between path and opened descriptor identity", () => {
  expect(() =>
    inspectOracleWithProbes(request, probes({ fstat: () => snapshot(2n) })),
  ).toThrow("changed during inspection");
});

test("rejects a path substituted while repository probes run", () => {
  const snapshots = [snapshot(1n), snapshot(2n)];
  expect(() =>
    inspectOracleWithProbes(
      request,
      probes({ lstat: () => snapshots.shift() ?? snapshot(2n) }),
    ),
  ).toThrow("changed during inspection");
});

test("fails closed when no-follow open refuses the path", () => {
  expect(() =>
    inspectOracleWithProbes(
      request,
      probes({
        openNoFollow: () => {
          throw new Error("no-follow refused");
        },
      }),
    ),
  ).toThrow("no-follow refused");
});
