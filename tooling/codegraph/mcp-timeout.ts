const maximumTimeoutMilliseconds = 30_000;

function mcpTimeout(name: string, fallback: number): number {
  const value = process.env[`CODEGRAPH_MCP_${name}_TIMEOUT_MS`];
  if (value === undefined) {
    return fallback;
  }
  const parsed = Number(value);
  if (
    !Number.isSafeInteger(parsed) ||
    parsed < 1 ||
    parsed > maximumTimeoutMilliseconds
  ) {
    throw new Error(`invalid MCP ${name.toLowerCase()} timeout: ${value}`);
  }
  return parsed;
}

export { mcpTimeout };
