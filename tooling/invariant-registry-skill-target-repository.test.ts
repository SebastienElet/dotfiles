import { afterEach, expect, test } from "bun:test";
import {
  cleanupFixtures,
  conditionalRegistryText,
  initializeFixture,
} from "./invariant-registry-skill-target-repository-test-support.ts";
import { validateInvariantRegistryText } from "./invariant-registry-repository-validator.ts";

afterEach(cleanupFixtures);

test.each([
  [
    "missing target",
    { target: "missing" },
    "Conditional skill target does not exist",
  ],
  [
    "untracked target",
    { tracked: false },
    "Conditional skill target must be tracked by Git",
  ],
  [
    "symlink target",
    { target: "symlink" },
    "Conditional skill target must be a regular file",
  ],
  [
    "invalid name",
    { name: "Different Skill" },
    "Conditional skill target frontmatter is invalid",
  ],
  [
    "invalid category",
    { category: "unknown" },
    "Conditional skill target frontmatter is invalid",
  ],
  [
    "non-triggerable description",
    { description: "A reusable helper." },
    "Conditional skill description must support implicit discovery",
  ],
] as const)("rejects %s", async (_name, options, expected) => {
  const root = await initializeFixture(options);
  expect(() =>
    validateInvariantRegistryText(conditionalRegistryText(), root),
  ).toThrow(expected);
});
