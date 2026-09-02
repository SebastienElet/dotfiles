import type {
  InvariantRecord,
  OracleInspection,
  RegistryDiagnostic,
  ValidationOptions,
} from "./invariant-registry-schema.ts";
import { isAbsolute, relative, resolve, sep, win32 } from "node:path";

const diagnostic = (
  code: string,
  path: string,
  message: string,
): RegistryDiagnostic => ({ code, path, message });
const oracleInvocationLength = 3;

const resolveOracleTestPath = (
  repositoryRoot: string,
  testPath: string,
): string | undefined => {
  const windowsPath = win32.normalize(testPath);
  if (
    isAbsolute(testPath) ||
    win32.parse(testPath).root !== "" ||
    windowsPath === ".." ||
    windowsPath.startsWith(`..${win32.sep}`)
  ) {
    return undefined;
  }
  const root = resolve(repositoryRoot);
  const candidate = resolve(root, testPath);
  const pathFromRoot = relative(root, candidate);
  return pathFromRoot === ".." ||
    pathFromRoot.startsWith(`..${sep}`) ||
    isAbsolute(pathFromRoot)
    ? undefined
    : candidate;
};

const inspectionDiagnostics = (
  inspection: OracleInspection,
  path: string,
): readonly RegistryDiagnostic[] => {
  if (inspection.kind === "missing") {
    return [
      diagnostic(
        "missing-oracle-path",
        path,
        "Oracle test path does not exist.",
      ),
    ];
  }
  if (inspection.kind === "non-regular") {
    return [
      diagnostic(
        "non-regular-oracle-path",
        path,
        "Oracle test path must be a regular file.",
      ),
    ];
  }
  return [
    ...(inspection.tracked
      ? []
      : [
          diagnostic(
            "untracked-oracle-path",
            path,
            "Oracle test path must be tracked by Git.",
          ),
        ]),
    ...(inspection.discovered
      ? []
      : [
          diagnostic(
            "undiscovered-oracle-path",
            path,
            "Oracle test path must be discovered by the test suite.",
          ),
        ]),
  ];
};

const canonicalInvocation = (record: InvariantRecord): boolean =>
  record.oracle !== undefined &&
  record.oracle.invocation.length === oracleInvocationLength &&
  record.oracle.invocation[0] === "bun" &&
  record.oracle.invocation[1] === "test" &&
  record.oracle.invocation[2] === record.oracle.testPath;

const measurementMatchesOracle = (record: InvariantRecord): boolean => {
  if (
    record.oracle === undefined ||
    record.verification.state !== "verified" ||
    record.verification.lastRun.oracle === undefined
  ) {
    return false;
  }
  const measured = record.verification.lastRun.oracle;
  return (
    measured.name === record.oracle.name &&
    measured.testPath === record.oracle.testPath &&
    measured.invocation.length === record.oracle.invocation.length &&
    measured.invocation.every(
      (argument, index) => argument === record.oracle?.invocation[index],
    )
  );
};

const inspectOracle = (
  context: Readonly<{
    diagnosticPath: string;
    options: ValidationOptions;
    record: InvariantRecord;
    testPath: string;
  }>,
): readonly RegistryDiagnostic[] => {
  try {
    return inspectionDiagnostics(
      context.options.inspectOracle(
        context.testPath,
        context.record.oracle?.invocation ?? [],
      ),
      context.diagnosticPath,
    );
  } catch {
    return [
      diagnostic(
        "oracle-path-check-failed",
        context.diagnosticPath,
        "Oracle test path could not be checked.",
      ),
    ];
  }
};

const configuredOracleEvidenceDiagnostics = (
  record: InvariantRecord,
  path: string,
): readonly RegistryDiagnostic[] => [
  ...(canonicalInvocation(record)
    ? []
    : [
        diagnostic(
          "invalid-oracle-invocation",
          `${path}.oracle.invocation`,
          "Oracle invocation must run its exact test path.",
        ),
      ]),
  ...(record.verification.state !== "verified" ||
  measurementMatchesOracle(record)
    ? []
    : [
        diagnostic(
          "oracle-measurement-mismatch",
          `${path}.verification.lastRun.oracle`,
          "Verified measurement must identify the exact oracle and invocation.",
        ),
      ]),
];

const configuredOracleDiagnostics = (
  record: InvariantRecord,
  path: string,
  options: ValidationOptions,
): readonly RegistryDiagnostic[] => {
  if (record.oracle === undefined) {
    return [
      diagnostic(
        "missing-oracle",
        `${path}.oracle`,
        "Enforceable active or verified invariants require an oracle.",
      ),
    ];
  }
  const diagnosticPath = `${path}.oracle.testPath`;
  const testPath = resolveOracleTestPath(
    options.repositoryRoot,
    record.oracle.testPath,
  );
  if (testPath === undefined) {
    return [
      diagnostic(
        "invalid-oracle-path",
        diagnosticPath,
        "Oracle test path must stay within the repository.",
      ),
    ];
  }
  return [
    ...configuredOracleEvidenceDiagnostics(record, path),
    ...inspectOracle({ diagnosticPath, options, record, testPath }),
  ];
};

const oracleDiagnostics = (
  record: InvariantRecord,
  path: string,
  options: ValidationOptions,
): readonly RegistryDiagnostic[] => {
  const required =
    record.controlKind === "enforceable" &&
    (record.lifecycle === "active" || record.verification.state === "verified");
  return required ? configuredOracleDiagnostics(record, path, options) : [];
};

export { oracleDiagnostics };
