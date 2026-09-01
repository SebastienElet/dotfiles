import { lstat, readFile, readdir } from "node:fs/promises";
import { isAbsolute, join, relative, resolve } from "node:path";

import { runtimeCommand, treeDigest } from "./agent-memory-eval-fixture.ts";
import type { Agent } from "./agent-memory-eval-process.ts";

const project = resolve(import.meta.dir, "..");

type ProposalValidation = Readonly<{
  evaluatedStoreUnchanged: boolean;
  statementDetected: boolean;
  stored: boolean;
}>;
type RecoveryRelation = Readonly<{ profile: string; seed: string; window: string }>;

function extractProposal(text: string): string {
  const matches = [...text.matchAll(/```ya?ml\s*\n([\s\S]*?)\n```/gu)];
  if (matches.length !== 1) throw new Error("expected exactly one fenced YAML proposal");
  const proposal = matches[0]?.[1]?.trim();
  if (proposal === undefined || proposal === "") throw new Error("empty YAML proposal");
  return proposal;
}

async function validateProposalWithRuntime(options: Readonly<{
  environment: NodeJS.ProcessEnv;
  evaluatedStore: string;
  expectedRelation: RecoveryRelation;
  proposal: string;
  repository: string;
  runtime: string;
  validationStore: string;
}>): Promise<ProposalValidation> {
  const before = await treeDigest(options.evaluatedStore);
  const output = await runtimeCommand(
    options.runtime,
    ["admit", "--format", "json"],
    options.proposal,
    options.repository,
    { ...options.environment, AGENT_MEMORY_ROOT: options.validationStore },
  );
  const response: unknown = JSON.parse(output.stdout);
  const stored = isRecord(response) && response.status === "stored";
  return {
    evaluatedStoreUnchanged: before === (await treeDigest(options.evaluatedStore)),
    statementDetected: proposalStatesAcceptedRecovery(options.proposal, options.expectedRelation),
    stored,
  };
}

function proposalStatesAcceptedRecovery(
  proposal: string,
  relation: RecoveryRelation,
): boolean {
  const parsed: unknown = Bun.YAML.parse(proposal);
  if (!isRecord(parsed) || typeof parsed.statement !== "string") {
    throw new Error("proposal statement is missing");
  }
  const statement = parsed.statement.toLowerCase();
  if (
    ![relation.window, relation.seed, relation.profile].every((term) =>
      statement.includes(term.toLowerCase()),
    ) ||
    !/\baccepted?\b/u.test(statement) ||
    /\b(?:not|never)\s+(?:the\s+)?accepted?\b/u.test(statement)
  ) {
    throw new Error("proposal does not state the accepted recovery relation");
  }
  return true;
}

function isRecord(value: unknown): value is Readonly<Record<string, unknown>> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

async function validateStoreArtifacts(store: string): Promise<boolean> {
  const artifacts = await storeArtifacts(store);
  if (artifacts.some((artifact) => artifact.mode !== (artifact.directory ? 0o700 : 0o600))) {
    return false;
  }
  const index = await readJson(join(store, "index.json"));
  const cache = await readJson(join(store, "oracle-cache.json"));
  if (index?.schema_version !== 2 || cache?.schema_version !== 1) return false;
  const rows = records(index.entries);
  const cached = records(cache.entries);
  if (rows === undefined || cached === undefined) return false;
  const indexedIds = new Set<string>();
  const yamlPaths = new Set(
    artifacts
      .filter((artifact) => !artifact.directory && artifact.path.endsWith(".yaml"))
      .map((artifact) => artifact.path),
  );
  for (const row of rows) {
    if (typeof row.id !== "string" || typeof row.path !== "string") return false;
    if (!confinedExistingYaml(store, row.path, yamlPaths)) return false;
    indexedIds.add(row.id);
  }
  return cached.every((entry) => freshSpecificCacheEntry(entry, indexedIds));
}

async function validateStoredProposal(
  store: string,
  entryId: string,
  proposal: string,
): Promise<boolean> {
  const index = await readJson(join(store, "index.json"));
  const row = records(index?.entries)?.find((entry) => entry.id === entryId);
  if (typeof row?.path !== "string") return false;
  const path = resolve(store, row.path);
  if (relative(resolve(store), path).startsWith("..")) return false;
  const [stored, expected] = await Promise.all([
    readFile(path, "utf8").then((value) => Bun.YAML.parse(value)),
    Promise.resolve(Bun.YAML.parse(proposal)),
  ]);
  if (!isRecord(stored) || !isRecord(expected) || stored.id !== entryId) return false;
  return Object.entries(expected).every(
    ([key, value]) => equalStoredField(key, stored[key], value),
  );
}

function equalStoredField(key: string, stored: unknown, expected: unknown): boolean {
  if (key === "scope" && typeof expected === "string" && isRecord(stored)) {
    return stored.type === expected;
  }
  return equalValue(stored, expected);
}

function equalValue(left: unknown, right: unknown): boolean {
  if (Array.isArray(left) || Array.isArray(right)) {
    return (
      Array.isArray(left) &&
      Array.isArray(right) &&
      left.length === right.length &&
      left.every((value, index) => equalValue(value, right[index]))
    );
  }
  if (isRecord(left) || isRecord(right)) {
    if (!isRecord(left) || !isRecord(right)) return false;
    return Object.keys(right).every(
      (key) => key in left && equalValue(left[key], right[key]),
    );
  }
  return left === right;
}

