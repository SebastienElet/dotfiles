import { dispatchSelectedIssue } from "./launch.ts";
import { openPopupQuestions } from "./popup-questions.ts";
import { parseIssueClick } from "./click.ts";
import { selectIssueWork } from "./selection.ts";
import { z } from "zod";

async function pickIssueWork(): Promise<void> {
  const contextJson = z
    .string()
    .min(1)
    .parse(process.env.DOTFILES_ISSUE_CLICK_JSON);
  const click = parseIssueClick(contextJson);
  const questions = openPopupQuestions();
  try {
    const selection = await selectIssueWork(click, questions.ask);
    if (selection === null) {
      return;
    }
    const outcome = await dispatchSelectedIssue(selection);
    await questions.ask(
      `${JSON.stringify(outcome)}\n${outcome.kind === "started" ? "Preparation is running; the final issue worktree is not yet verified." : "Inspect the retained pane to continue; no existing agent received additional input."}\nEnter or Escape closes this picker.`,
    );
  } catch (error) {
    await questions.ask(
      `${JSON.stringify({ state: "stopped", reason: error instanceof Error ? error.message : String(error) })}\nExisting panes and worktrees are retained. Inspect their state before another click. Enter or Escape closes.`,
    );
    process.exitCode = 1;
  } finally {
    questions.close();
  }
}

if (import.meta.main) {
  try {
    await pickIssueWork();
  } catch (error) {
    process.stderr.write(
      `${JSON.stringify({ state: "picker-unavailable", reason: error instanceof Error ? error.message : String(error) })}\n`,
    );
    process.exitCode = 1;
  }
}
