import { afterEach, describe, expect, test } from "bun:test";
import { mkdir, mkdtemp, readFile, rm, stat, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

import {
  assertCandidateIdentity,
  evaluatorCandidateSha,
  mergeReceipt,
  renderNormalizedReport,
} from "./agent-memory-eval-report.ts";
import type { AgentReceipt } from "./agent-memory-eval-report.ts";

const roots: string[] = [];

afterEach(async () => {
  await Promise.all(roots.splice(0).map((root) => rm(root, { force: true, recursive: true })));
});

describe("agent memory evaluation receipts", () => {
  test("merges agents on the same candidate in a private receipt", async () => {
    const root = await mkdtemp(join(tmpdir(), "agent-memory-receipt-test-"));
    roots.push(root);
    const path = join(root, "receipt.json");
    await mergeReceipt(path, "candidate", "runtime", agent("codex", "complete", 3));
    const receipt = await mergeReceipt(path, "candidate", "runtime", agent("claude", "complete", 3));
    expect(Object.keys(receipt.agents).sort()).toEqual(["claude", "codex"]);
    expect((await stat(path)).mode & 0o777).toBe(0o600);
    expect(JSON.parse(await readFile(path, "utf8"))).toEqual(receipt);
  });

  test("records a blocking failure without overclaiming capabilities", async () => {
    const root = await mkdtemp(join(tmpdir(), "agent-memory-receipt-test-"));
    roots.push(root);
    const path = join(root, "receipt.json");
    const receipt = await mergeReceipt(path, "candidate", "runtime", {
      ...agent("cursor", "blocked", 0),
      errorClass: "usage_limit",
    });
    const report = renderNormalizedReport(receipt);
    expect(report).toContain("Cursor | fixture-version | blocked | 0/3 | usage_limit | complete");
    expect(report).not.toContain("Cursor | complete");
    expect(report).toContain(
      "`bun tooling/agent-memory-eval.ts --agent cursor --replicates 3`",
    );
    expect(report).toContain("Cleanup is recorded only after fixture removal is verified");
    await expect(
      mergeReceipt(path, "different", "runtime", agent("codex", "complete", 3)),
    ).rejects.toThrow("candidate");
  });

  test("binds the candidate to evaluator and runtime source bytes", async () => {
    const root = await mkdtemp(join(tmpdir(), "agent-memory-candidate-test-"));
    roots.push(root);
    await mkdir(join(root, "tooling/agent-memory/src"), { recursive: true });
    await mkdir(join(root, "harness/skills/memory-governance/references"), { recursive: true });
    await mkdir(join(root, "harness/rules"), { recursive: true });
    await writeFile(join(root, "tooling/agent-memory-eval.ts"), "evaluator-a");
    await writeFile(join(root, "tooling/agent-memory/src/lib.rs"), "runtime-a");
    await writeFile(join(root, "harness/skills/memory-governance/SKILL.md"), "skill-a");
    await writeFile(
      join(root, "harness/skills/memory-governance/references/entry-contract.md"),
      "contract-a",
    );
    await writeFile(join(root, "harness/rules/memory-governance-cursor.mdc"), "rule-a");
    const initial = await evaluatorCandidateSha(root);
    await writeFile(join(root, "tooling/agent-memory-eval.ts"), "evaluator-b");
    const evaluatorChanged = await evaluatorCandidateSha(root);
    await writeFile(join(root, "tooling/agent-memory/src/lib.rs"), "runtime-b");
    const runtimeChanged = await evaluatorCandidateSha(root);
    await writeFile(join(root, "harness/skills/memory-governance/SKILL.md"), "skill-b");
    const skillChanged = await evaluatorCandidateSha(root);
    await writeFile(join(root, "harness/rules/memory-governance-cursor.mdc"), "rule-b");
    const ruleChanged = await evaluatorCandidateSha(root);
    expect(evaluatorChanged).not.toBe(initial);
    expect(runtimeChanged).not.toBe(evaluatorChanged);
    expect(skillChanged).not.toBe(runtimeChanged);
    expect(ruleChanged).not.toBe(skillChanged);
  });

  test("refuses publication when candidate or runtime changes during a run", () => {
    expect(() =>
      assertCandidateIdentity("candidate-a", "runtime-a", "candidate-a", "runtime-a"),
    ).not.toThrow();
    expect(() =>
      assertCandidateIdentity("candidate-a", "runtime-a", "candidate-b", "runtime-a"),
    ).toThrow("changed during evaluation");
    expect(() =>
      assertCandidateIdentity("candidate-a", "runtime-a", "candidate-a", "runtime-b"),
    ).toThrow("changed during evaluation");
  });
});

function agent(
  agentName: AgentReceipt["agent"],
  status: AgentReceipt["status"],
  completedReplicates: number,
): AgentReceipt {
  return {
    agent: agentName,
    capabilities: completedReplicates === 3 ? { fresh_retrieval: 3 } : {},
    completedReplicates,
    cleanup: "complete",
    requestedReplicates: 3,
    status,
    version: "fixture-version",
  };
}
