import {
  type HarnessReflectionContract,
  harnessReflectionContractSchema,
} from "./harness-reflection-contract-schema.ts";

const contractBlockPattern = /```json\n(?<json>[\s\S]*?)\n```/u;
const insertionKey = "__duplicate_contract_key__";
const jsonIndent = 2;
type DuplicateKeyMutation = Readonly<{
  key: string;
  path: readonly string[];
  shadowedValue: unknown;
}>;

const isRecord = (value: unknown): value is Readonly<Record<string, unknown>> =>
  typeof value === "object" && value !== null && !Array.isArray(value);

const parseContract = (reference: string): HarnessReflectionContract => {
  const source = contractBlockPattern.exec(reference)?.groups?.json;
  if (source === undefined) {
    throw new Error("authoritative contract block missing");
  }
  const parsed: unknown = JSON.parse(source);
  return harnessReflectionContractSchema.parse(parsed);
};

const renderContract = (reference: string, contract: unknown): string =>
  reference.replace(
    contractBlockPattern,
    `\`\`\`json\n${JSON.stringify(contract, null, jsonIndent)}\n\`\`\``,
  );

const recordAtPath = (
  contract: unknown,
  path: readonly string[],
): Readonly<Record<string, unknown>> => {
  let current: unknown = contract;
  for (const segment of path) {
    if (!isRecord(current)) {
      throw new TypeError(`contract path is not an object: ${path.join(".")}`);
    }
    current = Reflect.get(current, segment);
  }
  if (!isRecord(current)) {
    throw new TypeError(`contract path is not an object: ${path.join(".")}`);
  }
  return current;
};

const mutateContract = (
  reference: string,
  path: readonly string[],
  mutate: (target: Readonly<Record<string, unknown>>) => void,
): string => {
  const contract = parseContract(reference);
  mutate(recordAtPath(contract, path));
  return renderContract(reference, contract);
};

const duplicateContractKey = (
  reference: string,
  mutation: DuplicateKeyMutation,
): string => {
  const contract = parseContract(reference);
  const target = recordAtPath(contract, mutation.path);
  const entries = Object.entries(target);
  for (const entryKey of Object.keys(target)) {
    Reflect.deleteProperty(target, entryKey);
  }
  Reflect.set(target, insertionKey, null);
  for (const [entryKey, value] of entries) {
    Reflect.set(target, entryKey, value);
  }
  const rendered = renderContract(reference, contract);
  return rendered.replace(
    `${JSON.stringify(insertionKey)}: null`,
    `${JSON.stringify(mutation.key)}: ${JSON.stringify(mutation.shadowedValue)}`,
  );
};

export { duplicateContractKey, mutateContract };
