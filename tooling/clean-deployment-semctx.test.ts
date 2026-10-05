import { afterEach, expect, test } from "bun:test";
import {
  cleanupDeploymentFixtures,
  createDeploymentFixture,
  runDeploymentHelper,
} from "./deployment-test-support.ts";
import { dirname, join } from "node:path";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";

afterEach(cleanupDeploymentFixtures);

function fixture(): ReturnType<typeof createDeploymentFixture> {
  const context = createDeploymentFixture("clean-semctx");
  mkdirSync(join(context.repository, "home"));
  writeFileSync(
    join(context.repository, "home/.arnes.yaml"),
    '{"version":1,"skills":[]}',
  );
  return context;
}

type Fixture = ReturnType<typeof fixture>;

function write(
  context: Fixture,
  relativePath: string,
  contents: string,
): string {
  const path = join(context.home, relativePath);
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, contents);
  return path;
}

function clean(context: Fixture): ReturnType<typeof runDeploymentHelper> {
  return runDeploymentHelper(context, {
    helper: "clean-deployment.ts",
    arguments: [context.repository, context.home, "--apply"],
  });
}

test.each(["claude", "codex"])(
  "removes owned semctx-nudge commands for %s while preserving neighboring handlers",
  (agent) => {
    const context = fixture();
    const executable = join(context.repository, "tooling/semctx-nudge");
    const command = `'${executable}' --host ${agent}`;
    const owned = [command, `${executable} --host ${agent}`].map((value) => ({
      type: "command",
      command: value,
    }));
    const neighbors = [
      { type: "command", command: `'${executable}' --host foreign` },
      { type: "command", command, args: ["keep"] },
      { type: "prompt", command },
      { type: "command", command: `${command} --extra` },
    ];
    const relativePath =
      agent === "claude" ? ".claude/settings.json" : ".codex/hooks.json";
    const path = write(
      context,
      relativePath,
      JSON.stringify({
        autoMemoryEnabled: false,
        hooks: {
          SessionStart: [{ matcher: "keep", hooks: [...owned, ...neighbors] }],
        },
      }),
    );
    expect(clean(context).exitCode).toBe(0);
    expect(JSON.parse(readFileSync(path, "utf8"))).toEqual({
      autoMemoryEnabled: false,
      hooks: { SessionStart: [{ matcher: "keep", hooks: neighbors }] },
    });
  },
);

test("preserves semctx-nudge handlers outside their installed Cursor scope", () => {
  const context = fixture();
  const original = {
    hooks: {
      SessionStart: [
        {
          type: "command",
          command: `'${context.repository}/tooling/semctx-nudge' --host codex`,
        },
      ],
    },
  };
  const path = write(context, ".cursor/hooks.json", JSON.stringify(original));
  expect(clean(context).exitCode).toBe(0);
  expect(JSON.parse(readFileSync(path, "utf8"))).toEqual(original);
});
