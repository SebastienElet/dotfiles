import { expect, test } from "bun:test";
import { resolve } from "node:path";
import { z } from "zod";

const repositoryRoot = resolve(import.meta.dir, "..");
const workflowPath = resolve(
  repositoryRoot,
  ".github/workflows/test-agent-memory.yml",
);
const typeScriptWorkflowPath = resolve(
  repositoryRoot,
  ".github/workflows/test-typescript.yml",
);
const pathFilters = [
  ".cargo/**",
  "tooling/agent-memory/**",
  "tooling/agent-memory-eval*.ts",
  "tooling/agent-memory-eval-scenarios.json",
  "tooling/deployment-agent-memory.test.ts",
  "tooling/arnes/**",
  "harness/rules/memory-governance-cursor.mdc",
  "harness/skills/memory-governance/**",
  "home/.arnes.yaml",
  "Makefile",
  ".github/workflows/test-agent-memory.yml",
] as const;
const pathFilterSchema = z.tuple([
  z.literal(pathFilters[0]),
  z.literal(pathFilters[1]),
  z.literal(pathFilters[2]),
  z.literal(pathFilters[3]),
  z.literal(pathFilters[4]),
  z.literal(pathFilters[5]),
  z.literal(pathFilters[6]),
  z.literal(pathFilters[7]),
  z.literal(pathFilters[8]),
  z.literal(pathFilters[9]),
  z.literal(pathFilters[10]),
]);
const triggerSchema = z.object({ paths: pathFilterSchema }).strict();
const memoryJobSchema = z
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
            "cargo fmt --manifest-path tooling/agent-memory/Cargo.toml --check",
          ),
        })
        .strict(),
      z
        .object({
          run: z.literal(
            "cargo clippy --manifest-path tooling/agent-memory/Cargo.toml --all-targets -- -D warnings",
          ),
        })
        .strict(),
      z
        .object({
          run: z.literal(
            "cargo test --manifest-path tooling/agent-memory/Cargo.toml",
          ),
        })
        .strict(),
    ]),
  })
  .strict();
const workflowSchema = z
  .object({
    name: z.literal("Agent Memory tests"),
    on: z.object({ push: triggerSchema, pull_request: triggerSchema }).strict(),
    jobs: z.object({ "agent-memory": memoryJobSchema }).strict(),
  })
  .strict();

test("runs every agent-memory Rust oracle on macOS and Linux", async () => {
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
  ...[
    "actions/checkout@v5",
    "cargo fmt --manifest-path tooling/agent-memory/Cargo.toml --check",
    "cargo clippy --manifest-path tooling/agent-memory/Cargo.toml --all-targets -- -D warnings",
    "cargo test --manifest-path tooling/agent-memory/Cargo.toml",
  ].map(
    (step) =>
      [
        `missing ${step}`,
        (workflow: string): string =>
          workflow
            .split("\n")
            .filter((line) => !line.includes(step))
            .join("\n"),
      ] as const,
  ),
  [
    "continue-on-error",
    (workflow: string): string =>
      workflow.replace(
        "  agent-memory:\n",
        "  agent-memory:\n    continue-on-error: true\n",
      ),
  ],
  [
    "successful fallback",
    (workflow: string): string =>
      workflow.replace(
        "cargo test --manifest-path tooling/agent-memory/Cargo.toml",
        "cargo test --manifest-path tooling/agent-memory/Cargo.toml || true",
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

  expect(
    workflowSchema.safeParse(Bun.YAML.parse(mutateWorkflow(workflow))).success,
  ).toBeFalse();
});

test("builds the memory runtime before Bun tests consume it", async () => {
  const workflow = await Bun.file(typeScriptWorkflowPath).text();
  const build = workflow.indexOf(
    "cargo build --release --manifest-path tooling/agent-memory/Cargo.toml",
  );
  const tests = workflow.indexOf(
    "bun --config=/dev/null --no-env-file test --timeout 15000",
  );

  expect(build).toBeGreaterThan(0);
  expect(tests).toBeGreaterThan(build);
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
