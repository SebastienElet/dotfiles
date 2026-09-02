import type {
  InvariantRecord,
  RegistryDiagnostic,
  ValidationOptions,
} from "./invariant-registry-schema.ts";
import { isAbsolute, relative, resolve, sep, win32 } from "node:path";

const diagnostic = (
  code: string,
  path: string,
  message: string,
): RegistryDiagnostic => ({ code, path, message });

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

const pathExistenceDiagnostics = (
  testPath: string,
  diagnosticPath: string,
  options: ValidationOptions,
): readonly RegistryDiagnostic[] => {
  try {
    return options.pathExists(testPath)
      ? []
      : [
          diagnostic(
            "missing-oracle-path",
            diagnosticPath,
            "Oracle test path does not exist.",
          ),
        ];
  } catch {
    return [
      diagnostic(
        "oracle-path-check-failed",
        diagnosticPath,
        "Oracle test path could not be checked.",
      ),
    ];
  }
};

const oracleDiagnostics = (
  record: InvariantRecord,
  path: string,
  options: ValidationOptions,
): readonly RegistryDiagnostic[] => {
  const required =
    record.controlKind === "enforceable" &&
    (record.lifecycle === "active" || record.verification.state === "verified");
  if (required) {
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
    return testPath === undefined
      ? [
          diagnostic(
            "invalid-oracle-path",
            diagnosticPath,
            "Oracle test path must stay within the repository.",
          ),
        ]
      : pathExistenceDiagnostics(testPath, diagnosticPath, options);
  }
  return [];
};

export { oracleDiagnostics };
