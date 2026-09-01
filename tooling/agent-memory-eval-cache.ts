import { readFile } from "node:fs/promises";
import { dirname, join } from "node:path";

async function cacheContainsFixture(path: string, expectedTerms: string): Promise<boolean> {
  try {
    const cache: unknown = JSON.parse(await readFile(path, "utf8"));
    const index: unknown = JSON.parse(await readFile(join(dirname(path), "index.json"), "utf8"));
    if (!isRecord(cache) || cache.schema_version !== 1 || !isRecord(index)) return false;
    if (!Array.isArray(cache.entries) || !Array.isArray(index.entries)) return false;
    const expectedIds = new Set(
      index.entries
        .filter(isRecord)
        .filter((entry) => indexEntryMatches(entry, expectedTerms))
        .map((entry) => entry.id),
    );
    return cache.entries
      .filter(isRecord)
      .some((entry) => entry.verdict === "valid" && expectedIds.has(entry.entry_id));
  } catch {
    return false;
  }
}

function indexEntryMatches(
  entry: Readonly<Record<string, unknown>>,
  expectedTerms: string,
): boolean {
  if (typeof entry.id !== "string") return false;
  const phrase = expectedTerms.toLowerCase();
  const retrievalTerms = Array.isArray(entry.retrieval_terms)
    ? entry.retrieval_terms.filter((value): value is string => typeof value === "string")
    : [];
  const statementTokens = Array.isArray(entry.statement_tokens)
    ? entry.statement_tokens.filter((value): value is string => typeof value === "string")
    : [];
  return (
    retrievalTerms.some((term) => term.toLowerCase().includes(phrase)) ||
    phrase.split(/\s+/u).every((term) => statementTokens.includes(term))
  );
}

function isRecord(value: unknown): value is Readonly<Record<string, unknown>> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

export { cacheContainsFixture };
