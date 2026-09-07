import { readFile } from "node:fs/promises";

const registry: unknown = JSON.parse(
  await readFile("harness/invariants/registry.json", "utf8"),
);
if (typeof registry !== "object" || registry === null) {
  throw new TypeError("fixture-registry-missing");
}
const invariants: unknown = Reflect.get(registry, "invariants");
if (!Array.isArray(invariants) || invariants.length !== 1) {
  throw new TypeError("fixture-invariant-missing");
}
const record: unknown = invariants[0];
if (typeof record !== "object" || record === null) {
  throw new TypeError("fixture-invariant-invalid");
}
const ablation: unknown = Reflect.get(record, "marginalAblation");
if (typeof ablation !== "object" || ablation === null) {
  throw new TypeError("fixture-ablation-missing");
}
const candidateText: unknown = Reflect.get(ablation, "candidateTextExact");
if (typeof candidateText !== "string") {
  throw new TypeError("fixture-candidate-text-missing");
}
const surface = await readFile("harness/AGENTS.md", "utf8");
const lifecycle: unknown = Reflect.get(record, "lifecycle");

if (surface.includes(candidateText) !== (lifecycle === "active")) {
  throw new Error("fixture registry lifecycle disagrees with its surface text");
}

const workflowStateValidated = true;

export { workflowStateValidated };
