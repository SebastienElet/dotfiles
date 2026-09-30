import { afterEach, expect, test } from "bun:test";
import {
  cleanupDeploymentFixtures,
  createDeploymentFixture,
  pathExists,
  runDeploymentHelper,
} from "./deployment-test-support.ts";
import { dirname, join } from "node:path";
import {
  mkdirSync,
  readFileSync,
  readlinkSync,
  realpathSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";

afterEach(cleanupDeploymentFixtures);

function fixture(): ReturnType<typeof createDeploymentFixture> {
  const context = createDeploymentFixture("clean");
  mkdirSync(join(context.repository, "home"));
  writeFileSync(
    join(context.repository, "home/.arnes.yaml"),
    "version: 1\nskills: []\n",
  );
  return context;
}

type Fixture = ReturnType<typeof fixture>;

function clean(
  context: Fixture,
  arguments_: readonly string[] = ["--apply"],
): ReturnType<typeof runDeploymentHelper> {
  return runDeploymentHelper(context, {
    helper: "clean-deployment.ts",
    arguments: [context.repository, context.home, ...arguments_],
  });
}

function managedLink(
  context: Fixture,
  path = ".config/nvim",
  sourcePath = "home/.config/nvim",
): Readonly<{ destination: string; source: string }> {
  const destination = join(context.home, path);
  const source = join(context.repository, sourcePath);
  mkdirSync(dirname(source), { recursive: true });
  writeFileSync(source, "source stays\n");
  mkdirSync(dirname(destination), { recursive: true });
  symlinkSync(source, destination);
  return { destination, source };
}

test("inspects a deployed link by default without changing it or its source", () => {
  const context = fixture();
  const { destination, source } = managedLink(context);
  const result = clean(context, []);
  expect(result.exitCode).toBe(0);
  expect(result.stdout).toContain("would-remove");
  expect(result.stdout).toContain(destination);
  expect(readlinkSync(destination)).toBe(source);
  expect(readFileSync(source, "utf8")).toBe("source stays\n");
});

test("removes only the owned link and succeeds when repeated", () => {
  const context = fixture();
  const { destination, source } = managedLink(context);
  expect(clean(context).exitCode).toBe(0);
  expect(pathExists(destination)).toBeFalse();
  expect(readFileSync(source, "utf8")).toBe("source stays\n");
  expect(clean(context).exitCode).toBe(0);
});

test("removes an owned dangling link", () => {
  const context = fixture();
  const destination = join(context.home, ".config/starship.toml");
  mkdirSync(dirname(destination), { recursive: true });
  symlinkSync(
    join(context.repository, "home/.config/starship.toml"),
    destination,
  );
  expect(clean(context).exitCode).toBe(0);
  expect(pathExists(destination)).toBeFalse();
});

test.each(["file", "directory", "foreign-link"])(
  "preserves a divergent %s at a managed link destination",
  (kind) => {
    const context = fixture();
    const destination = join(context.home, ".config/nvim");
    mkdirSync(dirname(destination), { recursive: true });
    const foreign = join(context.root, "foreign");
    writeFileSync(foreign, "keep\n");
    if (kind === "file") {
      writeFileSync(destination, "keep\n");
    }
    if (kind === "directory") {
      mkdirSync(destination);
    }
    if (kind === "foreign-link") {
      symlinkSync(foreign, destination);
    }
    const result = clean(context);
    expect(result.exitCode).toBe(0);
    expect(result.stdout).toContain("preserved");
    expect(pathExists(destination)).toBeTrue();
    expect(readFileSync(foreign, "utf8")).toBe("keep\n");
  },
);

test("removes an edited generated instruction and copied agent definition while preserving histories", () => {
  const context = fixture();
  const generated = join(context.home, ".codex/AGENTS.md");
  const copied = join(context.home, ".codex/agents/design-claim-auditor.toml");
  const history = join(context.home, ".codex/history.jsonl");
  mkdirSync(dirname(copied), { recursive: true });
  for (const path of [generated, copied, history]) {
    writeFileSync(path, "edited\n");
  }
  expect(clean(context).exitCode).toBe(0);
  expect(pathExists(generated)).toBeFalse();
  expect(pathExists(copied)).toBeFalse();
  expect(readFileSync(history, "utf8")).toBe("edited\n");
});

test("preserves a foreign symlink at a copied destination", () => {
  const context = fixture();
  const destination = join(context.home, ".codex/AGENTS.md");
  const foreign = join(context.root, "foreign");
  mkdirSync(dirname(destination), { recursive: true });
  writeFileSync(foreign, "keep\n");
  symlinkSync(foreign, destination);
  expect(clean(context).exitCode).toBe(0);
  expect(readlinkSync(destination)).toBe(foreign);
  expect(readFileSync(foreign, "utf8")).toBe("keep\n");
});

test("uses manifest user installations to remove managed skills without deleting third party skills", () => {
  const context = fixture();
  writeFileSync(
    join(context.repository, "home/.arnes.yaml"),
    "version: 1\nskills:\n  - slug: example\n    installations:\n      - { agent: codex, scope: user }\n      - { agent: cursor, scope: user }\n",
  );
  const codex = managedLink(
    context,
    ".agents/skills/example",
    "harness/skills/example",
  );
  const cursor = join(context.home, ".cursor/skills/example");
  mkdirSync(dirname(cursor), { recursive: true });
  symlinkSync(codex.source, cursor);
  const thirdParty = join(context.home, ".agents/skills/third-party");
  mkdirSync(thirdParty);
  writeFileSync(join(thirdParty, "SKILL.md"), "keep\n");
  expect(clean(context).exitCode).toBe(0);
  expect(pathExists(codex.destination)).toBeFalse();
  expect(pathExists(cursor)).toBeFalse();
  expect(readFileSync(join(thirdParty, "SKILL.md"), "utf8")).toBe("keep\n");
});

test.each(["outside", "repository"])(
  "refuses a symbolic parent redirecting to %s",
  (kind) => {
    const context = fixture();
    const target =
      kind === "repository"
        ? join(context.repository, "home/.config")
        : join(context.root, "outside");
    mkdirSync(target, { recursive: true });
    const protectedPath = join(target, "starship.toml");
    writeFileSync(protectedPath, "keep\n");
    symlinkSync(target, join(context.home, ".config"));
    const result = clean(context);
    expect(result.exitCode).not.toBe(0);
    expect(result.stderr).toContain("symbolic parent");
    expect(readFileSync(protectedPath, "utf8")).toBe("keep\n");
  },
);

test.each([
  { arguments: ["--force"] },
  { arguments: ["--apply", "--apply"] },
  { arguments: ["--apply", "extra"] },
])(
  "refuses unsupported or duplicated arguments before deleting: %j",
  ({ arguments: arguments_ }) => {
    const context = fixture();
    const { destination } = managedLink(context);
    expect(clean(context, arguments_).exitCode).not.toBe(0);
    expect(pathExists(destination)).toBeTrue();
  },
);

test.each(["/", "relative", "repository", "inside-repository"])(
  "refuses unsafe HOME %s before mutation",
  (kind) => {
    const context = fixture();
    const homes = new Map([
      ["repository", context.repository],
      ["inside-repository", join(context.repository, "home")],
    ]);
    const home = homes.get(kind) ?? kind;
    const result = runDeploymentHelper(context, {
      helper: "clean-deployment.ts",
      arguments: [context.repository, home, "--apply"],
    });
    expect(result.exitCode).not.toBe(0);
    expect(
      readFileSync(join(context.repository, "home/.arnes.yaml"), "utf8"),
    ).toContain("version: 1");
  },
);

test.each([
  "invalid YAML: [",
  "version: 1\nskills:\n  - slug: ../escape\n    installations: []\n",
])("refuses an invalid inventory before deleting", (manifest) => {
  const context = fixture();
  const { destination } = managedLink(context);
  writeFileSync(join(context.repository, "home/.arnes.yaml"), manifest);
  expect(clean(context).exitCode).not.toBe(0);
  expect(pathExists(destination)).toBeTrue();
});

test("preserves memory databases, credentials, application data and third party executables", () => {
  const context = fixture();
  const retained = [
    ".local/share/nvim/data",
    ".cache/nvim/cache",
    ".remem/memory.db",
    ".codex/auth.json",
    ".local/bin/remem",
    ".tmux/plugins/tpm/tpm",
  ];
  for (const path of retained) {
    mkdirSync(dirname(join(context.home, path)), { recursive: true });
    writeFileSync(join(context.home, path), "keep\n");
  }
  expect(clean(context).exitCode).toBe(0);
  for (const path of retained) {
    expect(readFileSync(join(context.home, path), "utf8")).toBe("keep\n");
  }
});

test("reports the unresolved worker service before removing its config or deployments", () => {
  const context = fixture();
  const { destination } = managedLink(context);
  const plist = join(
    context.home,
    "Library/LaunchAgents/dev.remem.worker.plist",
  );
  mkdirSync(dirname(plist), { recursive: true });
  writeFileSync(plist, "owned worker definition\n");
  const result = clean(context);
  expect(result.exitCode).not.toBe(0);
  expect(result.stderr).toContain("Worker service cleanup is unresolved");
  expect(pathExists(destination)).toBeTrue();
  expect(readFileSync(plist, "utf8")).toBe("owned worker definition\n");
});

test.each(["trailing-slash", "dot-segment"])(
  "cleans a normalized %s HOME without an unbounded parent walk",
  async (kind) => {
    const context = fixture();
    const { destination } = managedLink(context);
    const canonicalHome = realpathSync(context.home);
    const home =
      kind === "trailing-slash"
        ? `${canonicalHome}/`
        : `${canonicalHome}/../home`;
    const child = Bun.spawn(
      [
        process.execPath,
        join(import.meta.dir, "clean-deployment.ts"),
        context.repository,
        home,
        "--apply",
      ],
      { stdout: "pipe", stderr: "pipe" },
    );
    const timeoutMilliseconds = 1000;
    const timeout = setTimeout((): void => {
      child.kill();
    }, timeoutMilliseconds);
    try {
      const exit = await child.exited;
      expect(exit).toBe(0);
      expect(pathExists(destination)).toBeFalse();
    } finally {
      clearTimeout(timeout);
      child.kill();
    }
  },
);
