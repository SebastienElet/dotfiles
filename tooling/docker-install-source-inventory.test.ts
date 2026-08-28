import { expect, test } from "bun:test";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { checkSoftwareSources } from "./check-software-sources.ts";
import { inventorySources } from "./software-source-inventory.ts";
import { join } from "node:path";
import { tmpdir } from "node:os";

test("inventories an image installed through the Docker artifact entry point", () => {
  const command =
    '"/repo/tooling/install-docker-artifact" install scrapling "allow-skip" "registry.example.test/scrapling:1"';

  expect(inventorySources(command)).toEqual([
    "channel:docker",
    "docker:registry.example.test/scrapling:1",
  ]);
});

test.each([
  '"/repo/tooling/install-docker-artifact" install scrapling "allow-skip"',
  '"/repo/tooling/install-docker-artifact" install unknown "allow-skip" "image"',
  '"/repo/tooling/install-docker-artifact" install scrapling "allow-skip" "image" extra',
])("refuses unsupported Docker artifact entry point syntax", (command) => {
  expect(inventorySources(command)).toContain(
    "docker-installer:unsupported-syntax",
  );
});

test("the complete gate rejects an undeclared image through a relative entry point", () => {
  const root = mkdtempSync(join(tmpdir(), "docker-install-gate-"));
  const makefile = join(root, "Makefile");
  writeFileSync(
    makefile,
    [
      "all:",
      "\t@brew install example",
      '\t@tooling/install-docker-artifact install scrapling "allow-skip" "evil.example/payload:latest"',
      "",
    ].join("\n"),
  );

  try {
    expect(checkSoftwareSources(makefile)).toContain(
      "docker:evil.example/payload:latest",
    );
  } finally {
    rmSync(root, { force: true, recursive: true });
  }
});
