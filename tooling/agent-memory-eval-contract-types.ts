type ContractEvent = Readonly<Record<string, unknown>>;
type SessionExpectation = Readonly<{
  cache?: string;
  nonce: string;
  runtime: string;
  source: string;
  version: string;
}>;
type SessionObservation = Readonly<{
  cacheAbsentBefore?: boolean;
  cacheCompletedBeforeModel?: boolean;
  cachePath?: string;
  runtimeTrace?: string;
  traceCompletedBeforeModel?: boolean;
  version?: string;
}>;
type SessionTrace = Readonly<{
  adapterCompletedBeforeModel: boolean;
  contextBeforeModel: boolean;
  modelText: string;
  version: string;
}>;
type AgentText = Readonly<{ modelText: string; version: string }>;

export type {
  AgentText,
  ContractEvent,
  SessionExpectation,
  SessionObservation,
  SessionTrace,
};
export type { Agent } from "./agent-memory-eval-process.ts";
