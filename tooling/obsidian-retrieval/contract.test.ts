import { describe, expect, test } from "bun:test";
import { mkdtemp, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { z } from "zod";
import {
  loadContractSources,
  validateContract,
  type ContractSources,
} from "./contract.ts";

const repositoryRoot = resolve(import.meta.dir, "../..");

const mutate = (
  sources: ContractSources,
  field: "skill" | "reference",
  transform: (value: string) => string,
): ContractSources => ({ ...sources, [field]: transform(sources[field]) });

describe("Obsidian retrieval contract", () => {
  test("declares the pinned Bun version", async () => {
    const packageDocument = z
      .object({ packageManager: z.string() })
      .parse(await Bun.file(join(repositoryRoot, "package.json")).json());

    expect(packageDocument.packageManager).toBe("bun@1.4.0");
  });

  test("accepts the repository skill", async () => {
    const sources = await loadContractSources(repositoryRoot);

    expect(validateContract(sources)).toEqual([]);
  });

  test.each([
    [
      "adds a mutator to the allowlist",
      (value: string) => value.replace("`read`,", "`read`, `create`,"),
    ],
    [
      "duplicates an allowed command",
      (value: string) => value.replace("`read`,", "`read`, `read`,"),
    ],
    [
      "adds a second allowlist",
      (value: string) => `${value}\n## Read-only allowlist\n\n\`vaults\`\n`,
    ],
    [
      "exposes a backticked mutator",
      (value: string) => `${value}\nAllowed command: \`eval\`.\n`,
    ],
    [
      "invokes a mutator",
      (value: string) => `${value}\nRun obsidian property:set.\n`,
    ],
  ])("rejects a reference that %s", async (_name, transform) => {
    const sources = await loadContractSources(repositoryRoot);

    expect(
      validateContract(mutate(sources, "reference", transform)),
    ).not.toEqual([]);
  });

  test.each([
    "Use create to add a note.",
    "Use\ncreate to add a note.",
    "Use\neval to inspect the vault.",
    "Use\nopen to display the active note.",
    "Invoke task to update a checkbox.",
  ])("rejects positive dangerous guidance: %s", async (guidance) => {
    const sources = await loadContractSources(repositoryRoot);

    expect(
      validateContract(
        mutate(sources, "skill", (value) => `${value}\n\n${guidance}\n`),
      ),
    ).not.toEqual([]);
  });

  test("rejects positive dangerous guidance in the reference", async () => {
    const sources = await loadContractSources(repositoryRoot);

    expect(
      validateContract(
        mutate(
          sources,
          "reference",
          (value) => `${value}\n\nUse create to add a note.\n`,
        ),
      ),
    ).not.toEqual([]);
  });

  test.each([
    "Must not create notes.",
    "Refuse to delete notes.",
    "Write the answer with citations.",
  ])("accepts non-mutating guidance: %s", async (guidance) => {
    const sources = await loadContractSources(repositoryRoot);

    expect(
      validateContract(
        mutate(sources, "skill", (value) => `${value}\n\n${guidance}\n`),
      ),
    ).toEqual([]);
  });

  test("rejects malformed evaluation data", async () => {
    const sources = await loadContractSources(repositoryRoot);

    expect(
      validateContract({
        ...sources,
        evaluations: { skill: "obsidian-retrieval" },
      }),
    ).not.toEqual([]);
  });

  test("rejects unknown evaluation fields", async () => {
    const sources = await loadContractSources(repositoryRoot);
    const evaluations = JSON.parse(
      JSON.stringify(sources.evaluations).replace(
        '"reason":',
        '"unexpected":true,"reason":',
      ),
    ) as unknown;

    expect(validateContract({ ...sources, evaluations })).not.toEqual([]);
  });

  test("runs the real command entry point", async () => {
    const process = Bun.spawn(
      [Bun.which("bun") ?? "bun", "run", join(import.meta.dir, "contract.ts")],
      {
        cwd: repositoryRoot,
        stderr: "pipe",
        stdout: "pipe",
      },
    );

    expect(await process.exited).toBe(0);
    expect(await new Response(process.stdout).text()).toContain(
      "Obsidian retrieval contract passed",
    );
  });

  test("the command fails closed when an input is missing", async () => {
    const root = await mkdtemp(join(tmpdir(), "obsidian-contract-"));
    await writeFile(join(root, "SKILL.md"), "incomplete", "utf8");
    const process = Bun.spawn(
      [
        Bun.which("bun") ?? "bun",
        "run",
        join(import.meta.dir, "contract.ts"),
        root,
      ],
      { stderr: "pipe", stdout: "pipe" },
    );

    expect(await process.exited).not.toBe(0);
    expect(await new Response(process.stderr).text()).toContain(
      "unable to read contract inputs",
    );
  });

  test("the command rejects an empty repository root", async () => {
    const process = Bun.spawn(
      [
        Bun.which("bun") ?? "bun",
        "run",
        join(import.meta.dir, "contract.ts"),
        "",
      ],
      { stderr: "pipe", stdout: "pipe" },
    );

    expect(await process.exited).not.toBe(0);
    expect(await new Response(process.stderr).text()).toContain(
      "repository root",
    );
  });
});
