type ClosedDirectoryPolicy = Readonly<{
  files: readonly string[];
  mode: "closed";
}>;
type OpenDirectoryPolicy = Readonly<{ mode: "open" }>;
type ResourceFilePolicy = Readonly<{
  resourceDirectories: Readonly<
    Record<string, ClosedDirectoryPolicy | OpenDirectoryPolicy>
  >;
  rootFiles: readonly string[];
  version: 1;
}>;

function isRecord(value: unknown): value is Readonly<Record<string, unknown>> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function hasExactKeys(
  value: Readonly<Record<string, unknown>>,
  expectedKeys: readonly string[],
): boolean {
  const keys = Object.keys(value);
  return (
    keys.length === expectedKeys.length &&
    keys.every((key) => expectedKeys.includes(key))
  );
}

function isFileName(value: unknown): value is string {
  return (
    typeof value === "string" &&
    value.length > 0 &&
    !value.includes("/") &&
    !value.includes("\\")
  );
}

function parseFileNames(value: unknown): readonly string[] {
  if (
    !Array.isArray(value) ||
    value.length === 0 ||
    !value.every((entry) => isFileName(entry))
  ) {
    throw new Error("Invalid resource file policy.");
  }
  return value;
}

function parseDirectoryPolicy(
  value: unknown,
): ClosedDirectoryPolicy | OpenDirectoryPolicy {
  if (!isRecord(value)) {
    throw new Error("Invalid resource file policy.");
  }
  if (value.mode === "open" && hasExactKeys(value, ["mode"])) {
    return { mode: "open" };
  }
  if (value.mode === "closed" && hasExactKeys(value, ["files", "mode"])) {
    return { files: parseFileNames(value.files), mode: "closed" };
  }
  throw new Error("Invalid resource file policy.");
}

function parseResourceFilePolicy(input: unknown): ResourceFilePolicy {
  if (
    !isRecord(input) ||
    !hasExactKeys(input, ["resourceDirectories", "rootFiles", "version"]) ||
    input.version !== 1 ||
    !isRecord(input.resourceDirectories)
  ) {
    throw new Error("Invalid resource file policy.");
  }
  const resourceDirectories: Record<
    string,
    ClosedDirectoryPolicy | OpenDirectoryPolicy
  > = {};
  for (const [name, policy] of Object.entries(input.resourceDirectories)) {
    if (!isFileName(name)) {
      throw new Error("Invalid resource file policy.");
    }
    resourceDirectories[name] = parseDirectoryPolicy(policy);
  }
  return {
    resourceDirectories,
    rootFiles: parseFileNames(input.rootFiles),
    version: 1,
  };
}

export { parseResourceFilePolicy };
export type { ResourceFilePolicy };