async function storedProposalEntryId(store: string, proposal: string): Promise<string> {
  const index = await readJson(join(store, "index.json"));
  const rows = records(index?.entries) ?? [];
  const matches: string[] = [];
  for (const row of rows) {
    if (
      typeof row.id === "string" &&
      (await validateStoredProposal(store, row.id, proposal))
    ) {
      matches.push(row.id);
    }
  }
  if (matches.length !== 1) {
    const mismatch =
      rows.length === 1 && typeof rows[0]?.path === "string"
        ? await storedProposalMismatchKeys(store, rows[0].path, proposal)
        : [];
    throw new Error(
      `store lacks one exact admitted proposal${mismatch.length === 0 ? "" : `: ${mismatch.join(",")}`}`,
    );
  }
  return matches[0] ?? "";
}

async function storedProposalMismatchKeys(
  store: string,
  path: string,
  proposal: string,
): Promise<string[]> {
  const stored: unknown = Bun.YAML.parse(await readFile(resolve(store, path), "utf8"));
  const expected: unknown = Bun.YAML.parse(proposal);
  if (!isRecord(stored) || !isRecord(expected)) return ["document"];
  return Object.entries(expected)
    .filter(([key, value]) => !equalStoredField(key, stored[key], value))
    .map(([key]) => key);
}

async function validateAdapterInstallation(options: Readonly<{
  agent: Agent;
  home: string;
  runtime: string;
  runtimeSource: string;
}>): Promise<boolean> {
  const runtimeMode = (await lstat(options.runtime)).mode & 0o777;
  if (runtimeMode !== 0o700) return false;
  const [runtime, source] = await Promise.all([
    readFile(options.runtime),
    readFile(options.runtimeSource),
  ]);
  if (!runtime.equals(source)) return false;
  const skillRoot = join(
    options.home,
    options.agent === "codex" ? ".agents" : options.agent === "claude" ? ".claude" : ".cursor",
    "skills/memory-governance",
  );
  if (!(await privateCanonicalFile(join(skillRoot, "SKILL.md"), "harness/skills/memory-governance/SKILL.md"))) return false;
  if (!(await privateCanonicalFile(join(skillRoot, "references/entry-contract.md"), "harness/skills/memory-governance/references/entry-contract.md"))) return false;
  return options.agent === "cursor"
    ? privateCanonicalFile(
        join(options.home, ".cursor/rules/memory-governance-cursor.mdc"),
        "harness/rules/memory-governance-cursor.mdc",
      )
    : exactHookConfig(options.agent, options.home, options.runtime);
}

async function privateCanonicalFile(path: string, source: string): Promise<boolean> {
  const mode = (await lstat(path)).mode & 0o777;
  if (mode !== 0o600) return false;
  const [actual, canonical] = await Promise.all([readFile(path), readFile(join(project, source))]);
  return actual.equals(canonical);
}

async function exactHookConfig(agent: Agent, home: string, runtime: string): Promise<boolean> {
  const directory = join(home, agent === "codex" ? ".codex" : ".claude");
  const path = join(directory, agent === "codex" ? "hooks.json" : "settings.json");
  if (((await lstat(path)).mode & 0o777) !== 0o600) return false;
  const command = `'${runtime.replaceAll("'", "'\\''")}' hook --agent ${agent}`;
  const expected = `${JSON.stringify({ hooks: { UserPromptSubmit: [{ hooks: [{ command, timeout: 30, type: "command" }] }] } }, null, 2)}\n`;
  return (await readFile(path, "utf8")) === expected;
}

type StoreArtifact = Readonly<{ directory: boolean; mode: number; path: string }>;

async function storeArtifacts(root: string): Promise<readonly StoreArtifact[]> {
  const artifacts: StoreArtifact[] = [];
  async function visit(path: string): Promise<void> {
    const metadata = await lstat(path);
    artifacts.push({
      directory: metadata.isDirectory(),
      mode: metadata.mode & 0o777,
      path: relative(root, path),
    });
    if (!metadata.isDirectory()) return;
    const entries = await readdir(path);
    for (const entry of entries) await visit(join(path, entry));
  }
  await visit(root);
  return artifacts;
}

async function readJson(path: string): Promise<Readonly<Record<string, unknown>> | undefined> {
  try {
    const parsed: unknown = JSON.parse(await readFile(path, "utf8"));
    return isRecord(parsed) ? parsed : undefined;
  } catch {
    return undefined;
  }
}

function records(value: unknown): readonly Readonly<Record<string, unknown>>[] | undefined {
  return Array.isArray(value) && value.every(isRecord) ? value : undefined;
}

function confinedExistingYaml(
  store: string,
  path: string,
  yamlPaths: ReadonlySet<string>,
): boolean {
  const relativePath = relative(resolve(store), resolve(store, path));
  if (relativePath.startsWith("..") || isAbsolute(relativePath) || !path.endsWith(".yaml")) {
    return false;
  }
  return yamlPaths.has(path);
}

function freshSpecificCacheEntry(
  entry: Readonly<Record<string, unknown>>,
  indexedIds: ReadonlySet<string>,
): boolean {
  if (
    typeof entry.entry_id !== "string" ||
    !indexedIds.has(entry.entry_id) ||
    entry.verdict !== "valid" ||
    typeof entry.validated_at !== "string"
  ) {
    return false;
  }
  const age = Date.now() - Date.parse(entry.validated_at);
  return Number.isFinite(age) && age >= 0 && age < 48 * 60 * 60 * 1000;
}

export {
  extractProposal,
  proposalStatesAcceptedRecovery,
  storedProposalEntryId,
  validateAdapterInstallation,
  validateProposalWithRuntime,
  validateStoredProposal,
  validateStoreArtifacts,
};
export type { ProposalValidation };
export type { RecoveryRelation };
