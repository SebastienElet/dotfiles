import {
  type Command,
  privacyEnvironment,
  requireDaemonStopped,
  runCommand,
} from "./integration-fixture.ts";
import { McpClient, delay } from "./mcp-client.ts";
import { readFileSync, renameSync, unlinkSync, writeFileSync } from "node:fs";
import type { ProbeReport } from "./mcp-probe-report.ts";
import { join } from "node:path";
import { mcpTimeout } from "./mcp-timeout.ts";

type FreshnessClient = Readonly<
  Pick<McpClient, "diagnostic" | "explore" | "start" | "stop">
>;
type TimingRecorder = Readonly<{
  record: (query: string, milliseconds: number) => void;
}>;
type WatcherInterruptionOptions = Readonly<{
  client: FreshnessClient;
  codegraph: Command;
  repository: string;
  source: string;
  timings: TimingRecorder;
}>;
type FreshnessScenarioOptions = Readonly<{
  client: FreshnessClient;
  codegraph: Command;
  repository: string;
  timings: TimingRecorder;
}>;

const initialExpectation = {
  current: ["entryValue", "liveValue", "removableValue", "branchMainValue"],
  query:
    "How does entryValue depend on liveValue, removableValue, and branchMainValue?",
  stale: [],
};
const defaultRequestTimeoutMilliseconds = 30_000;
const defaultStopTimeoutMilliseconds = 3000;
const daemonShutdownDelayMilliseconds = 2000;
const freshnessAttemptLimit = 30;
const freshnessPollDelayMilliseconds = 300;

async function runFreshnessProbe(
  repository: string,
  codegraph: Command,
): Promise<ProbeReport> {
  const timingValues: Record<string, number> = {};
  const timings: TimingRecorder = {
    record: (query, milliseconds) => {
      timingValues[query] = milliseconds;
    },
  };
  const client = createFreshnessClient(repository, codegraph);
  const outcome = await proveFreshnessScenarios({
    client,
    codegraph,
    repository,
    timings,
  });
  return {
    queryLatencyMs: timingValues,
    scenarios: {
      branchSwitch: true,
      daemonStopped: true,
      delete: true,
      edit: true,
      initial: true,
      reconciliation: true,
      rename: true,
      restart: true,
      watcherInterruption: outcome.watcherInterruption,
    },
    synchronizationMs: outcome.synchronizationMilliseconds,
    tools: ["codegraph_explore"],
  };
}

function createFreshnessClient(
  repository: string,
  codegraph: Command,
): McpClient {
  return new McpClient({
    command: codegraph,
    environment: privacyEnvironment,
    repository,
    requestTimeoutMilliseconds: mcpTimeout(
      "REQUEST",
      defaultRequestTimeoutMilliseconds,
    ),
    stopTimeoutMilliseconds: mcpTimeout("STOP", defaultStopTimeoutMilliseconds),
  });
}

async function proveFreshnessScenarios({
  client,
  codegraph,
  repository,
  timings,
}: FreshnessScenarioOptions): Promise<{
  synchronizationMilliseconds: number;
  watcherInterruption: "fresh" | "alerted-stale";
}> {
  const source = join(repository, "src");
  try {
    await client.start();
    await waitFresh(client, timings, initialExpectation);
    await proveBranchSwitches(client, timings, repository);
    await proveMutations(client, timings, source);
    await restartAndProveFresh(client, timings);
    const outcome = await proveWatcherInterruption({
      client,
      codegraph,
      repository,
      source,
      timings,
    });
    await delay(daemonShutdownDelayMilliseconds);
    requireDaemonStopped(repository);
    return outcome;
  } finally {
    await client.stop();
  }
}

async function proveBranchSwitches(
  client: FreshnessClient,
  timings: TimingRecorder,
  repository: string,
): Promise<void> {
  runCommand(["git"], ["switch", "codegraph-alt"], {
    cwd: repository,
    environment: process.env,
  });
  await waitFresh(client, timings, {
    current: ["branchAltValue", "FIXTURE_BRANCH_ALT"],
    query: "Where is branchAltValue defined?",
    stale: ["FIXTURE_BRANCH_MAIN"],
  });
  runCommand(["git"], ["switch", "main"], {
    cwd: repository,
    environment: process.env,
  });
  await waitFresh(client, timings, {
    current: ["branchMainValue", "FIXTURE_BRANCH_MAIN"],
    query: "Where is branchMainValue defined?",
    stale: ["branchAltValue", "FIXTURE_BRANCH_ALT"],
  });
}

