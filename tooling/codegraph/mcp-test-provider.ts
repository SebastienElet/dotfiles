#!/usr/bin/env bun

const scenario = process.env.CODEGRAPH_MCP_TEST_SCENARIO;

if (scenario === "startup") process.exit(42);
if (scenario === "timeout") await Bun.sleep(60_000);
if (scenario === "malformed") {
  process.stdin.once("data", () =>
    process.stdout.write('{"jsonrpc":"2.0","id":1,"result":\n'),
  );
  await Bun.sleep(60_000);
}

process.stderr.write(`unknown MCP test scenario: ${scenario ?? "missing"}\n`);
process.exit(64);
