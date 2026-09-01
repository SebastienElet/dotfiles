type Agent = "codex" | "claude" | "cursor";
type AgentCondition =
  | "admission"
  | "contradiction"
  | "control"
  | "proposal"
  | "relevant"
  | "sensitive"
  | "unavailable"
  | "unrelated";
type ProcessOutput = Readonly<{
  cacheAbsentBefore?: boolean;
  cacheCompletedBeforeModel?: boolean;
  cachePath?: string;
  exitCode: number;
  runtimeTrace?: string;
  stderr: string;
  stdout: string;
  traceAbsent?: boolean;
  traceCompletedBeforeModel?: boolean;
}>;

export type { Agent, AgentCondition, ProcessOutput };
