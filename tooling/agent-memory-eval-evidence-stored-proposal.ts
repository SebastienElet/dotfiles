import { join, relative, resolve } from "node:path";

import {
  readJson,
  records,
} from "./agent-memory-eval-evidence-store-artifacts.ts";
import { isRecord } from "./agent-memory-eval-evidence-types.ts";
import { readFile } from "node:fs/promises";

async function validateStoredProposal(
  store: string,
  entryId: string,
  proposal: string,
): Promise<boolean> {
  const index = await readJson(join(store, "index.json"));
  const row = records(index?.entries)?.find((entry) => entry.id === entryId);
  if (typeof row?.path !== "string") {
    return false;
  }
  const path = resolve(store, row.path);
  if (relative(resolve(store), path).startsWith("..")) {
    return false;
  }
  const [stored, expected] = await Promise.all([
    readFile(path, "utf8").then((value) => Bun.YAML.parse(value)),
    Promise.resolve(Bun.YAML.parse(proposal)),
  ]);
  if (!isRecord(stored) || !isRecord(expected) || stored.id !== entryId) {
    return false;
  }
  return storedProposalMatchesExpected(stored, expected);
}

function storedProposalMatchesExpected(
  stored: Readonly<Record<string, unknown>>,
  expected: Readonly<Record<string, unknown>>,
): boolean {
  for (const [key, value] of Object.entries(expected)) {
    if (!equalStoredField(key, stored[key], value)) {
      return false;
    }
  }

  return true;
}

async function storedProposalEntryId(
  store: string,
  proposal: string,
): Promise<string> {
  const index = await readJson(join(store, "index.json"));
  const rows = records(index?.entries) ?? [];
  const matches = await matchingProposalIds(store, proposal, rows);
  if (matches.length !== 1) {
    const mismatch = await storedProposalMismatch(store, proposal, rows);
    throw new Error(
      `store lacks one exact admitted proposal${mismatch.length === 0 ? "" : `: ${mismatch.join(",")}`}`,
    );
  }
  return matches[0] ?? "";
}

async function matchingProposalIds(
  store: string,
  proposal: string,
  rows: readonly Readonly<Record<string, unknown>>[],
): Promise<string[]> {
  const matches: string[] = [];
  for (const row of rows) {
    if (
      typeof row.id === "string" &&
      (await validateStoredProposal(store, row.id, proposal))
    ) {
      matches.push(row.id);
    }
  }
  return matches;
}

function storedProposalMismatch(
  store: string,
  proposal: string,
  rows: readonly Readonly<Record<string, unknown>>[],
): Promise<string[]> {
  if (rows.length !== 1 || typeof rows[0]?.path !== "string") {
    return Promise.resolve([]);
  }
  return storedProposalMismatchKeys(store, rows[0].path, proposal);
}

async function storedProposalMismatchKeys(
  store: string,
  path: string,
  proposal: string,
): Promise<string[]> {
  const stored: unknown = Bun.YAML.parse(
    await readFile(resolve(store, path), "utf8"),
  );
  const expected: unknown = Bun.YAML.parse(proposal);
  if (!isRecord(stored) || !isRecord(expected)) {
    return ["document"];
  }
  return storedProposalMismatchKeysFromExpected(stored, expected);
}

function storedProposalMismatchKeysFromExpected(
  stored: Readonly<Record<string, unknown>>,
  expected: Readonly<Record<string, unknown>>,
): string[] {
  const mismatchedKeys: string[] = [];

  for (const [key, value] of Object.entries(expected)) {
    if (!equalStoredField(key, stored[key], value)) {
      mismatchedKeys.push(key);
    }
  }

  return mismatchedKeys;
}

function equalStoredField(
  key: string,
  stored: unknown,
  expected: unknown,
): boolean {
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
    if (!isRecord(left) || !isRecord(right)) {
      return false;
    }
    return Object.keys(right).every(
      (key) => key in left && equalValue(left[key], right[key]),
    );
  }
  return left === right;
}

export { storedProposalEntryId, validateStoredProposal };
