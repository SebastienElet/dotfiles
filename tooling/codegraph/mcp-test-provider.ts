#!/usr/bin/env bun

const malformedPauseMilliseconds = 60_000;
const scenario = process.env.CODEGRAPH_MCP_TEST_SCENARIO;
const startupExitCode = 42;
const usageExitCode = 64;

if (scenario === "startup") {
  process.exit(startupExitCode);
}
if (scenario === "timeout") {
  await Bun.sleep(malformedPauseMilliseconds);
}
if (scenario === "malformed") {
  process.stdin.once("data", () => {
    process.stdout.write('{"jsonrpc":"2.0","id":1,"result":\n');
  });
  await Bun.sleep(malformedPauseMilliseconds);
}

process.stderr.write(`unknown MCP test scenario: ${scenario ?? "missing"}\n`);
process.exit(usageExitCode);
