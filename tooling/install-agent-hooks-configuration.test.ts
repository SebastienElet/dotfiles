import { afterEach, expect, test } from "bun:test";
import {
  chmodSync,
  lstatSync,
  readFileSync,
  realpathSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import {
  cleanupDeploymentFixtures,
  expectSuccess,
  fileIdentity,
} from "./deployment-test-support.ts";
import {
  hooksFixture,
  runHookInstaller,
  writeHookManifest,
} from "./install-agent-hooks-test-support.ts";
import { join } from "node:path";
import { z } from "zod";

const commandSchema = z.looseObject({ command: z.string() });
const groupSchema = z.looseObject({ hooks: z.array(commandSchema).optional() });
const hookConfigurationSchema = z.looseObject({
  hooks: z.record(z.string(), z.array(groupSchema)),
});
const secureMode = 0o640;
const permissionMask = 0o777;
afterEach(cleanupDeploymentFixtures);

function seedLegacyConfiguration(destination: string, old: string): void {
  writeFileSync(
    destination,
    JSON.stringify({
      model: "personal",
      permissions: { allow: ["Read"] },
      hooks: {
        Stop: [
          {
            matcher: "keep",
            hooks: [
              { type: "command", command: "third-party", async: true },
              { type: "command", command: old },
            ],
          },
        ],
        FutureEvent: [
          {
            hooks: [{ type: "future", command: "keep-future", custom: true }],
          },
        ],
      },
    }),
  );
}

function assertManagedMatchers(
  hooks: Readonly<Record<string, readonly unknown[]>>,
  source: Readonly<{ agent: string; home: string; repository: string }>,
): void {
  expect(hooks.SessionStart).toContainEqual({
    matcher: "startup|resume|clear|compact",
    hooks: [
      {
        type: "command",
        command: `'${join(source.home, ".local/bin/output-discipline-hook")}'`,
        timeout: 30,
      },
    ],
  });
  expect(hooks.PostToolUse).toContainEqual({
    matcher: source.agent === "claude" ? "Edit|Write|MultiEdit" : "apply_patch",
    hooks: [
      {
        type: "command",
        command: `'${join(realpathSync(source.repository), "tooling/format-edited-file")}'`,
        timeout: 30,
      },
    ],
  });
}

test.each(["claude", "codex"] as const)(
  "preserves %s settings and third-party hooks while replacing legacy routes",
  (agent) => {
    const { fixture, destination } = hooksFixture(agent, [
      "measurement",
      "output-discipline",
      "handoff",
      "format-edited-file",
    ]);
    const old = `'${join(fixture.home, ".local/bin/arnes")}' measure hook --agent ${agent === "claude" ? "claude-code" : agent}`;
    seedLegacyConfiguration(destination, old);
    chmodSync(destination, secureMode);
    expectSuccess(runHookInstaller(fixture, agent));
    const result = hookConfigurationSchema.parse(
      JSON.parse(readFileSync(destination, "utf8")),
    );
    expect(result).toMatchObject({
      model: "personal",
      permissions: { allow: ["Read"] },
      hooks: {
        FutureEvent: [
          { hooks: [{ type: "future", command: "keep-future", custom: true }] },
        ],
      },
    });
    expect(result.hooks.Stop).toContainEqual({
      matcher: "keep",
      hooks: [{ type: "command", command: "third-party", async: true }],
    });
    expect(readFileSync(destination, "utf8")).not.toContain(old);
    assertManagedMatchers(result.hooks, { ...fixture, agent });
    expect(lstatSync(destination).mode & permissionMask).toBe(secureMode);
    const before = fileIdentity(destination);
    expectSuccess(runHookInstaller(fixture, agent));
    expect(fileIdentity(destination)).toEqual(before);
  },
);

test("Cursor keeps version and unrelated handlers and removes collection from excluded events", () => {
  const { fixture, destination } = hooksFixture("cursor");
  const command = `'${join(fixture.home, ".local/bin/agent-measure")}' hook --agent cursor`;
  writeFileSync(
    destination,
    JSON.stringify({
      version: 3,
      custom: true,
      hooks: {
        afterAgentThought: [{ command }, { command: "personal" }],
        stop: [{ type: "prompt", prompt: "keep" }],
      },
    }),
  );
  expectSuccess(runHookInstaller(fixture, "cursor"));
  expect(JSON.parse(readFileSync(destination, "utf8"))).toMatchObject({
    version: 3,
    custom: true,
    hooks: {
      afterAgentThought: [{ command: "personal" }],
      stop: [{ type: "prompt", prompt: "keep" }, { command }],
    },
  });
});

test.each(["claude", "codex"] as const)(
  "removes undeclared %s managed hooks without touching other handlers",
  (agent) => {
    const { fixture, destination } = hooksFixture(agent, [
      "measurement",
      "handoff",
    ]);
    expectSuccess(runHookInstaller(fixture, agent));
    writeHookManifest(fixture, agent, []);
    expectSuccess(runHookInstaller(fixture, agent));
    expect(readFileSync(destination, "utf8")).not.toContain("agent-measure");
    expect(readFileSync(destination, "utf8")).not.toContain("agent-handoff");
  },
);

test("a missing executable refuses installation before changing existing settings", () => {
  const { fixture, destination } = hooksFixture("claude");
  writeFileSync(destination, '{"model":"personal"}');
  const before = fileIdentity(destination);
  rmSync(join(fixture.home, ".local/bin/agent-measure"));
  expect(runHookInstaller(fixture, "claude").exitCode).not.toBe(0);
  expect(fileIdentity(destination)).toEqual(before);
});

test("normalizes the managed handoff execution fields while preserving its timeout", () => {
  const { fixture, destination } = hooksFixture("claude", ["handoff"]);
  writeFileSync(
    destination,
    JSON.stringify({
      hooks: {
        Stop: [
          {
            hooks: [
              {
                type: "command",
                command: join(fixture.home, ".local/bin/agent-handoff"),
                timeout: 7,
                async: true,
                asyncRewake: true,
                once: true,
                if: "never",
              },
            ],
          },
        ],
      },
    }),
  );
  expectSuccess(runHookInstaller(fixture, "claude"));
  expect(JSON.parse(readFileSync(destination, "utf8"))).toEqual({
    hooks: {
      Stop: [
        {
          hooks: [
            {
              type: "command",
              command: join(fixture.home, ".local/bin/agent-handoff"),
              timeout: 7,
              args: [],
            },
          ],
        },
      ],
    },
  });
});

test("reconciles the previous handoff symlink target without removing a third-party command", () => {
  const { fixture, destination } = hooksFixture("codex", ["handoff"]);
  const executable = join(fixture.home, ".local/bin/agent-handoff");
  const target = join(fixture.root, "old-handoff");
  writeFileSync(target, "fixture", { mode: 0o755 });
  rmSync(executable);
  symlinkSync(target, executable);
  writeFileSync(
    destination,
    JSON.stringify({
      hooks: {
        Stop: [
          {
            hooks: [
              { type: "command", command: target, timeout: 7 },
              { type: "command", command: "third-party" },
            ],
          },
        ],
      },
    }),
  );
  expectSuccess(runHookInstaller(fixture, "codex"));
  expect(readFileSync(destination, "utf8")).not.toContain(target);
  expect(readFileSync(destination, "utf8")).toContain("third-party");
  const result = hookConfigurationSchema.parse(
    JSON.parse(readFileSync(destination, "utf8")),
  );
  expect(result.hooks.Stop).toContainEqual({
    hooks: [{ type: "command", command: executable, timeout: 7 }],
  });
});

test("preserves customized Remem arguments while removing only retired empty-argument hooks", () => {
  const { fixture, destination } = hooksFixture("claude", []);
  const command = `${join(fixture.home, ".local/bin/remem")} context --host claude-code`;
  const personal = { type: "command", command, args: ["--personal"] };
  writeFileSync(
    destination,
    JSON.stringify({
      hooks: {
        Stop: [{ hooks: [personal, { type: "command", command, args: [] }] }],
      },
    }),
  );
  expectSuccess(runHookInstaller(fixture, "claude"));
  expect(JSON.parse(readFileSync(destination, "utf8"))).toEqual({
    hooks: { Stop: [{ hooks: [personal] }] },
  });
});
