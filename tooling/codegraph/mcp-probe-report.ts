type ScenarioValue = boolean | "fresh" | "alerted-stale";

type ProbeReport = Readonly<{
  queryLatencyMs: Readonly<Record<string, number>>;
  scenarios: Readonly<Record<string, ScenarioValue>>;
  synchronizationMs: number;
  tools: readonly ["codegraph_explore"];
}>;

export { type ProbeReport };
