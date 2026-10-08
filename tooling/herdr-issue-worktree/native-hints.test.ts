import { afterEach, expect, test } from "bun:test";
import { mkdtempSync, rmSync } from "node:fs";
import type { Bindings } from "./binding-store.ts";
import type { HerdrCommand } from "./native-herdr.ts";
import type { IssueFixture } from "./dispatch-test-support.ts";
import { MemoryHerdr } from "./memory-herdr-test-support.ts";
import { createNativeHerdr } from "./native-herdr.ts";
import { dispatchWithReservation } from "./launch.ts";
import { issueFixture } from "./dispatch-test-support.ts";
import { join } from "node:path";
import { tmpdir } from "node:os";

const fixtures: IssueFixture[] = [];
const directories: string[] = [];
const agentNameIndex = 2;
afterEach(() => {
  for (const fixture of fixtures.splice(0)) {
    fixture.dispose();
  }
  for (const directory of directories.splice(0)) {
    rmSync(directory, { recursive: true, force: true });
  }
});
type NativeTestPort = Readonly<
  Pick<
    MemoryHerdr,
    "fixture" | "state" | "snapshot" | "createPreparation" | "start"
  >
>;
function commandFor(native: NativeTestPort): HerdrCommand {
  return async (args) => {
    if (args[0] === "api") {
      return { snapshot: await native.snapshot() };
    }
    if (args[0] === "worktree") {
      return {
        source: {
          repo_key: native.fixture.selection.repository.commonDirectory,
          source_checkout_path: native.fixture.selection.repository.root,
          source_workspace_id: "source",
        },
      };
    }
    if (args[0] === "tab") {
      const paneId = await native.createPreparation(
        native.fixture.selection,
        args[args.indexOf("--label") + 1] ?? "",
      );
      return {
        root_pane: native.state.panes.find((pane) => pane.pane_id === paneId),
      };
    }
    if (args[0] === "pane") {
      throw new Error("Display hints timed out");
    }
    if (args[0] !== "agent") {
      throw new Error("Unexpected native command");
    }
    await native.start({
      paneId: args[args.indexOf("--pane") + 1] ?? "",
      name: args[agentNameIndex] ?? "",
      kind: "codex",
      prompt: args.at(-1) ?? "",
    });
    return { type: "agent_started" };
  };
}
const failures = ["hints", "binding-before", "binding-after"] as const;
test.each([...failures])(
  "keeps durable identity authoritative across %s failures",
  async (failure) => {
    const fixture = issueFixture();
    fixtures.push(fixture);
    const directory = mkdtempSync(join(tmpdir(), "herdr-hints-"));
    directories.push(directory);
    const native = new MemoryHerdr(fixture);
    let durable: Bindings = {};
    const adapter = createNativeHerdr(
      commandFor(native),
      {
        directory,
        read: () => Promise.resolve(durable),
        patch: (paneId, tokens) => {
          if (failure === "binding-before") {
            return Promise.reject(new Error("Binding persistence failed"));
          }
          durable = { ...durable, [paneId]: { ...durable[paneId], ...tokens } };
          return failure === "binding-after"
            ? Promise.reject(new Error("Binding write outcome unknown"))
            : Promise.resolve();
        },
      },
      native.nativeEnvironment,
    );
    const result = await dispatchWithReservation(
      fixture.selection,
      adapter,
      directory,
    ).catch((error: unknown) => error);
    if (failure !== "hints") {
      if (!(result instanceof Error)) {
        throw new Error("Expected a persistence failure");
      }
      expect(result.message).toContain("Binding");
      expect(native.initialPrompts).toHaveLength(0);
      return;
    }
    expect(result).toMatchObject({ kind: "started" });
    expect(durable["preparation:p1"]).toMatchObject({
      issue_phase: "sent",
      issue_session: "preparation-session",
    });
    expect(
      await dispatchWithReservation(fixture.selection, adapter, directory),
    ).toMatchObject({ kind: "reused" });
    expect(native.initialPrompts).toHaveLength(1);
  },
);
