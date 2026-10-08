import { expect, test } from "bun:test";
import { openIssuePicker } from "./action.ts";

test("opens a native popup without forbidden topology targets while retaining clicked context", async () => {
  const context = {
    clicked_url: "https://github.com/fixture/project/issues/17?full=1#comment",
    focused_pane_id: "fixture:p1",
    invocation_source: "link_click",
    selected_text: "displayed text is not a launch command: '$()`\n",
    workspace_cwd: "/tmp/fixture-source",
    workspace_id: "fixture",
  };
  const calls: (readonly string[])[] = [];
  await openIssuePicker(
    {
      HERDR_PLUGIN_CONTEXT_JSON: JSON.stringify(context),
      HERDR_PLUGIN_ID: "dotfiles.issue-worktree",
    },
    (commandArguments: readonly string[]) => {
      calls.push(commandArguments);
      if (
        commandArguments.includes("--workspace") ||
        commandArguments.includes("--target-pane")
      ) {
        return Promise.reject(
          new Error("overlay and popup plugin panes target the active pane"),
        );
      }
      return Promise.resolve({ type: "ok" });
    },
  );
  const passed = calls[0]?.find((argument) =>
    argument.startsWith("DOTFILES_ISSUE_CLICK_JSON="),
  );
  expect(passed).toBe(`DOTFILES_ISSUE_CLICK_JSON=${JSON.stringify(context)}`);
  expect(calls).toHaveLength(1);
});

test("rejects malformed click data before calling native Herdr", async () => {
  let called = false;
  const action = openIssuePicker(
    {
      HERDR_PLUGIN_CONTEXT_JSON: "{}",
      HERDR_PLUGIN_ID: "dotfiles.issue-worktree",
    },
    () => {
      called = true;
      return Promise.resolve({ type: "ok" });
    },
  );
  const [result] = await Promise.allSettled([action]);
  expect(result?.status).toBe("rejected");
  expect(called).toBe(false);
});
