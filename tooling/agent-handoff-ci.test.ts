import { expect, test } from "bun:test";
import { resolve } from "node:path";
import { z } from "zod";

const repositoryRoot = resolve(import.meta.dir, "..");
const workflowPath = resolve(
  repositoryRoot,
  ".github/workflows/test-agent-handoff.yml",
);
const pathFilters = [
  ".cargo/**",
  "tooling/agent-handoff/**",
  "Makefile",
  "tooling/deployment-codex-wiring.test.ts",
  "tooling/arnes/**",
  ".github/workflows/test-agent-handoff.yml",
] as const;
const pathFilterSchema = z.tuple([
  z.literal(pathFilters[0]),
  z.literal(pathFilters[1]),
  z.literal(pathFilters[2]),
  z.literal(pathFilters[3]),
  z.literal(pathFilters[4]),
  z.literal(pathFilters[5]),
]);
const triggerSchema = z.object({ paths: pathFilterSchema }).strict();
const agentHandoffJob = z
  .object({
    strategy: z
      .object({
        matrix: z
          .object({
            os: z.tuple([
              z.literal("macos-latest"),
              z.literal("ubuntu-latest"),
            ]),
          })
          .strict(),
      })
      .strict(),
    "runs-on": z.literal(`\${{ matrix.os }}`),
    env: z.object({ RUSTFLAGS: z.literal("-Funsafe-code") }).strict(),
    steps: z.tuple([
      z.object({ uses: z.literal("actions/checkout@v5") }).strict(),
      z
        .object({
          run: z.literal(
            "cargo fmt --manifest-path tooling/agent-handoff/Cargo.toml --check",
          ),
        })
        .strict(),
      z
        .object({
          run: z.literal(
            "cargo clippy --manifest-path tooling/agent-handoff/Cargo.toml --all-targets -- -D warnings",
          ),
        })
        .strict(),
      z
        .object({
          run: z.literal(
            "cargo test --manifest-path tooling/agent-handoff/Cargo.toml",
          ),
        })
        .strict(),
    ]),
  })
  .strict();
const workflowSchema = z
  .object({
    name: z.literal("Agent Handoff tests"),
    on: z
      .object({
        push: triggerSchema,
        pull_request: triggerSchema,
      })
      .strict(),
    jobs: z.object({ "agent-handoff": agentHandoffJob }).strict(),
  })
  .strict();

test("runs every agent-handoff Rust oracle on macOS and Linux", async () => {
  const workflow = await Bun.file(workflowPath).text();

  expect(workflowSchema.safeParse(Bun.YAML.parse(workflow)).success).toBeTrue();
});

const pullRequestPathFilterOccurrence = 2;
const mutationCases = [
  [
    "missing Linux",
    (workflow: string): string =>
      workflow.replace(
        "os: [macos-latest, ubuntu-latest]",
        "os: [macos-latest]",
      ),
  ],
  [
    "missing macOS",
    (workflow: string): string =>
      workflow.replace(
        "os: [macos-latest, ubuntu-latest]",
        "os: [ubuntu-latest]",
      ),
  ],
  [
    "missing checkout",
    (workflow: string): string =>
      workflow.replace("      - uses: actions/checkout@v5\n", ""),
  ],
  [
    "missing format check",
    (workflow: string): string =>
      workflow.replace(
        "      - run: cargo fmt --manifest-path tooling/agent-handoff/Cargo.toml --check\n",
        "",
      ),
  ],
  [
    "missing Clippy",
    (workflow: string): string =>
      workflow.replace(
        "      - run: cargo clippy --manifest-path tooling/agent-handoff/Cargo.toml --all-targets -- -D warnings\n",
        "",
      ),
  ],
  [
    "missing tests",
    (workflow: string): string =>
      workflow.replace(
        "      - run: cargo test --manifest-path tooling/agent-handoff/Cargo.toml\n",
        "",
      ),
  ],
  [
    "continue-on-error",
    (workflow: string): string =>
      workflow.replace(
        "  agent-handoff:\n",
        "  agent-handoff:\n    continue-on-error: true\n",
      ),
  ],
  [
    "a permissive condition",
    (workflow: string): string =>
      workflow.replace(
        "  agent-handoff:\n",
        "  agent-handoff:\n    if: false\n",
      ),
  ],
  [
    "a successful fallback",
    (workflow: string): string =>
      workflow.replace(
        "cargo test --manifest-path tooling/agent-handoff/Cargo.toml",
        "cargo test --manifest-path tooling/agent-handoff/Cargo.toml || true",
      ),
  ],
  [
    "stderr suppression",
    (workflow: string): string =>
      workflow.replace(
        "cargo clippy --manifest-path tooling/agent-handoff/Cargo.toml --all-targets -- -D warnings",
        "cargo clippy --manifest-path tooling/agent-handoff/Cargo.toml --all-targets -- -D warnings 2>/dev/null",
      ),
  ],
  ...pathFilters.flatMap((pathFilter) => [
    [
      `missing push path filter ${pathFilter}`,
      (workflow: string): string =>
        removeOccurrence(workflow, `      - ${pathFilter}\n`, 1),
    ] as const,
    [
      `missing pull request path filter ${pathFilter}`,
      (workflow: string): string =>
        removeOccurrence(
          workflow,
          `      - ${pathFilter}\n`,
          pullRequestPathFilterOccurrence,
        ),
    ] as const,
  ]),
] satisfies readonly (readonly [string, (workflow: string) => string])[];

test.each(mutationCases)("rejects %s", async (_name, mutateWorkflow) => {
  const workflow = await Bun.file(workflowPath).text();
  const mutation = mutateWorkflow(workflow);

  expect(
    workflowSchema.safeParse(Bun.YAML.parse(mutation)).success,
  ).toBeFalse();
});

function removeOccurrence(
  contents: string,
  needle: string,
  occurrence: number,
): string {
  let seen = 0;
  return contents.replaceAll(needle, (match) => {
    seen += 1;
    return seen === occurrence ? "" : match;
  });
}
