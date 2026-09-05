import type { Observation, Oracle } from "./contracts.ts";

function evaluate(
  oracle: Oracle,
  observations: readonly Observation[],
): "PASS" | "FAIL" {
  const readIndex = observations.findIndex(
    (event) =>
      event.tool === "cat" &&
      event.exitCode === 0 &&
      event.args.includes(".agents/skills/code-search/SKILL.md"),
  );
  const conceptualIndex = observations.findIndex(
    (event) => event.tool === "colgrep-search" && event.exitCode === 0,
  );
  if (oracle === "structural-v1") {
    return readIndex !== -1 && conceptualIndex > readIndex ? "PASS" : "FAIL";
  }
  if (observations.some((event) => event.tool === "colgrep-search")) {
    return "FAIL";
  }
  if (oracle === "literal-v1") {
    return observations.some(
      (event) =>
        event.tool === "rg" &&
        event.exitCode === 0 &&
        event.args.includes("FEATURE_FLAG_DISABLED"),
    )
      ? "PASS"
      : "FAIL";
  }
  if (
    observations.some((event) => event.tool === "rg" || event.tool === "fd")
  ) {
    return "FAIL";
  }
  return observations.some(
    (event) =>
      event.tool === "cat" &&
      event.exitCode === 0 &&
      event.args.includes("src/auth/session.ts"),
  )
    ? "PASS"
    : "FAIL";
}

export { evaluate };
