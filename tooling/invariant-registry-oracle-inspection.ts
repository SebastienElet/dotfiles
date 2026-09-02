import { isAbsolute, relative, sep } from "node:path";
import type { OracleInspection } from "./invariant-registry-contract.ts";

type FileSnapshot =
  | Readonly<{ kind: "missing" | "non-regular" | "symlink" }>
  | Readonly<{ device: bigint; inode: bigint; kind: "regular-file" }>;
type OracleInspectionProbes = Readonly<{
  close: (descriptor: number) => void;
  fstat: (descriptor: number) => FileSnapshot;
  gitIndexMode: (root: string, path: string) => string | undefined;
  lstat: (path: string) => FileSnapshot;
  openNoFollow: (path: string) => number;
  realpath: (path: string) => string;
}>;
type OracleInspectionRequest = Readonly<{
  invocation: readonly string[];
  path: string;
  root: string;
}>;
type OpenedInspection = Readonly<{
  descriptor: number;
  initial: Extract<FileSnapshot, { kind: "regular-file" }>;
  probes: OracleInspectionProbes;
  request: OracleInspectionRequest;
}>;

const oracleInvocationLength = 3;

const isOutside = (root: string, path: string): boolean => {
  const pathFromRoot = relative(root, path);
  return (
    pathFromRoot === ".." ||
    pathFromRoot.startsWith(`..${sep}`) ||
    isAbsolute(pathFromRoot)
  );
};

const unavailable = (kind: OracleInspection["kind"]): OracleInspection => ({
  discovered: false,
  kind,
  tracked: false,
});

const sameIdentity = (
  left: Extract<FileSnapshot, { kind: "regular-file" }>,
  right: Extract<FileSnapshot, { kind: "regular-file" }>,
): boolean => left.device === right.device && left.inode === right.inode;

const changedPath = (): never => {
  throw new Error("Oracle path changed during inspection.");
};

const inspectOpenedOracle = (context: OpenedInspection): OracleInspection => {
  const { descriptor, initial, probes, request } = context;
  const opened = probes.fstat(descriptor);
  if (opened.kind !== "regular-file") {
    return unavailable("non-regular");
  }
  if (!sameIdentity(initial, opened)) {
    return changedPath();
  }
  const target = probes.realpath(request.path);
  if (isOutside(request.root, target)) {
    return unavailable("missing");
  }
  const repositoryPath = relative(request.root, request.path);
  const indexMode = probes.gitIndexMode(request.root, repositoryPath);
  const final = probes.lstat(request.path);
  if (final.kind !== "regular-file" || !sameIdentity(initial, final)) {
    return changedPath();
  }
  const tracked = indexMode === "100644" || indexMode === "100755";
  const discovered =
    repositoryPath.endsWith(".test.ts") &&
    request.invocation.length === oracleInvocationLength &&
    request.invocation[0] === "bun" &&
    request.invocation[1] === "test" &&
    request.invocation[2] === repositoryPath;
  return { discovered, kind: "regular-file", tracked };
};

const inspectOracleWithProbes = (
  request: OracleInspectionRequest,
  probes: OracleInspectionProbes,
): OracleInspection => {
  const initial = probes.lstat(request.path);
  if (initial.kind === "missing") {
    return unavailable("missing");
  }
  if (initial.kind !== "regular-file") {
    return unavailable("non-regular");
  }
  const descriptor = probes.openNoFollow(request.path);
  try {
    return inspectOpenedOracle({ descriptor, initial, probes, request });
  } finally {
    probes.close(descriptor);
  }
};

export {
  inspectOracleWithProbes,
  type FileSnapshot,
  type OracleInspectionProbes,
  type OracleInspectionRequest,
};
