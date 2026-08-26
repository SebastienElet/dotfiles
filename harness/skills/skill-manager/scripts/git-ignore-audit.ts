function parseIgnoredPathOutput(
  output: Readonly<ArrayLike<number>>,
  auditedPaths: readonly string[],
): ReadonlySet<string> {
  const serialized = new TextDecoder("utf-8", { fatal: true }).decode(
    Uint8Array.from(output),
  );
  if (serialized === "") {
    return new Set();
  }
  if (!serialized.endsWith("\0")) {
    throw new Error("Git ignore audit returned malformed path evidence.");
  }
  const ignoredPaths = serialized.slice(0, -1).split("\0");
  const auditedPathSet = new Set(auditedPaths);
  const ignoredPathSet = new Set(ignoredPaths);
  if (
    ignoredPathSet.size !== ignoredPaths.length ||
    ignoredPaths.some((path) => !auditedPathSet.has(path))
  ) {
    throw new Error("Git ignore audit returned malformed path evidence.");
  }
  return ignoredPathSet;
}

function findIgnoredPaths(
  skillRoot: string,
  auditedPaths: readonly string[],
): ReadonlySet<string> {
  const input = new TextEncoder().encode(`${auditedPaths.join("\0")}\0`);
  const result = Bun.spawnSync(
    ["git", "-C", skillRoot, "check-ignore", "--no-index", "--stdin", "-z"],
    { stdin: input },
  );
  if (result.exitCode === 1) {
    return new Set();
  }
  if (result.exitCode !== 0) {
    throw new Error("Git ignore audit failed.");
  }
  try {
    return parseIgnoredPathOutput(result.stdout, auditedPaths);
  } catch {
    throw new Error("Git ignore audit returned malformed path evidence.");
  }
}

export { findIgnoredPaths, parseIgnoredPathOutput };
