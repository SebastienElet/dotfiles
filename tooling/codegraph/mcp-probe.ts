import { readFileSync, renameSync, unlinkSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import {
  type Command,
  privacyEnvironment,
  requireDaemonStopped,
  runCommand,
} from "./integration-fixture.ts";
import { delay, McpClient } from "./mcp-client.ts";

type ScenarioValue = boolean | "fresh" | "alerted-stale";

export type ProbeReport = Readonly<{
  tools: readonly ["codegraph_explore"];
  scenarios: Readonly<Record<string, ScenarioValue>>;
  queryLatencyMs: Readonly<Record<string, number>>;
  synchronizationMs: number;
}>;

export async function runFreshnessProbe(
  repository: string,
  codegraph: Command,
): Promise<ProbeReport> {
  const source = join(repository, "src");
  const timings: Record<string, number> = {};
  const client = new McpClient(
    codegraph,
    repository,
    privacyEnvironment,
    timeout("REQUEST", 30_000),
    timeout("STOP", 3_000),
  );
  let watcherInterruption: "fresh" | "alerted-stale" = "fresh";
  let synchronizationMilliseconds = 0;
  try {
    await client.start();
    await waitFresh(client, timings, initialExpectation);
    await proveBranchSwitches(client, timings, repository);
    await proveMutations(client, timings, source);
    await restartAndProveFresh(client, timings);
    const reconciliation = await proveWatcherInterruption(
      client,
      timings,
      repository,
      source,
      codegraph,
    );
    watcherInterruption = reconciliation.watcherInterruption;
    synchronizationMilliseconds = reconciliation.synchronizationMilliseconds;
    await delay(2_000);
    requireDaemonStopped(repository);
  } finally {
    await client.stop();
  }
  return {
    tools: ["codegraph_explore"],
    scenarios: {
      initial: true,
      branchSwitch: true,
      edit: true,
      rename: true,
      delete: true,
      restart: true,
      watcherInterruption,
      reconciliation: true,
      daemonStopped: true,
    },
    queryLatencyMs: timings,
    synchronizationMs: synchronizationMilliseconds,
  };
}

const initialExpectation = {
  query:
    "How does entryValue depend on liveValue, removableValue, and branchMainValue?",
  current: ["entryValue", "liveValue", "removableValue", "branchMainValue"],
  stale: [],
};

async function proveBranchSwitches(
  client: McpClient,
  timings: Record<string, number>,
  repository: string,
): Promise<void> {
  runCommand(["git"], ["switch", "codegraph-alt"], repository, process.env);
  await waitFresh(client, timings, {
    query: "Where is branchAltValue defined?",
    current: ["branchAltValue", "FIXTURE_BRANCH_ALT"],
    stale: ["FIXTURE_BRANCH_MAIN"],
  });
  runCommand(["git"], ["switch", "main"], repository, process.env);
  await waitFresh(client, timings, {
    query: "Where is branchMainValue defined?",
    current: ["branchMainValue", "FIXTURE_BRANCH_MAIN"],
    stale: ["branchAltValue", "FIXTURE_BRANCH_ALT"],
  });
}

async function proveMutations(
  client: McpClient,
  timings: Record<string, number>,
  source: string,
): Promise<void> {
  const live = join(source, "live.ts");
  const renamed = join(source, "renamed-live.ts");
  const entry = join(source, "entry.ts");
  replace(live, "FIXTURE_LIVE_V1", "FIXTURE_LIVE_V2");
  await waitFresh(client, timings, {
    query: "Where is liveSentinel defined?",
    current: ["FIXTURE_LIVE_V2"],
    stale: ["FIXTURE_LIVE_V1"],
  });
  renameSync(live, renamed);
  replace(renamed, "liveValue", "renamedLiveValue");
  replace(entry, "./live.js", "./renamed-live.js");
  replace(entry, "liveValue", "renamedLiveValue");
  await waitFresh(client, timings, {
    query: "How does entryValue use renamedLiveValue?",
    current: ["src/renamed-live.ts", "renamedLiveValue"],
    stale: ["src/live.ts", "liveValue"],
  });
  unlinkSync(join(source, "removable.ts"));
  replace(entry, 'import { removableValue } from "./removable.js";\n', "");
  replace(entry, " + removableValue", "");
  await waitFresh(client, timings, {
    query: "How is entryValue computed?",
    current: ["entryValue", "renamedLiveValue"],
    stale: ["removableValue", "FIXTURE_REMOVABLE"],
  });
}

async function restartAndProveFresh(
  client: McpClient,
  timings: Record<string, number>,
): Promise<void> {
  await client.stop();
  await client.start();
  await waitFresh(client, timings, {
    query: "How is entryValue computed after restart?",
    current: ["entryValue", "renamedLiveValue"],
    stale: ["removableValue", "liveValue"],
  });
  await client.stop();
  await delay(2_000);
}

async function proveWatcherInterruption(
  client: McpClient,
  timings: Record<string, number>,
  repository: string,
  source: string,
  codegraph: Command,
) {
  await client.start(["--no-watch"]);
  replace(
    join(source, "branch.ts"),
    "FIXTURE_BRANCH_MAIN",
    "FIXTURE_WATCHER_INTERRUPTED",
  );
  const query = "Where is branchSentinel defined?";
  const queryStarted = performance.now();
  const immediate = await client.explore(query, true);
  timings[query] = Math.round(performance.now() - queryStarted);
  const immediateFresh = immediate.includes("FIXTURE_WATCHER_INTERRUPTED");
  const explicitStale = /stale|out[- ]of[- ]date|sync|refresh|reindex/i.test(
    immediate,
  );
  if (!immediateFresh && !explicitStale) {
    throw new Error(
      `watcher interruption returned silent stale output: ${immediate}`,
    );
  }
  await client.stop();
  const started = performance.now();
  runCommand(codegraph, ["sync", repository], repository);
  const synchronizationMilliseconds = Math.round(performance.now() - started);
  await client.start();
  await waitFresh(client, timings, {
    query: "Where is branchSentinel defined after reconciliation?",
    current: ["FIXTURE_WATCHER_INTERRUPTED"],
    stale: ["FIXTURE_BRANCH_MAIN"],
  });
  await client.stop();
  return {
    watcherInterruption: immediateFresh
      ? ("fresh" as const)
      : ("alerted-stale" as const),
    synchronizationMilliseconds,
  };
}

async function waitFresh(
  client: McpClient,
  timings: Record<string, number>,
  expectation: Readonly<{
    query: string;
    current: readonly string[];
    stale: readonly string[];
  }>,
): Promise<void> {
  let last = "";
  for (let attempt = 0; attempt < 30; attempt += 1) {
    const started = performance.now();
    try {
      last = await client.explore(expectation.query);
      timings[expectation.query] = Math.round(performance.now() - started);
      if (
        expectation.current.every((term) => last.includes(term)) &&
        expectation.stale.every((term) => !last.includes(term))
      )
        return;
    } catch (error) {
      last = String(error);
    }
    await delay(300);
  }
  throw new Error(
    `freshness timeout for ${expectation.query}: ${last}\n${client.diagnostic()}`,
  );
}

function replace(file: string, before: string, after: string): void {
  const current = readFileSync(file, "utf8");
  if (!current.includes(before))
    throw new Error(`missing fixture text: ${before}`);
  writeFileSync(file, current.replaceAll(before, after));
}

function timeout(name: string, fallback: number): number {
  const value = process.env[`CODEGRAPH_MCP_${name}_TIMEOUT_MS`];
  if (value === undefined) return fallback;
  const parsed = Number(value);
  if (!Number.isSafeInteger(parsed) || parsed < 1 || parsed > 30_000) {
    throw new Error(`invalid MCP ${name.toLowerCase()} timeout: ${value}`);
  }
  return parsed;
}