async function proveMutations(
  client: FreshnessClient,
  timings: TimingRecorder,
  source: string,
): Promise<void> {
  const live = join(source, "live.ts");
  const renamed = join(source, "renamed-live.ts");
  const entry = join(source, "entry.ts");
  replace(live, "FIXTURE_LIVE_V1", "FIXTURE_LIVE_V2");
  await waitFresh(client, timings, {
    current: ["FIXTURE_LIVE_V2"],
    query: "Where is liveSentinel defined?",
    stale: ["FIXTURE_LIVE_V1"],
  });
  renameSync(live, renamed);
  replace(renamed, "liveValue", "renamedLiveValue");
  replace(entry, "./live.js", "./renamed-live.js");
  replace(entry, "liveValue", "renamedLiveValue");
  await waitFresh(client, timings, {
    current: ["src/renamed-live.ts", "renamedLiveValue"],
    query: "How does entryValue use renamedLiveValue?",
    stale: ["src/live.ts", "liveValue"],
  });
  unlinkSync(join(source, "removable.ts"));
  replace(entry, 'import { removableValue } from "./removable.js";\n', "");
  replace(entry, " + removableValue", "");
  await waitFresh(client, timings, {
    current: ["entryValue", "renamedLiveValue"],
    query: "How is entryValue computed?",
    stale: ["removableValue", "FIXTURE_REMOVABLE"],
  });
}

async function restartAndProveFresh(
  client: FreshnessClient,
  timings: TimingRecorder,
): Promise<void> {
  await client.stop();
  await client.start();
  await waitFresh(client, timings, {
    current: ["entryValue", "renamedLiveValue"],
    query: "How is entryValue computed after restart?",
    stale: ["removableValue", "liveValue"],
  });
  await client.stop();
  await delay(daemonShutdownDelayMilliseconds);
}

async function proveWatcherInterruption({
  client,
  codegraph,
  repository,
  source,
  timings,
}: WatcherInterruptionOptions): Promise<{
  synchronizationMilliseconds: number;
  watcherInterruption: "fresh" | "alerted-stale";
}> {
  await client.start(["--no-watch"]);
  replace(
    join(source, "branch.ts"),
    "FIXTURE_BRANCH_MAIN",
    "FIXTURE_WATCHER_INTERRUPTED",
  );
  const query = "Where is branchSentinel defined?";
  const queryStarted = performance.now();
  const immediate = await client.explore(query, true);
  timings.record(query, Math.round(performance.now() - queryStarted));
  const immediateFresh = immediate.includes("FIXTURE_WATCHER_INTERRUPTED");
  const explicitStale = /stale|out[- ]of[- ]date|sync|refresh|reindex/iu.test(
    immediate,
  );
  if (!immediateFresh && !explicitStale) {
    throw new Error(
      `watcher interruption returned silent stale output: ${immediate}`,
    );
  }
  await client.stop();
  const started = performance.now();
  runCommand(codegraph, ["sync", repository], { cwd: repository });
  const synchronizationMilliseconds = Math.round(performance.now() - started);
  await client.start();
  await waitFresh(client, timings, {
    current: ["FIXTURE_WATCHER_INTERRUPTED"],
    query: "Where is branchSentinel defined after reconciliation?",
    stale: ["FIXTURE_BRANCH_MAIN"],
  });
  await client.stop();
  return {
    synchronizationMilliseconds,
    watcherInterruption: immediateFresh
      ? ("fresh" as const)
      : ("alerted-stale" as const),
  };
}

async function waitFresh(
  client: FreshnessClient,
  timings: TimingRecorder,
  expectation: Readonly<{
    query: string;
    current: readonly string[];
    stale: readonly string[];
  }>,
): Promise<void> {
  let last = "";
  for (let attempt = 0; attempt < freshnessAttemptLimit; attempt += 1) {
    const started = performance.now();
    try {
      last = await client.explore(expectation.query);
      timings.record(
        expectation.query,
        Math.round(performance.now() - started),
      );
      if (
        includesEvery(last, expectation.current) &&
        includesNone(last, expectation.stale)
      ) {
        return;
      }
    } catch (error) {
      last = String(error);
    }
    await delay(freshnessPollDelayMilliseconds);
  }
  throw new Error(
    `freshness timeout for ${expectation.query}: ${last}\n${client.diagnostic()}`,
  );
}

function includesEvery(value: string, terms: readonly string[]): boolean {
  return terms.every((term) => value.includes(term));
}

function includesNone(value: string, terms: readonly string[]): boolean {
  return terms.every((term) => !value.includes(term));
}

function replace(file: string, before: string, after: string): void {
  const current = readFileSync(file, "utf8");
  if (!current.includes(before)) {
    throw new Error(`missing fixture text: ${before}`);
  }
  writeFileSync(file, current.replaceAll(before, after));
}

export type { ProbeReport } from "./mcp-probe-report.ts";
export { runFreshnessProbe };
