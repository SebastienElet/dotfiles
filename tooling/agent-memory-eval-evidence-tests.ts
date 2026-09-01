import { afterEach, describe, expect, test } from "bun:test";
import { chmod, mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

import {
  extractProposal,
  proposalStatesAcceptedRecovery,
  validateAdapterInstallation,
  validateProposalWithRuntime,
  validateStoredProposal,
  validateStoreArtifacts,
} from "./agent-memory-eval-evidence.ts";
import { treeDigest } from "./agent-memory-eval-fixture.ts";
import { installAdapter } from "./agent-memory-eval-auth.ts";
import { auditHasInvalidatedEntry } from "./agent-memory-eval-phase-evidence.ts";

const roots: string[] = [];

afterEach(async () => {
  await Promise.all(roots.splice(0).map((root) => rm(root, { force: true, recursive: true })));
});

describe("agent memory proposal evidence", () => {
  test("requires the complete accepted recovery relation in the statement", () => {
    const relation = {
      profile: "profile-saffron-7",
      seed: "seed-ember-91",
      window: "window-mica-37",
    };
    expect(
      proposalStatesAcceptedRecovery(
        'statement: "window-mica-37 with seed-ember-91 and profile-saffron-7 was accepted"',
        relation,
      ),
    ).toBe(true);
    for (const statement of [
      "window-mica-37 with seed-ash-13 and profile-saffron-7 was accepted",
      "window-mica-37 with seed-ember-91 and profile-saffron-7 was rejected",
      "window-mica-37 with seed-ember-91 and profile-saffron-7 was not accepted",
    ]) {
      expect(() => proposalStatesAcceptedRecovery(`statement: ${JSON.stringify(statement)}`, relation)).toThrow();
    }
  });

  test("extracts one YAML proposal without relying on prose substrings", () => {
    const yaml = "schema_version: 1\nstatement: fixture proposal invariant\n";
    expect(extractProposal(`\`\`\`yaml\n${yaml}\`\`\``)).toBe(yaml.trim());
    expect(() => extractProposal("schema_version appears in prose")).toThrow();
    expect(() => extractProposal(`\`\`\`yaml\n${yaml}\`\`\`\n\`\`\`yaml\n${yaml}\`\`\``)).toThrow();
  });

  test("admits the proposal only in a distinct validation store", async () => {
    const root = await mkdtemp(join(tmpdir(), "agent-memory-proposal-test-"));
    roots.push(root);
    const evaluatedStore = join(root, "evaluated");
    const validationStore = join(root, "validation");
    const repository = join(root, "repository");
    await Promise.all(
      [evaluatedStore, validationStore, repository].map((path) =>
        mkdir(path, { recursive: true, mode: 0o700 }),
      ),
    );
    await writeFile(join(evaluatedStore, "sentinel"), "unchanged");
    const runtime = join(root, "runtime");
    await writeFile(
      runtime,
      `#!/usr/bin/env bun\nconst input = await Bun.stdin.text(); if (!input.includes("fixture proposal invariant")) process.exit(2); await Bun.write(process.env.AGENT_MEMORY_ROOT + "/stored.yaml", input); console.log('{"status":"stored"}');\n`,
    );
    await chmod(runtime, 0o700);
    const result = await validateProposalWithRuntime({
      environment: { PATH: process.env.PATH },
      evaluatedStore,
      expectedRelation: {
        profile: "fixture",
        seed: "fixture",
        window: "fixture",
      },
      proposal: "statement: fixture proposal invariant accepted",
      repository,
      runtime,
      validationStore,
    });
    expect(result).toEqual({
      evaluatedStoreUnchanged: true,
      statementDetected: true,
      stored: true,
    });
  });

  test("full tree digests detect content and permission mutations", async () => {
    const root = await mkdtemp(join(tmpdir(), "agent-memory-digest-test-"));
    roots.push(root);
    const path = join(root, "entry.yaml");
    await writeFile(path, "statement: fixture\n", { mode: 0o600 });
    const before = await treeDigest(root);
    await chmod(path, 0o640);
    expect(await treeDigest(root)).not.toBe(before);
  });

  test("validates private YAML, referential index, and fresh specific cache", async () => {
    const root = await mkdtemp(join(tmpdir(), "agent-memory-layout-test-"));
    roots.push(root);
    const entries = join(root, "entries", "user");
    await mkdir(entries, { recursive: true, mode: 0o700 });
    await writeFile(join(entries, "mem_fixture.yaml"), "schema_version: 1\n", { mode: 0o600 });
    await writeFile(
      join(root, "index.json"),
      JSON.stringify({
        schema_version: 2,
        entries: [{ id: "mem_fixture", path: "entries/user/mem_fixture.yaml" }],
      }),
      { mode: 0o600 },
    );
    await writeFile(
      join(root, "oracle-cache.json"),
      JSON.stringify({
        schema_version: 1,
        entries: [
          { entry_id: "mem_fixture", validated_at: new Date().toISOString(), verdict: "valid" },
        ],
      }),
      { mode: 0o600 },
    );
    expect(await validateStoreArtifacts(root)).toBe(true);
    await writeFile(
      join(root, "index.json"),
      JSON.stringify({ schema_version: 2, entries: [{ id: "missing", path: "missing.yaml" }] }),
      { mode: 0o600 },
    );
    expect(await validateStoreArtifacts(root)).toBe(false);
  });

  test("binds the stored entry identity to the accepted proposal", async () => {
    const root = await mkdtemp(join(tmpdir(), "agent-memory-stored-proposal-test-"));
    roots.push(root);
    const entries = join(root, "entries", "user");
    await mkdir(entries, { recursive: true, mode: 0o700 });
    await writeFile(
      join(entries, "mem_candidate.yaml"),
      'id: mem_candidate\nstatus: active\nstatement: "accepted relation"\n',
      { mode: 0o600 },
    );
    await writeFile(
      join(root, "index.json"),
      JSON.stringify({
        schema_version: 2,
        entries: [{ id: "mem_candidate", path: "entries/user/mem_candidate.yaml" }],
      }),
      { mode: 0o600 },
    );
    expect(
      await validateStoredProposal(root, "mem_candidate", 'statement: "accepted relation"'),
    ).toBe(true);
    await writeFile(
      join(entries, "mem_candidate.yaml"),
      'id: mem_candidate\nstatus: active\nscope:\n  type: project\n  key: project_fixture\nproof:\n  summary: evidence\n  sources:\n    - locator: proof.txt\n      kind: git-file\nstatement: "accepted relation"\n',
      { mode: 0o600 },
    );
    expect(
      await validateStoredProposal(
        root,
        "mem_candidate",
        'statement: "accepted relation"\nscope: project\nproof:\n  sources:\n    - kind: git-file\n      locator: proof.txt\n  summary: evidence',
      ),
    ).toBe(true);
    expect(
      await validateStoredProposal(root, "mem_unrelated", 'statement: "accepted relation"'),
    ).toBe(false);
    expect(
      await validateStoredProposal(root, "mem_candidate", 'statement: "different relation"'),
    ).toBe(false);
  });

  test("validates the exact private adapter config and copied runtime", async () => {
    const root = await mkdtemp(join(tmpdir(), "agent-memory-adapter-test-"));
    roots.push(root);
    const home = join(root, "home");
    const runtimeSource = join(root, "runtime-source");
    const runtime = join(root, "runtime");
    await mkdir(home, { mode: 0o700 });
    await writeFile(runtimeSource, "runtime", { mode: 0o700 });
    await writeFile(runtime, "runtime", { mode: 0o700 });
    await installAdapter("codex", home, runtime);
    expect(
      await validateAdapterInstallation({ agent: "codex", home, runtime, runtimeSource }),
    ).toBe(true);
    const config = join(home, ".codex", "hooks.json");
    await writeFile(config, '{"hooks":{}}\n', { mode: 0o600 });
    expect(
      await validateAdapterInstallation({ agent: "codex", home, runtime, runtimeSource }),
    ).toBe(false);
  });

  test("requires invalidation of the admitted entry identity", () => {
    const audit = {
      entries: [
        { id: "mem_unrelated", status: "invalidated" },
        { id: "mem_candidate", status: "valid" },
      ],
    };
    expect(auditHasInvalidatedEntry(audit, "mem_candidate")).toBe(false);
    expect(
      auditHasInvalidatedEntry(
        { entries: [{ id: "mem_candidate", status: "invalidated" }] },
        "mem_candidate",
      ),
    ).toBe(true);
  });
});
